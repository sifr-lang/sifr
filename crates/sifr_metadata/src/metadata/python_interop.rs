//! Explicit DX.5 wire records. IDs replace live compiler ownership and recursion.
use super::{ParamConvention, PythonArrowKind, Ref, Text, TextRange, Type};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PythonTargetPath {
    pub segments: Vec<Ref<Text>>,
    pub span: Ref<TextRange>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PythonBufferAccess {
    Read,
    Write,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PythonBufferLayout {
    Any,
    CContiguous,
    FContiguous,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PythonBufferDeclaration {
    pub element_type: Ref<Type>,
    pub access: Ref<PythonBufferAccess>,
    pub layout: Ref<PythonBufferLayout>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PythonArrowSchemaMode {
    Omitted,
    Parameter {
        name: Ref<Text>,
        span: Ref<TextRange>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PythonArrowDeclaration {
    pub kind: Ref<PythonArrowKind>,
    pub schema: Ref<PythonArrowSchemaMode>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PythonDlpackDevice {
    Cpu,
    Cuda,
    Any,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PythonDlpackStreamMode {
    None,
    Parameter {
        name: Ref<Text>,
        span: Ref<TextRange>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PythonDlpackDeclaration {
    pub device: Ref<PythonDlpackDevice>,
    pub stream: Ref<PythonDlpackStreamMode>,
    pub element_type: Option<Ref<Type>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PythonInteropEffect {
    BlockingIo,
    Async,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PythonCleanupPolicy {
    Drop,
    Close,
    AsyncClose,
    Context,
    AsyncContext,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PythonParameterKind {
    Positional,
    KeywordOnly,
    PositionalVariadic,
    KeywordVariadic,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PythonCallbackLifetime {
    Call,
    Result,
    Receiver,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PythonCallbackDispatch {
    Current,
    Foreign,
    Asyncio,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PythonCallbackConcurrency {
    Serial,
    Parallel,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PythonCallbackDeclaration {
    pub parameter_name: Ref<Text>,
    pub span: Ref<TextRange>,
    pub lifetime: Ref<PythonCallbackLifetime>,
    pub dispatch: Ref<PythonCallbackDispatch>,
    pub concurrency: Option<Ref<PythonCallbackConcurrency>>,
    pub argument_types: Vec<Ref<Type>>,
    pub argument_conventions: Vec<Ref<ParamConvention>>,
    pub success_type: Ref<Type>,
    pub handler_error_type: Option<Ref<Type>>,
    pub is_async: bool,
    pub owner_class: Option<Ref<Text>>,
    pub owner_cleanup: Option<Ref<PythonCleanupPolicy>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PythonInteropParameter {
    pub name: Ref<Text>,
    pub kind: Ref<PythonParameterKind>,
    pub has_default: bool,
    pub omit_when_absent: bool,
    pub span: Ref<TextRange>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PythonInteropDeclaration {
    pub kind: Ref<PythonInteropDecoratorKind>,
    pub target: Option<Ref<PythonTargetPath>>,
    pub span: Ref<TextRange>,
    pub effect: Ref<PythonInteropEffect>,
    pub cleanup: Option<Ref<PythonCleanupPolicy>>,
    pub consumes_receiver: bool,
    pub parameters: Vec<Ref<PythonInteropParameter>>,
    pub required_import_root: Option<Ref<Text>>,
    pub callbacks: Vec<Ref<PythonCallbackDeclaration>>,
    pub buffer: Option<Ref<PythonBufferDeclaration>>,
    pub arrow: Option<Ref<PythonArrowDeclaration>>,
    pub dlpack: Option<Ref<PythonDlpackDeclaration>>,
}
