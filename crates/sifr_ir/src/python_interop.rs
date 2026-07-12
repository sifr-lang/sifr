//! Declaration-first Python interop metadata carried through HIR.

use ruff_text_size::TextRange;

/// A Python declaration target written as a structured dotted path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonTargetPath {
    pub segments: Vec<String>,
    pub span: TextRange,
}

impl PythonTargetPath {
    #[must_use]
    pub fn dotted(&self) -> String {
        self.segments.join(".")
    }

    #[must_use]
    pub fn root(&self) -> Option<&str> {
        self.segments.first().map(String::as_str)
    }
}

/// Declaration forms recognized in the dedicated Python decorator namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonInteropDecoratorKind {
    Function,
    Coroutine,
    Opaque,
    Attribute,
    Item,
    ContextEnter,
    ContextExit,
    ContextAsyncEnter,
    ContextAsyncExit,
    Callback,
    Buffer,
    Arrow,
    Dlpack,
}

/// Compiler-synthesized execution effect for a Python declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonInteropEffect {
    BlockingIo,
    Async,
}

/// Semantic cleanup obligation attached to an opaque Python class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonCleanupPolicy {
    Drop,
    Close,
    AsyncClose,
    Context,
    AsyncContext,
}

/// How a declared Sifr parameter contributes to the Python call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonParameterKind {
    Positional,
    KeywordOnly,
    PositionalVariadic,
    KeywordVariadic,
}

/// Call-shape metadata retained independently of the ordinary function type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonInteropParameter {
    pub name: String,
    pub kind: PythonParameterKind,
    pub has_default: bool,
    pub omit_when_absent: bool,
    pub span: TextRange,
}

/// A declaration-first Python binding attached to a HIR function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonInteropDeclaration {
    pub kind: PythonInteropDecoratorKind,
    pub target: Option<PythonTargetPath>,
    pub span: TextRange,
    pub effect: PythonInteropEffect,
    pub cleanup: Option<PythonCleanupPolicy>,
    /// Whether an opaque instance method consumes its receiver (`own self`).
    pub consumes_receiver: bool,
    pub parameters: Vec<PythonInteropParameter>,
    /// The non-reserved import root contributed by this declaration.
    pub required_import_root: Option<String>,
}
