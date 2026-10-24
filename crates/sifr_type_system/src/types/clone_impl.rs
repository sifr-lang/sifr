//! Own recursive type cloning in the type-system crate.
//!
//! This non-generic implementation intentionally has no inline hint: exporting
//! its large recursive body duplicated code generation across compiler crates.
//! Each field keeps its existing Clone semantics, including SharedVec's
//! copy-on-write ownership. Exhaustive patterns require newly added fields.

use super::Type;

impl Clone for Type {
    fn clone(&self) -> Self {
        match self {
            Self::Int => Self::Int,
            Self::FixedInt(field0) => Self::FixedInt(*field0),
            Self::Float => Self::Float,
            Self::Bool => Self::Bool,
            Self::Str => Self::Str,
            Self::Bytes => Self::Bytes,
            Self::None => Self::None,
            Self::Function(field0) => Self::Function(field0.clone()),
            Self::AsyncFunction(field0) => Self::AsyncFunction(field0.clone()),
            Self::Coroutine(field0, field1) => Self::Coroutine(field0.clone(), field1.clone()),
            Self::Task(field0, field1) => Self::Task(field0.clone(), field1.clone()),
            Self::TaskResult(field0, field1) => Self::TaskResult(field0.clone(), field1.clone()),
            Self::Failure(field0) => Self::Failure(field0.clone()),
            Self::TimeoutResult(field0) => Self::TimeoutResult(field0.clone()),
            Self::Select2(field0, field1) => Self::Select2(field0.clone(), field1.clone()),
            Self::BlockingTask(field0, field1) => {
                Self::BlockingTask(field0.clone(), field1.clone())
            }
            Self::JoinSet(field0, field1) => Self::JoinSet(field0.clone(), field1.clone()),
            Self::Awaitable(field0) => Self::Awaitable(field0.clone()),
            Self::AsyncIterator(field0, field1) => {
                Self::AsyncIterator(field0.clone(), field1.clone())
            }
            Self::AsyncGenerator(field0, field1) => {
                Self::AsyncGenerator(field0.clone(), field1.clone())
            }
            Self::PythonBuffer(field0) => Self::PythonBuffer(field0.clone()),
            Self::PythonArrow(field0) => Self::PythonArrow(*field0),
            Self::PythonDlpackTensor(field0) => Self::PythonDlpackTensor(field0.clone()),
            Self::PythonDlpackStream => Self::PythonDlpackStream,
            Self::List(field0) => Self::List(field0.clone()),
            Self::Dict(field0, field1) => Self::Dict(field0.clone(), field1.clone()),
            Self::Set(field0) => Self::Set(field0.clone()),
            Self::Tuple(field0) => Self::Tuple(field0.clone()),
            Self::Template(field0) => Self::Template(field0.clone()),
            Self::StructuralRecord(field0) => Self::StructuralRecord(field0.clone()),
            Self::Range => Self::Range,
            Self::Iterable(field0) => Self::Iterable(field0.clone()),
            Self::Iterator(field0) => Self::Iterator(field0.clone()),
            Self::Any => Self::Any,
            Self::Never => Self::Never,
            Self::Union(field0) => Self::Union(field0.clone()),
            Self::Intersection(field0) => Self::Intersection(field0.clone()),
            Self::LiteralInt(field0) => Self::LiteralInt(*field0),
            Self::LiteralStr(field0) => Self::LiteralStr(field0.clone()),
            Self::LiteralBool(field0) => Self::LiteralBool(*field0),
            Self::Alias {
                name,
                type_args,
                body,
            } => Self::Alias {
                name: name.clone(),
                type_args: type_args.clone(),
                body: body.clone(),
            },
            Self::Unknown => Self::Unknown,
            Self::Result(field0, field1) => Self::Result(field0.clone(), field1.clone()),
            Self::Class {
                identity,
                type_args,
                name,
                fields,
                methods,
                parent_class,
            } => Self::Class {
                identity: identity.clone(),
                type_args: type_args.clone(),
                name: name.clone(),
                fields: fields.clone(),
                methods: methods.clone(),
                parent_class: parent_class.clone(),
            },
            Self::Protocol {
                identity,
                name,
                methods,
            } => Self::Protocol {
                identity: identity.clone(),
                name: name.clone(),
                methods: methods.clone(),
            },
            Self::Newtype {
                identity,
                name,
                inner,
            } => Self::Newtype {
                identity: identity.clone(),
                name: name.clone(),
                inner: inner.clone(),
            },
            Self::TypeVar(field0) => Self::TypeVar(field0.clone()),
            Self::Callable(field0, field1, field2) => {
                Self::Callable(field0.clone(), field1.clone(), field2.clone())
            }
            Self::AsyncCallable(field0, field1, field2) => {
                Self::AsyncCallable(field0.clone(), field1.clone(), field2.clone())
            }
            Self::Enum {
                identity,
                name,
                variants,
            } => Self::Enum {
                identity: identity.clone(),
                name: name.clone(),
                variants: variants.clone(),
            },
            Self::Decimal => Self::Decimal,
            Self::BigDecimal => Self::BigDecimal,
        }
    }
}
