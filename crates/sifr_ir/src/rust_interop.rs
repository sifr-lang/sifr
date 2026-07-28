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
