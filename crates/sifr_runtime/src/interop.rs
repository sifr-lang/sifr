//! Stable Rust interop helper types used by generated bridge contracts.

use crate::{IntegerParseError, IntegerRangeError, SifrInt};
use std::any::Any;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Mutex;

// `catch_unwind` still invokes the process-wide panic hook, so bridge panic
// redaction serializes hook swaps while the recoverable boundary is active.
static RUST_INTEROP_PANIC_HOOK_LOCK: Mutex<()> = Mutex::new(());

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
    let _hook_guard = match RUST_INTEROP_PANIC_HOOK_LOCK.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    std::panic::set_hook(previous_hook);
    result.map_err(|payload| RustPanicErrorBridge::from_panic_payload(payload.as_ref()))
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
    fn handle_poison_guard_marks_open_handle_when_rust_call_unwinds() {
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
        let error =
            super::catch_rust_panic(|| panic!("secret backend token")).expect_err("panic maps");

        assert_eq!(error.message(), "Rust bridge panicked");
        assert!(!error.message().contains("secret"));
    }

    #[test]
    fn catch_rust_panic_preserves_successful_values() {
        assert_eq!(super::catch_rust_panic(|| 42_i64), Ok(42));
    }

    #[test]
    fn double_close_keeps_stable_closed_state() {
        let mut handle = Handle::new(1_i64);
        handle.mark_closed(super::__generated_glue::token());
        handle.mark_closed(super::__generated_glue::token());

        assert!(matches!(handle.inner_ref(), Err(HandleStateError::Closed)));
    }
}
