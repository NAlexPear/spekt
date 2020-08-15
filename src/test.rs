use async_trait::async_trait;
use std::{future::Future, sync::Arc};

/// Test-running trait to handle test lifecycles
#[async_trait]
pub trait Test
where
    Self: Sized + Send + Sync + 'static,
{
    /// The format-able error shared by each step. anyhow::Error is recommended!
    type Error: std::fmt::Display + Send + Sync;

    /// Initialize test suite with new instance of test's state
    async fn before() -> Result<Self, Self::Error>;

    /// Optionally clean up after test run
    async fn after(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Run a Result-emitting test task, handling assertion errors gracefully
    async fn test<F, T>(task: T)
    where
        F: Future<Output = Result<(), Self::Error>> + Send,
        T: Send + Sync + FnOnce(Arc<Self>) -> F,
    {
        let before = Self::before().await;

        let state = match before {
            Err(error) => return assert!(false, format!("{}", error)),
            Ok(state) => Arc::new(state),
        };

        let test_run = task(state.clone()).await;
        let after = state.after().await;

        if let Err(error) = test_run {
            assert!(false, format!("{}", error));
        }

        if let Err(error) = after {
            assert!(false, format!("{}", error));
        }
    }
}
