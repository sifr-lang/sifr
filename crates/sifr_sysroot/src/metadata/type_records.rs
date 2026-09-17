//! Explicit DX.5 wire records. IDs replace live compiler ownership and recursion.
use super::{Binder, Declaration, NominalView, Ref, Text};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralRecordField {
    pub name: Ref<Text>,
    pub ty: Ref<Type>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralRecordType {
    pub canonical_fields: Vec<Ref<StructuralRecordField>>,
    pub source_order: Vec<Ref<Text>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PythonArrowKind {
    Array,
    Schema,
    Stream,
    DeviceArray,
    DeviceStream,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Type {
    Int,
    FixedInt(Ref<FixedIntType>),
    Float,
    Bool,
    Str,
    Bytes,
    None,
    Function(Ref<FunctionType>),
    AsyncFunction(Ref<FunctionType>),
    Coroutine(Ref<Type>, Ref<Type>),
    Task(Ref<Type>, Ref<Type>),
    TaskResult(Ref<Type>, Ref<Type>),
    Failure(Ref<Type>),
    TimeoutResult(Ref<Type>),
    Select2(Ref<Type>, Ref<Type>),
    BlockingTask(Ref<Type>, Ref<Type>),
    JoinSet(Ref<Type>, Ref<Type>),
    Awaitable(Ref<Type>),
    AsyncIterator(Ref<Type>, Ref<Type>),
    AsyncGenerator(Ref<Type>, Ref<Type>),
    PythonBuffer(Ref<Type>),
    PythonArrow(Ref<PythonArrowKind>),
    PythonDlpackTensor(Ref<Type>),
    PythonDlpackStream,
    List(Ref<Type>),
    Dict(Ref<Type>, Ref<Type>),
    Set(Ref<Type>),
    Tuple(Vec<Ref<Type>>),
    Template(Vec<Ref<Type>>),
    StructuralRecord(Ref<StructuralRecordType>),
    Range,
    Iterable(Ref<Type>),
    Iterator(Ref<Type>),
    Any,
    Never,
    Union(Vec<Ref<Type>>),
    Intersection(Vec<Ref<Type>>),
    LiteralInt(i64),
    LiteralStr(Ref<Text>),
    LiteralBool(bool),
    Alias {
        name: Ref<Text>,
        declaration: Ref<Declaration>,
        type_args: Vec<Ref<Type>>,
        body: Option<Ref<Type>>,
    },
    Unknown,
    Result(Ref<Type>, Ref<Type>),
    Class {
        declaration: Ref<Declaration>,
        view: Ref<NominalView>,
        type_args: Vec<Ref<Type>>,
    },
    Protocol {
        declaration: Ref<Declaration>,
        view: Ref<NominalView>,
        type_args: Vec<Ref<Type>>,
    },
    Newtype {
        declaration: Ref<Declaration>,
        view: Ref<NominalView>,
        type_args: Vec<Ref<Type>>,
    },
    TypeVar {
        binder: Ref<Binder>,
        slot: u32,
    },
    Callable(Vec<Ref<Type>>, Vec<Ref<ParamConvention>>, Ref<Type>),
    AsyncCallable(Vec<Ref<Type>>, Vec<Ref<ParamConvention>>, Ref<Type>),
    Enum {
        declaration: Ref<Declaration>,
        view: Ref<NominalView>,
        type_args: Vec<Ref<Type>>,
    },
    Decimal,
    BigDecimal,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FixedIntType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    ISize,
    USize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionType {
    pub receiver: Option<Ref<ReceiverConvention>>,
    pub params: Vec<(Ref<Text>, Ref<Type>, Ref<ParamConvention>)>,
    pub return_type: Ref<Type>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ReceiverConvention {
    SharedBorrow,
    MutableBorrow,
    Owned,
    OwnedMutable,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ParamOwnership {
    Borrow,
    Own,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ParamMutability {
    Immutable,
    Mutable,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParamConvention {
    pub ownership: Ref<ParamOwnership>,
    pub mutability: Ref<ParamMutability>,
}
