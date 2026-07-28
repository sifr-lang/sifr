//! Rust interop declaration metadata carried through HIR.

use ruff_text_size::TextRange;

/// A structured Rust target path such as `bridge.hash.digest` or `Self.poll`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustTargetPath {
    pub segments: Vec<String>,
    pub span: TextRange,
}

impl RustTargetPath {
    #[must_use]
    pub fn dotted(&self) -> String {
        self.segments.join(".")
    }
}

/// Rust interop decorator forms accepted by the declaration grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustInteropDecoratorKind {
    Function,
    Opaque,
    Async,
    Callback,
    ZeroCopy,
    View,
}

/// Static effect classification attached to a Rust interop declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustInteropEffect {
    Sync,
    Async,
    BlockingIo,
    CpuHeavy,
}

/// ABI obligations that later Rust bridge probing and glue generation must satisfy.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RustInteropAbiRequirements {
    pub async_boundary: bool,
    pub opaque_handle: bool,
    pub zero_copy: bool,
    pub view: bool,
}

/// A parsed decorator argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustInteropArgument {
    pub name: Option<String>,
    pub value: RustInteropValue,
    pub span: TextRange,
}

/// Small, static Rust interop decorator value grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustInteropValue {
    Boolean(bool),
    Symbol(String),
    Integer(i64),
    IntegerList(Vec<i64>),
    PolicyCall {
        name: String,
        argument: Box<RustInteropValue>,
        span: TextRange,
    },
    TargetPath(RustTargetPath),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RustCallbackBackpressure {
    Direct,
    Bounded(i64),
    Unbounded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RustCallbackOverflow {
    Error,
    DropOldest,
    DropNewest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RustCallbackShutdown {
    Drain,
    Cancel,
    DetachForbidden,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RustThreadsafeCallbackContract {
    pub backpressure: RustCallbackBackpressure,
    pub overflow: RustCallbackOverflow,
    pub shutdown: RustCallbackShutdown,
}

pub fn rust_threadsafe_callback_contract(
    declaration: &RustInteropDeclaration,
) -> Result<RustThreadsafeCallbackContract, String> {
    let mut backpressure = None;
    let mut overflow = None;
    let mut shutdown = None;

    for argument in &declaration.arguments {
        let Some(name) = argument.name.as_deref() else {
            return Err("`@rust.callback(...)` requires named arguments".to_string());
        };
        match name {
            "backpressure" => {
                if backpressure.is_some() {
                    return Err("duplicate `backpressure=` policy".to_string());
                }
                backpressure = Some(parse_callback_backpressure(&argument.value)?);
            }
            "overflow" => {
                if overflow.is_some() {
                    return Err("duplicate `overflow=` policy".to_string());
                }
                overflow = Some(parse_callback_overflow(&argument.value)?);
            }
            "shutdown" => {
                if shutdown.is_some() {
                    return Err("duplicate `shutdown=` policy".to_string());
                }
                shutdown = Some(parse_callback_shutdown(&argument.value)?);
            }
            other => return Err(format!("unsupported `@rust.callback(...)` key `{other}`")),
        }
    }

    Ok(RustThreadsafeCallbackContract {
        backpressure: backpressure
            .ok_or_else(|| "missing required `backpressure=` policy".to_string())?,
        overflow: overflow.ok_or_else(|| "missing required `overflow=` policy".to_string())?,
        shutdown: shutdown.ok_or_else(|| "missing required `shutdown=` policy".to_string())?,
    })
}

fn parse_callback_backpressure(
    value: &RustInteropValue,
) -> Result<RustCallbackBackpressure, String> {
    match value {
        RustInteropValue::Symbol(symbol) if symbol == "direct" => {
            Ok(RustCallbackBackpressure::Direct)
        }
        RustInteropValue::Symbol(symbol) if symbol == "unbounded" => {
            Ok(RustCallbackBackpressure::Unbounded)
        }
        RustInteropValue::PolicyCall { name, argument, .. } if name == "bounded" => {
            let RustInteropValue::Integer(bound) = argument.as_ref() else {
                return Err("`backpressure=bounded(...)` requires an integer bound".to_string());
            };
            if *bound <= 0 {
                return Err("`backpressure=bounded(...)` requires a positive bound".to_string());
            }
            Ok(RustCallbackBackpressure::Bounded(*bound))
        }
        _ => Err("`backpressure=` must be direct, unbounded, or bounded(N)".to_string()),
    }
}

fn parse_callback_overflow(value: &RustInteropValue) -> Result<RustCallbackOverflow, String> {
    let RustInteropValue::Symbol(symbol) = value else {
        return Err("`overflow=` must be error, drop_oldest, or drop_newest".to_string());
    };
    match symbol.as_str() {
        "error" => Ok(RustCallbackOverflow::Error),
        "drop_oldest" => Ok(RustCallbackOverflow::DropOldest),
        "drop_newest" => Ok(RustCallbackOverflow::DropNewest),
        _ => Err("`overflow=` must be error, drop_oldest, or drop_newest".to_string()),
    }
}

fn parse_callback_shutdown(value: &RustInteropValue) -> Result<RustCallbackShutdown, String> {
    let RustInteropValue::Symbol(symbol) = value else {
        return Err("`shutdown=` must be drain, cancel, or detach_forbidden".to_string());
    };
    match symbol.as_str() {
        "drain" => Ok(RustCallbackShutdown::Drain),
        "cancel" => Ok(RustCallbackShutdown::Cancel),
        "detach_forbidden" => Ok(RustCallbackShutdown::DetachForbidden),
        _ => Err("`shutdown=` must be drain, cancel, or detach_forbidden".to_string()),
    }
}

/// A Rust interop declaration associated with a HIR function or class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustInteropDeclaration {
    pub kind: RustInteropDecoratorKind,
    pub target: Option<RustTargetPath>,
    pub arguments: Vec<RustInteropArgument>,
    pub span: TextRange,
    pub effect: RustInteropEffect,
    pub abi_requirements: RustInteropAbiRequirements,
    /// Whether an opaque instance method's source receiver is declared `own self`.
    pub consumes_receiver: bool,
}

/// Return the canonical member selected by an opaque Rust close policy.
#[must_use]
pub fn rust_opaque_close_method(declarations: &[RustInteropDeclaration]) -> Option<&'static str> {
    let opaque = declarations
        .iter()
        .find(|declaration| declaration.kind == RustInteropDecoratorKind::Opaque)?;
    opaque.arguments.iter().find_map(|argument| {
        if argument.name.as_deref() != Some("close") {
            return None;
        }
        match &argument.value {
            RustInteropValue::Symbol(policy) if policy == "close" => Some("close"),
            RustInteropValue::Symbol(policy) if policy == "async_close" => Some("aclose"),
            _ => None,
        }
    })
}
