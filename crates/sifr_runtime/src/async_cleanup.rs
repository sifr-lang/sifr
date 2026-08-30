//! Runtime-neutral support for compiler-generated asynchronous cleanup.

use std::any::Any;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AsyncCleanupEvidence {
    CleanupFailed {
        error: String,
        location: String,
        resource: String,
        operation: String,
        budget: Duration,
    },
    CleanupTimedOut {
        location: String,
        resource: String,
        operation: String,
        budget: Duration,
    },
}

impl AsyncCleanupEvidence {
    #[must_use]
    pub fn cleanup_failed(
        error: String,
        location: String,
        resource: String,
        operation: String,
        budget: Duration,
    ) -> Self {
        Self::CleanupFailed {
            error,
            location,
            resource,
            operation,
            budget,
        }
    }

    #[must_use]
    pub fn cleanup_timed_out(
        location: String,
        resource: String,
        operation: String,
        budget: Duration,
    ) -> Self {
        Self::CleanupTimedOut {
            location,
            resource,
            operation,
            budget,
        }
    }
}

pub struct CatchUnwindFuture<F> {
    future: Pin<Box<F>>,
}

#[must_use]
pub fn catch_unwind_future<F>(future: F) -> CatchUnwindFuture<F>
where
    F: Future,
{
    CatchUnwindFuture {
        future: Box::pin(future),
    }
}

impl<F> Future for CatchUnwindFuture<F>
where
    F: Future,
{
    type Output = Result<F::Output, Box<dyn Any + Send>>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        match std::panic::catch_unwind(AssertUnwindSafe(|| self.future.as_mut().poll(context))) {
            Ok(Poll::Ready(output)) => Poll::Ready(Ok(output)),
            Ok(Poll::Pending) => Poll::Pending,
            Err(payload) => Poll::Ready(Err(payload)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn catches_future_poll_panics() {
        let result = catch_unwind_future(async {
            panic!("poll panic");
        })
        .await;
        assert!(result.is_err());
    }
}
