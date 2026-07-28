use crate::interop::{catch_rust_panic, RustPanicErrorBridge};
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackBackpressure {
    Direct,
    Bounded(usize),
    Unbounded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackOverflow {
    Error,
    DropOldest,
    DropNewest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackShutdown {
    Drain,
    Cancel,
    DetachForbidden,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThreadsafeCallbackPolicy {
    pub backpressure: CallbackBackpressure,
    pub overflow: CallbackOverflow,
    pub shutdown: CallbackShutdown,
}

/// An owned callback that a package bridge may retain and invoke from managed
/// worker threads.
///
/// Queueing remains package-specific, but the exact declaration policy travels
/// with the callback so the bridge cannot silently substitute different
/// backpressure, overflow, or shutdown semantics. Every invocation contains a
/// panic boundary and exposes only the stable redacted panic error.
pub struct ThreadsafeCallbackBridge<Args, Output> {
    callback: Arc<dyn Fn(Args) -> Output + Send + Sync + 'static>,
    policy: ThreadsafeCallbackPolicy,
}

impl<Args, Output> ThreadsafeCallbackBridge<Args, Output> {
    #[must_use]
    pub fn new<F>(policy: ThreadsafeCallbackPolicy, callback: F) -> Self
    where
        F: Fn(Args) -> Output + Send + Sync + 'static,
    {
        Self {
            callback: Arc::new(callback),
            policy,
        }
    }

    #[must_use]
    pub const fn policy(&self) -> ThreadsafeCallbackPolicy {
        self.policy
    }

    pub fn call(&self, args: Args) -> Result<Output, RustPanicErrorBridge> {
        catch_rust_panic(|| (self.callback)(args))
    }
}

impl<Args, Output> Clone for ThreadsafeCallbackBridge<Args, Output> {
    fn clone(&self) -> Self {
        Self {
            callback: Arc::clone(&self.callback),
            policy: self.policy,
        }
    }
}

impl<Args, Output> fmt::Debug for ThreadsafeCallbackBridge<Args, Output> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ThreadsafeCallbackBridge")
            .field("policy", &self.policy)
            .finish_non_exhaustive()
    }
}

/// A callback that may be invoked only while its generated Rust bridge call is
/// active.
///
/// The borrowed closure prevents a bridge from retaining the callback beyond
/// the call. The `Rc` marker deliberately makes the value neither `Send` nor
/// `Sync`, so moving a call-scoped callback to an unmanaged worker thread is
/// rejected by rustc.
pub struct CallScopedCallbackBridge<'call, Args, Output> {
    callback: &'call dyn Fn(Args) -> Output,
    _current_thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl<'call, Args, Output> CallScopedCallbackBridge<'call, Args, Output> {
    #[must_use]
    pub fn new(callback: &'call dyn Fn(Args) -> Output) -> Self {
        Self {
            callback,
            _current_thread: std::marker::PhantomData,
        }
    }

    pub fn call(&self, args: Args) -> Output {
        (self.callback)(args)
    }
}

impl<Args, Output> fmt::Debug for CallScopedCallbackBridge<'_, Args, Output> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CallScopedCallbackBridge")
            .finish_non_exhaustive()
    }
}
