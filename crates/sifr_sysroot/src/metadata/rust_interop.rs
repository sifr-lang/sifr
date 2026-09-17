//! Explicit DX.5 wire records. IDs replace live compiler ownership and recursion.
use super::{Ref, Text, TextRange};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RustTargetPath {
    pub segments: Vec<Ref<Text>>,
    pub span: Ref<TextRange>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RustInteropDecoratorKind {
    Function,
    Opaque,
    Async,
    Callback,
    Structural,
    ZeroCopy,
    View,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RustInteropEffect {
    Sync,
    Async,
    BlockingIo,
    CpuHeavy,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RustInteropAbiRequirements {
    pub async_boundary: bool,
    pub opaque_handle: bool,
    pub zero_copy: bool,
    pub view: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RustInteropArgument {
    pub name: Option<Ref<Text>>,
    pub value: Ref<RustInteropValue>,
    pub span: Ref<TextRange>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RustInteropValue {
    Boolean(bool),
    Symbol(Ref<Text>),
    Integer(i64),
    IntegerList(Vec<i64>),
    PolicyCall {
        name: Ref<Text>,
        argument: Ref<RustInteropValue>,
        span: Ref<TextRange>,
    },
    TargetPath(Ref<RustTargetPath>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RustCallbackBackpressure {
    Direct,
    Bounded(i64),
    Unbounded,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RustCallbackOverflow {
    Error,
    DropOldest,
    DropNewest,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RustCallbackShutdown {
    Drain,
    Cancel,
    DetachForbidden,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RustThreadsafeCallbackContract {
    pub backpressure: Ref<RustCallbackBackpressure>,
    pub overflow: Ref<RustCallbackOverflow>,
    pub shutdown: Ref<RustCallbackShutdown>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RustInteropDeclaration {
    pub kind: Ref<RustInteropDecoratorKind>,
    pub target: Option<Ref<RustTargetPath>>,
    pub arguments: Vec<Ref<RustInteropArgument>>,
    pub span: Ref<TextRange>,
    pub effect: Ref<RustInteropEffect>,
    pub abi_requirements: Ref<RustInteropAbiRequirements>,
    pub consumes_receiver: bool,
}
