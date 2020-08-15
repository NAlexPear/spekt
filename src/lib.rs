/*!
A `std::future::Future` and `Result`-based testing trait for managing the lifecycle of stateful, asynchronous tests.

## Why `spekt`:

Most Rust unit tests terminate based on `panic`s (generally triggered by `assert!`),
with resource clean-up implemented manually through the `Drop` trait. Working synchronously with a stateful resource
like a database might look like this:

```
use postgres::{Client, NoTls, Row, error::Error as PostgresError};

struct PostgresTest {
    client: Client
}

impl PostgresTest {
    fn new() -> Self  {
        let mut client = Client::connect("host=localhost user=postgres", NoTls).expect("Error connecting to database");

        Self { client }
    }

    fn add_test_table(&self) -> Result<Row, PostgresError> {
        self.client.batch_execute("CREATE TABLE my_test_table ()")
    }
}

impl Drop for PostgresTest {
    fn drop(&mut self) {
        self.client.batch_execute("DROP TABLE my_test_table").expect("Error cleaning up test table");
    }
}

#[test]
fn adds_queryable_test_table() {
    let client = TestClient::new();
    let create_response = client.adds_test_table();

    assert!(create_response.is_ok(), "Error creating test table");

    let query_response = client.query("SELECT FROM my_test_table");

    assert!(query_response.is_ok(), "Error creating test table");
}
```

While this works for many cases, there are a couple of issues with this recommendation:

1. Technically, Rust doesn't _guarantee_ that `Drop` will be run,
and [one shouldn't rely on `Drop` to be run in all cases](http://cglab.ca/%7Eabeinges/blah/everyone-poops/).
2. `Drop` also cannot be asynchronous!
There has been much discussion around [Asynchronous destructors](https://internals.rust-lang.org/t/asynchronous-destructors/11127),
but no reliable destructor trait has yet materialized for `async` functions.
3. `panic`-based assertions (and their associated unwinding) also behave in ways that
[might be unpredictable across runtimes](https://github.com/tokio-rs/tokio/issues/2002).
This is, specifically, an [issue in tests](https://github.com/tokio-rs/tokio/issues/2699) for which there is no good universal solution.
4. In addition, while `new` and `Drop` make sense for resources, those conventions make less sense for the more abstract idea of a "Test".
In most testing frameworks, the idea of a "test" is the combination of a some stateful test context initialized `before` the actual test,
a test case that can mutate its own context, and some clean-up to be run `after` the actual test.

`spekt` avoids all of these issues by providing a `Test` trait
that encompasses the `before` -> `test` -> `after` lifecycle of stateful `async` tests that use `Result` to drive assertions.

## How to use:

`spekt::Test` can be implemented for any `Send + Sync` test state, enabling a `test()` method that returns a `std::future::Future`.
The returned `Future` is runtime-agnostic, and can be evaluated synchronously with `.wait()`, through a per-suite custom runtime
(e.g. [`tokio::runtime::Runtime`](https://docs.rs/tokio/0.2.22/tokio/runtime/struct.Runtime.html)),
or through an `async` test-runner like [`tokio::test`](https://docs.rs/tokio/0.2.22/tokio/attr.test.html).

Rewriting the example above with `spekt::Test`:

```
use tokio_postgres::{Client, NoTls, Row, error::Error as PostgresError};
use spekt::Test;

struct PostgresTest {
    client: Client
}

// spekt optionally re-exports async_trait
#[spekt::async_trait]
impl Test for PostgresTest {
    type Error = anyhow::Error; // any Error will do, but anyhow is recommended

    async fn before() -> Result<Self, Self::Error> {
        let mut client = Client::connect("host=localhost user=postgres", NoTls).await?;

        client.batch_execute("CREATE TABLE my_test_table ()").await?;

        Ok(Self { client })
    }

    async fn after(&self) -> Result<(), Self::Error> {
        self.client.batch_execute("DROP TABLE my_test_table")?;

        Ok(())
    }
}

// any executor will do, but tokio::test is recommended
#[tokio::test]
async fn adds_queryable_test_table() {
    // PostgresTest::test runs before(), passing the output to test(), and runs after() regardless
    // of the result of the test run itself, bubbling all Self::Errors to top-level test failures
    PostgresTest::test(|context| async move {
        context.client.query("SELECT FROM my_test_table").await?;

        Ok(())
    }).await
}
```
*/
#[deny(missing_docs, unreachable_pub)]
mod test;

pub use self::test::*;
pub use async_trait::async_trait;
