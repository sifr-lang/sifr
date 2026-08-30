//! Stable Rust interop helper types used by generated bridge contracts.

use crate::{IntegerParseError, IntegerRangeError, SifrInt};
use std::any::Any;
use std::cell::Cell;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub use crate::interop_callbacks::{
    CallScopedCallbackBridge, CallbackBackpressure, CallbackOverflow, CallbackShutdown,
    ThreadsafeCallbackBridge, ThreadsafeCallbackPolicy,
};

#[cfg(any(test, feature = "structural"))]
pub mod structural;

type PanicHook = Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Send + Sync + 'static>;
type SharedPanicHook = Arc<Mutex<Option<PanicHook>>>;

struct RustInteropPanicHookState {
    active_boundaries: usize,
    previous_hook: Option<SharedPanicHook>,
}

// Hook installation/removal is serialized, but the mutex is never held while
// user code executes. Concurrent and nested boundaries share one active hook.
static RUST_INTEROP_PANIC_HOOK_STATE: Mutex<RustInteropPanicHookState> =
    Mutex::new(RustInteropPanicHookState {
        active_boundaries: 0,
        previous_hook: None,
    });

thread_local! {
    static RUST_INTEROP_PANIC_CATCH_DEPTH: Cell<usize> = const { Cell::new(0) };
}

#[doc(hidden)]
pub struct SilentPanicBoundary {
    _private: (),
}

impl SilentPanicBoundary {
    #[doc(hidden)]
    pub fn enter() -> Self {
        let mut state = panic_hook_state();
        if state.active_boundaries == 0 {
            let previous_hook = Arc::new(Mutex::new(Some(std::panic::take_hook())));
            let forwarded_hook = Arc::clone(&previous_hook);
            std::panic::set_hook(Box::new(move |info| {
                let suppress = RUST_INTEROP_PANIC_CATCH_DEPTH
                    .try_with(|depth| depth.get() > 0)
                    .unwrap_or(false);
                if !suppress {
                    let hook = match forwarded_hook.lock() {
                        Ok(hook) => hook,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    if let Some(hook) = hook.as_ref() {
                        hook(info);
                    }
                }
            }));
            state.previous_hook = Some(previous_hook);
        }
        state.active_boundaries = state.active_boundaries.saturating_add(1);
        Self { _private: () }
    }

    #[doc(hidden)]
    pub fn catch_unwind<T, F>(&self, f: F) -> std::thread::Result<T>
    where
        F: FnOnce() -> T,
    {
        let _thread_guard = SilentPanicThreadGuard::enter();
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
    }
}

struct SilentPanicThreadGuard;

impl SilentPanicThreadGuard {
    fn enter() -> Self {
        RUST_INTEROP_PANIC_CATCH_DEPTH.with(|depth| {
            depth.set(depth.get().saturating_add(1));
        });
        Self
    }
}

impl Drop for SilentPanicThreadGuard {
    fn drop(&mut self) {
        RUST_INTEROP_PANIC_CATCH_DEPTH.with(|depth| {
            depth.set(depth.get().saturating_sub(1));
        });
    }
}

impl Drop for SilentPanicBoundary {
    fn drop(&mut self) {
        let mut state = panic_hook_state();
        state.active_boundaries = state.active_boundaries.saturating_sub(1);
        if state.active_boundaries != 0 {
            return;
        }
        let installed_hook = std::panic::take_hook();
        let previous_hook = state.previous_hook.take();
        if let Some(previous_hook) = previous_hook {
            let restored_hook = {
                let mut previous_hook = match previous_hook.lock() {
                    Ok(hook) => hook,
                    Err(poisoned) => poisoned.into_inner(),
                };
                previous_hook.take()
            };
            if let Some(previous_hook) = restored_hook {
                std::panic::set_hook(previous_hook);
            }
        }
        drop(installed_hook);
        drop(state);
    }
}

fn panic_hook_state() -> std::sync::MutexGuard<'static, RustInteropPanicHookState> {
    match RUST_INTEROP_PANIC_HOOK_STATE.lock() {
        Ok(state) => state,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub use indexmap::IndexMap;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SifrIntBridge(SifrInt);

impl SifrIntBridge {
    #[must_use]
    pub const fn from_sifr_int(value: SifrInt) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_sifr_int(&self) -> &SifrInt {
        &self.0
    }

    #[must_use]
    pub fn into_sifr_int(self) -> SifrInt {
        self.0
    }

    pub fn parse_decimal(text: &str, max_digits: usize) -> Result<Self, IntegerParseError> {
        SifrInt::parse_decimal(text, max_digits).map(Self)
    }

    pub fn try_to_i8(&self) -> Result<i8, IntegerRangeError> {
        self.0.try_to_i8()
    }

    pub fn try_to_i16(&self) -> Result<i16, IntegerRangeError> {
        self.0.try_to_i16()
    }

    pub fn try_to_i32(&self) -> Result<i32, IntegerRangeError> {
        self.0.try_to_i32()
    }

    pub fn try_to_i64(&self) -> Result<i64, IntegerRangeError> {
        self.0.try_to_i64()
    }

    #[must_use]
    pub fn to_i64_saturating(&self) -> i64 {
        self.try_to_i64().unwrap_or_else(|_| {
            if self.0 < SifrInt::from_i64(0) {
                i64::MIN
            } else {
                i64::MAX
            }
        })
    }

    pub fn try_to_u8(&self) -> Result<u8, IntegerRangeError> {
        self.0.try_to_u8()
    }

    pub fn try_to_u16(&self) -> Result<u16, IntegerRangeError> {
        self.0.try_to_u16()
    }

    pub fn try_to_u32(&self) -> Result<u32, IntegerRangeError> {
        self.0.try_to_u32()
    }

    pub fn try_to_u64(&self) -> Result<u64, IntegerRangeError> {
        self.0.try_to_u64()
    }
}

impl From<SifrInt> for SifrIntBridge {
    fn from(value: SifrInt) -> Self {
        Self(value)
    }
}

impl From<&SifrInt> for SifrIntBridge {
    fn from(value: &SifrInt) -> Self {
        Self(value.clone())
    }
}

impl From<i64> for SifrIntBridge {
    fn from(value: i64) -> Self {
        Self(SifrInt::from(value))
    }
}

impl fmt::Display for SifrIntBridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RustInteropConversionError {
    Width {
        target: &'static str,
        value: String,
    },
    Overflow {
        target: &'static str,
        value: String,
    },
    InvalidUtf8 {
        context: &'static str,
    },
    UnsupportedContainer {
        container: &'static str,
    },
    RecordLayoutMismatch {
        type_name: &'static str,
    },
    InvalidEnumDiscriminant {
        type_name: &'static str,
        discriminant: i128,
    },
}

impl fmt::Display for RustInteropConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Width { target, value } => {
                write!(f, "value {value} cannot be represented as {target}")
            }
            Self::Overflow { target, value } => {
                write!(f, "value {value} overflows {target}")
            }
            Self::InvalidUtf8 { context } => write!(f, "invalid utf-8 while converting {context}"),
            Self::UnsupportedContainer { container } => {
                write!(f, "unsupported Rust bridge container {container}")
            }
            Self::RecordLayoutMismatch { type_name } => {
                write!(f, "record bridge layout mismatch for {type_name}")
            }
            Self::InvalidEnumDiscriminant {
                type_name,
                discriminant,
            } => write!(
                f,
                "invalid discriminant {discriminant} for enum bridge {type_name}"
            ),
        }
    }
}

impl std::error::Error for RustInteropConversionError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustPanicErrorBridge {
    message: String,
}

impl RustPanicErrorBridge {
    #[must_use]
    pub fn redacted() -> Self {
        Self {
            message: "Rust bridge panicked".to_string(),
        }
    }

    #[must_use]
    pub fn from_panic_payload(_payload: &(dyn Any + Send)) -> Self {
        Self::redacted()
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for RustPanicErrorBridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RustPanicErrorBridge {}

pub fn catch_rust_panic<T, F>(f: F) -> Result<T, RustPanicErrorBridge>
where
    F: FnOnce() -> T,
{
    catch_unwind_silently(f)
        .map_err(|payload| RustPanicErrorBridge::from_panic_payload(payload.as_ref()))
}

#[doc(hidden)]
pub fn catch_unwind_silently<T, F>(f: F) -> std::thread::Result<T>
where
    F: FnOnce() -> T,
{
    let boundary = SilentPanicBoundary::enter();
    boundary.catch_unwind(f)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HandleStateError {
    Closed,
    Poisoned(RustPanicErrorBridge),
}

impl fmt::Display for HandleStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed => f.write_str("Rust interop handle is closed"),
            Self::Poisoned(error) => write!(f, "Rust interop handle is poisoned: {error}"),
        }
    }
}

impl std::error::Error for HandleStateError {}

#[derive(Debug)]
pub struct GeneratedGlueToken {
    _private: (),
}

#[doc(hidden)]
pub mod __generated_glue {
    use super::GeneratedGlueToken;

    #[must_use]
    pub const fn token() -> GeneratedGlueToken {
        GeneratedGlueToken { _private: () }
    }
}

#[derive(Debug)]
pub struct Handle<T> {
    slot: HandleSlot<T>,
    _not_send_or_sync_by_default: PhantomData<Rc<()>>,
}

impl<T: Clone> Clone for Handle<T> {
    fn clone(&self) -> Self {
        let slot = match &self.slot {
            HandleSlot::Open(value) => HandleSlot::Open(value.clone()),
            HandleSlot::Closed => HandleSlot::Closed,
            HandleSlot::Poisoned(error) => HandleSlot::Poisoned(error.clone()),
        };
        Self {
            slot,
            _not_send_or_sync_by_default: PhantomData,
        }
    }
}

#[derive(Debug)]
enum HandleSlot<T> {
    Open(T),
    Closed,
    Poisoned(RustPanicErrorBridge),
}

impl<T> Handle<T> {
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self {
            slot: HandleSlot::Open(value),
            _not_send_or_sync_by_default: PhantomData,
        }
    }

    pub fn inner_ref(&self) -> Result<&T, HandleStateError> {
        match &self.slot {
            HandleSlot::Open(value) => Ok(value),
            HandleSlot::Closed => Err(HandleStateError::Closed),
            HandleSlot::Poisoned(error) => Err(HandleStateError::Poisoned(error.clone())),
        }
    }

    pub fn inner_mut(&mut self) -> Result<&mut T, HandleStateError> {
        match &mut self.slot {
            HandleSlot::Open(value) => Ok(value),
            HandleSlot::Closed => Err(HandleStateError::Closed),
            HandleSlot::Poisoned(error) => Err(HandleStateError::Poisoned(error.clone())),
        }
    }

    pub fn into_inner(self) -> Result<T, HandleStateError> {
        match self.slot {
            HandleSlot::Open(value) => Ok(value),
            HandleSlot::Closed => Err(HandleStateError::Closed),
            HandleSlot::Poisoned(error) => Err(HandleStateError::Poisoned(error)),
        }
    }

    pub fn mark_closed(&mut self, _token: GeneratedGlueToken) {
        if matches!(self.slot, HandleSlot::Open(_)) {
            self.slot = HandleSlot::Closed;
        }
    }

    pub fn mark_poisoned(&mut self, _token: GeneratedGlueToken, error: RustPanicErrorBridge) {
        self.slot = HandleSlot::Poisoned(error);
    }
}

#[derive(Debug)]
pub struct PoisonOnPanic<'a, T> {
    handle: Option<&'a mut Handle<T>>,
    token: Option<GeneratedGlueToken>,
    disarmed: bool,
}

impl<'a, T> PoisonOnPanic<'a, T> {
    #[must_use]
    pub fn new(handle: &'a mut Handle<T>, token: GeneratedGlueToken) -> Self {
        Self {
            handle: Some(handle),
            token: Some(token),
            disarmed: false,
        }
    }

    pub fn disarm(mut self) {
        self.disarmed = true;
    }
}

impl<T> Drop for PoisonOnPanic<'_, T> {
    fn drop(&mut self) {
        if self.disarmed || !std::thread::panicking() {
            return;
        }
        let Some(handle) = self.handle.as_deref_mut() else {
            return;
        };
        let Some(token) = self.token.take() else {
            return;
        };
        handle.mark_poisoned(token, RustPanicErrorBridge::redacted());
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Handle, HandleStateError, IndexMap, PoisonOnPanic, RustPanicErrorBridge, SifrIntBridge,
    };
    use crate::SifrInt;

    static PANIC_HOOK_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn panic_hook_test_guard() -> std::sync::MutexGuard<'static, ()> {
        match PANIC_HOOK_TEST_LOCK.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    #[test]
    fn exact_integer_bridge_preserves_exact_value_and_fixed_width_errors() {
        let bridge = SifrIntBridge::from(SifrInt::from_i128(i128::from(i64::MAX) + 1));

        assert_eq!(bridge.to_string(), "9223372036854775808");
        assert_eq!(
            bridge
                .try_to_i64()
                .expect_err("value must not fit i64")
                .target(),
            "i64"
        );
    }

    #[test]
    fn exact_integer_bridge_saturates_i64_conversion() {
        let too_large = SifrIntBridge::from(SifrInt::from_i128(i128::from(i64::MAX) + 1));
        let too_small = SifrIntBridge::from(SifrInt::from_i128(i128::from(i64::MIN) - 1));

        assert_eq!(SifrIntBridge::from(42_i64).to_i64_saturating(), 42);
        assert_eq!(too_large.to_i64_saturating(), i64::MAX);
        assert_eq!(too_small.to_i64_saturating(), i64::MIN);
    }

    #[test]
    fn ordered_index_map_reexport_preserves_insertion_order() {
        let mut map = IndexMap::new();
        map.insert("b".to_string(), 2_i64);
        map.insert("a".to_string(), 1_i64);

        assert_eq!(
            map.keys().cloned().collect::<Vec<_>>(),
            vec!["b".to_string(), "a".to_string()]
        );
    }

    #[test]
    fn handle_reports_closed_and_poisoned_states_without_panicking() {
        let mut handle = Handle::new("value".to_string());
        assert_eq!(handle.inner_ref().expect("open handle"), "value");
        handle.mark_closed(super::__generated_glue::token());

        assert!(matches!(handle.inner_ref(), Err(HandleStateError::Closed)));

        let mut poisoned = Handle::new(1_i64);
        poisoned.mark_poisoned(
            super::__generated_glue::token(),
            RustPanicErrorBridge::redacted(),
        );
        assert!(matches!(
            poisoned.inner_mut(),
            Err(HandleStateError::Poisoned(_))
        ));
    }

    #[test]
    fn cloned_handle_copies_cloneable_identity_without_sharing_handle_state() {
        let mut original = Handle::new("identity".to_string());
        let cloned = original.clone();

        original.mark_closed(super::__generated_glue::token());

        assert!(matches!(
            original.inner_ref(),
            Err(HandleStateError::Closed)
        ));
        assert_eq!(cloned.inner_ref().expect("clone stays open"), "identity");
    }

    #[test]
    fn handle_poison_guard_marks_open_handle_when_rust_call_unwinds() {
        let _test_guard = panic_hook_test_guard();
        let mut handle = Handle::new(1_i64);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = PoisonOnPanic::new(&mut handle, super::__generated_glue::token());
            panic!("simulated Rust bridge panic");
        }));

        assert!(result.is_err());
        assert!(matches!(
            handle.inner_ref(),
            Err(HandleStateError::Poisoned(_))
        ));
    }

    #[test]
    fn handle_poison_guard_can_be_disarmed_after_successful_call() {
        let mut handle = Handle::new(1_i64);
        {
            let guard = PoisonOnPanic::new(&mut handle, super::__generated_glue::token());
            guard.disarm();
        }

        assert_eq!(*handle.inner_ref().expect("handle remains open"), 1);
    }

    #[test]
    fn poisoned_state_wins_over_closed_state() {
        let mut handle = Handle::new(1_i64);
        handle.mark_closed(super::__generated_glue::token());
        handle.mark_poisoned(
            super::__generated_glue::token(),
            RustPanicErrorBridge::redacted(),
        );

        assert!(matches!(
            handle.inner_ref(),
            Err(HandleStateError::Poisoned(_))
        ));
    }

    #[test]
    fn catch_rust_panic_redacts_payload_details() {
        let _test_guard = panic_hook_test_guard();
        let error =
            super::catch_rust_panic(|| panic!("secret backend token")).expect_err("panic maps");

        assert_eq!(error.message(), "Rust bridge panicked");
        assert!(!error.message().contains("secret"));
    }

    #[test]
    fn catch_rust_panic_preserves_successful_values() {
        let _test_guard = panic_hook_test_guard();
        assert_eq!(super::catch_rust_panic(|| 42_i64), Ok(42));
    }

    #[test]
    fn call_scoped_callback_invokes_the_borrowed_handler() {
        let prefix = "event:".to_string();
        let handler = |(event,): (String,)| format!("{prefix}{event}");
        let callback = super::CallScopedCallbackBridge::new(&handler);

        assert_eq!(callback.call(("ready".to_string(),)), "event:ready");
    }

    #[test]
    fn call_scoped_callback_constructor_propagates_argument_types_to_the_closure() {
        fn accept_callback(
            callback: super::CallScopedCallbackBridge<
                '_,
                (super::SifrIntBridge, Vec<super::SifrIntBridge>),
                i64,
            >,
        ) -> i64 {
            callback.call((
                super::SifrIntBridge::from(2_i64),
                vec![super::SifrIntBridge::from(3_i64)],
            ))
        }

        let total = accept_callback(super::CallScopedCallbackBridge::new(&|(value, items)| {
            value.to_i64_saturating() + items[0].to_i64_saturating()
        }));

        assert_eq!(total, 5);
    }

    #[test]
    fn threadsafe_callback_preserves_policy_and_crosses_a_worker_thread() {
        let callback = super::ThreadsafeCallbackBridge::new(
            super::ThreadsafeCallbackPolicy {
                backpressure: super::CallbackBackpressure::Bounded(2),
                overflow: super::CallbackOverflow::Error,
                shutdown: super::CallbackShutdown::Drain,
            },
            |(event,): (String,)| event.to_uppercase(),
        );
        let policy = callback.policy();
        let worker = std::thread::spawn(move || callback.call(("ready".to_string(),)));

        assert_eq!(
            worker.join().expect("managed callback worker").as_deref(),
            Ok("READY")
        );
        assert_eq!(policy.backpressure, super::CallbackBackpressure::Bounded(2));
        assert_eq!(policy.overflow, super::CallbackOverflow::Error);
        assert_eq!(policy.shutdown, super::CallbackShutdown::Drain);
    }

    #[test]
    fn threadsafe_callback_redacts_panics_from_worker_threads() {
        let _test_guard = panic_hook_test_guard();
        let callback = super::ThreadsafeCallbackBridge::new(
            super::ThreadsafeCallbackPolicy {
                backpressure: super::CallbackBackpressure::Direct,
                overflow: super::CallbackOverflow::Error,
                shutdown: super::CallbackShutdown::Cancel,
            },
            |(): ()| panic!("private callback payload"),
        );
        let error = std::thread::spawn(move || callback.call(()))
            .join()
            .expect("managed callback worker")
            .expect_err("callback panic must map");

        assert_eq!(error.message(), "Rust bridge panicked");
    }

    #[test]
    fn catch_rust_panic_is_reentrant_for_nested_bridge_calls() {
        let _test_guard = panic_hook_test_guard();
        let outer = super::catch_rust_panic(|| {
            let nested = super::catch_rust_panic(|| panic!("nested private payload"))
                .expect_err("nested panic maps");
            assert_eq!(nested.message(), "Rust bridge panicked");
            42_i64
        });

        assert_eq!(outer, Ok(42));
    }

    #[test]
    fn silent_unwind_and_bridge_catch_share_the_reentrant_hook() {
        let _test_guard = panic_hook_test_guard();
        let worker = super::catch_unwind_silently(|| {
            let nested = super::catch_rust_panic(|| panic!("nested bridge payload"))
                .expect_err("nested bridge panic maps");
            assert_eq!(nested.message(), "Rust bridge panicked");
            panic!("worker payload")
        });

        assert!(worker.is_err());
        assert_eq!(super::catch_rust_panic(|| 42_i64), Ok(42));
    }

    #[test]
    fn catch_rust_panic_allows_overlapping_threads() {
        let _test_guard = panic_hook_test_guard();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let successful_barrier = std::sync::Arc::clone(&barrier);
        let successful = std::thread::spawn(move || {
            super::catch_rust_panic(|| {
                successful_barrier.wait();
                42_i64
            })
        });
        let panicking = std::thread::spawn(move || {
            super::catch_rust_panic(|| {
                barrier.wait();
                panic!("concurrent private payload")
            })
        });

        assert_eq!(successful.join().expect("successful thread joins"), Ok(42));
        let panic_error = panicking
            .join()
            .expect("panicking thread is contained")
            .expect_err("panic maps");
        assert_eq!(panic_error.message(), "Rust bridge panicked");
    }

    #[test]
    fn silent_boundaries_suppress_hooks_on_every_worker_thread() {
        let _test_guard = panic_hook_test_guard();
        let observed = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed_by_hook = std::sync::Arc::clone(&observed);
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |_| {
            observed_by_hook.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }));

        let boundary = std::sync::Arc::new(super::SilentPanicBoundary::enter());
        let workers = (0..8)
            .map(|worker_id| {
                let boundary = std::sync::Arc::clone(&boundary);
                std::thread::spawn(move || {
                    boundary
                        .catch_unwind(|| panic!("private worker payload {worker_id}"))
                        .is_err()
                })
            })
            .collect::<Vec<_>>();
        let worker_results = workers
            .into_iter()
            .map(|worker| worker.join().unwrap_or(false))
            .collect::<Vec<_>>();
        drop(boundary);
        let unprotected = std::panic::catch_unwind(|| panic!("unprotected test panic"));
        let observed_hooks = observed.load(std::sync::atomic::Ordering::SeqCst);
        let _test_hook = std::panic::take_hook();
        std::panic::set_hook(previous_hook);

        assert!(worker_results.into_iter().all(|result| result));
        assert!(unprotected.is_err());
        assert_eq!(observed_hooks, 1);
    }

    #[test]
    fn double_close_keeps_stable_closed_state() {
        let mut handle = Handle::new(1_i64);
        handle.mark_closed(super::__generated_glue::token());
        handle.mark_closed(super::__generated_glue::token());

        assert!(matches!(handle.inner_ref(), Err(HandleStateError::Closed)));
    }

    #[cfg(feature = "net")]
    #[tokio::test(flavor = "current_thread")]
    async fn async_handle_close_and_cancel_join_are_deterministic() {
        async fn close_after_yield(handle: &mut Handle<i64>) {
            tokio::task::yield_now().await;
            handle.mark_closed(super::__generated_glue::token());
        }

        let mut handle = Handle::new(1_i64);
        close_after_yield(&mut handle).await;
        handle.mark_closed(super::__generated_glue::token());
        assert!(matches!(handle.inner_ref(), Err(HandleStateError::Closed)));

        let task = tokio::spawn(async {
            std::future::pending::<()>().await;
        });
        task.abort();
        let join_error = task
            .await
            .expect_err("aborted task should not join successfully");
        assert!(join_error.is_cancelled());
    }
}
