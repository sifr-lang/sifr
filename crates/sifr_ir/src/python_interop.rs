//! Declaration-first Python interop metadata carried through HIR.

use ruff_text_size::TextRange;
use sifr_type_system::{ParamConvention, PythonArrowKind, Type};

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
    DlpackStream,
}

/// Access authority requested from a Python buffer exporter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonBufferAccess {
    Read,
    Write,
}

/// Physical layout required by a Python buffer declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonBufferLayout {
    Any,
    CContiguous,
    FContiguous,
}

/// Typed protocol facts carried by an active `@python.buffer` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonBufferDeclaration {
    pub element_type: Type,
    pub access: PythonBufferAccess,
    pub layout: PythonBufferLayout,
}

/// Requested-schema policy for an Arrow C Data Interface declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PythonArrowSchemaMode {
    Omitted,
    Parameter { name: String, span: TextRange },
}

/// Typed protocol facts carried by an active `@python.arrow` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonArrowDeclaration {
    pub kind: PythonArrowKind,
    pub schema: PythonArrowSchemaMode,
}

/// Device family accepted by a declaration-first DLPack boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonDlpackDevice {
    Cpu,
    Cuda,
    Any,
}

impl PythonDlpackDevice {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Cuda => "cuda",
            Self::Any => "any",
        }
    }
}

/// Explicit synchronization stream policy for DLPack acquisition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PythonDlpackStreamMode {
    None,
    Parameter { name: String, span: TextRange },
}

/// Typed protocol facts carried by an active DLPack declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonDlpackDeclaration {
    pub device: PythonDlpackDevice,
    pub stream: PythonDlpackStreamMode,
    pub element_type: Option<Type>,
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

/// Dynamic scope that owns a generated Python callback trampoline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonCallbackLifetime {
    Call,
    Result,
    Receiver,
}

/// Executor/thread authority used to invoke a declared callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonCallbackDispatch {
    Current,
    Foreign,
    Asyncio,
}

/// Admission policy for callback invocations that may overlap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonCallbackConcurrency {
    Serial,
    Parallel,
}

/// Typed callback adjunct attached to one Python implementation declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonCallbackDeclaration {
    pub parameter_name: String,
    pub span: TextRange,
    pub lifetime: PythonCallbackLifetime,
    pub dispatch: PythonCallbackDispatch,
    pub concurrency: Option<PythonCallbackConcurrency>,
    pub argument_types: Vec<Type>,
    pub argument_conventions: Vec<ParamConvention>,
    pub success_type: Type,
    pub handler_error_type: Option<Type>,
    pub is_async: bool,
    pub owner_class: Option<String>,
    pub owner_cleanup: Option<PythonCleanupPolicy>,
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
    /// Callback adjuncts validated against this implementation declaration.
    pub callbacks: Vec<PythonCallbackDeclaration>,
    /// Buffer protocol contract, present only for `PythonInteropDecoratorKind::Buffer`.
    pub buffer: Option<PythonBufferDeclaration>,
    /// Arrow protocol contract, present only for `PythonInteropDecoratorKind::Arrow`.
    pub arrow: Option<PythonArrowDeclaration>,
    /// DLPack contract, present only for tensor and stream declarations.
    pub dlpack: Option<PythonDlpackDeclaration>,
}
