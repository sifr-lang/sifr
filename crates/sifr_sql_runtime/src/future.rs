use crate::{SqlError, SqlErrorKind};
use std::future::Future;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct ProviderFuture<'a, T> {
    inner: Pin<Box<dyn Future<Output = Result<T, SqlError>> + Send + 'a>>,
}

impl<'a, T> ProviderFuture<'a, T> {
    #[must_use]
    pub fn new(future: impl Future<Output = Result<T, SqlError>> + Send + 'a) -> Self {
        Self {
            inner: Box::pin(future),
        }
    }

    #[must_use]
    pub fn from_factory<F, Factory>(factory: Factory) -> Self
    where
        F: Future<Output = Result<T, SqlError>> + Send + 'a,
        Factory: FnOnce() -> F,
    {
        match catch_unwind(AssertUnwindSafe(factory)) {
            Ok(future) => Self::new(future),
            Err(_) => Self::new(async { Err(SqlError::new(SqlErrorKind::Provider)) }),
        }
    }
}

impl<T> Future for ProviderFuture<'_, T> {
    type Output = Result<T, SqlError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        catch_unwind(AssertUnwindSafe(|| self.inner.as_mut().poll(context)))
            .unwrap_or_else(|_| Poll::Ready(Err(SqlError::new(SqlErrorKind::Provider))))
    }
}
