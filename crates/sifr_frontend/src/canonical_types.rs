//! Declared canonical encodings for checked type and callable identities.

use sifr_type_system::{
    FunctionType, ParamConvention, ParamMutability, ParamOwnership, ReceiverConvention, Type,
};

pub(crate) fn function_identity(function: &FunctionType) -> String {
    let receiver = match function.receiver {
        None => "none",
        Some(ReceiverConvention::SharedBorrow) => "shared",
        Some(ReceiverConvention::MutableBorrow) => "mutable",
        Some(ReceiverConvention::Owned) => "owned",
        Some(ReceiverConvention::OwnedMutable) => "owned-mutable",
    };
    format!(
        "fn[receiver={receiver};params={};return={}]",
        sequence(function.params.iter().map(|(name, ty, convention)| {
            format!(
                "{}:{}:{}",
                atom(name),
                convention_identity(*convention),
                atom(&type_identity(ty))
            )
        })),
        atom(&type_identity(&function.return_type)),
    )
}

pub(crate) fn type_identity(ty: &Type) -> String {
    match ty {
        Type::Int => "int".to_string(),
        Type::FixedInt(kind) => format!("fixed:{}", kind.source_name()),
        Type::Float => "float".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Str => "str".to_string(),
        Type::Bytes => "bytes".to_string(),
        Type::None => "none".to_string(),
        Type::Function(function) => format!("function:{}", atom(&function_identity(function))),
        Type::AsyncFunction(function) => {
            format!("async-function:{}", atom(&function_identity(function)))
        }
        Type::Coroutine(value, error) => pair("coroutine", value, error),
        Type::Task(value, error) => pair("task", value, error),
        Type::TaskResult(value, error) => pair("task-result", value, error),
        Type::Failure(error) => unary("failure", error),
        Type::TimeoutResult(value) => unary("timeout-result", value),
        Type::Select2(left, right) => pair("select2", left, right),
        Type::BlockingTask(value, error) => pair("blocking-task", value, error),
        Type::JoinSet(value, error) => pair("join-set", value, error),
        Type::Awaitable(value) => unary("awaitable", value),
        Type::AsyncIterator(value, error) => pair("async-iterator", value, error),
        Type::AsyncGenerator(value, error) => pair("async-generator", value, error),
        Type::PythonBuffer(value) => unary("python-buffer", value),
        Type::PythonArrow(kind) => format!("python-arrow:{}", kind.source_name()),
        Type::PythonDlpackTensor(value) => unary("python-dlpack-tensor", value),
        Type::PythonDlpackStream => "python-dlpack-stream".to_string(),
        Type::List(item) => unary("list", item),
        Type::Dict(key, value) => pair("dict", key, value),
        Type::Set(item) => unary("set", item),
        Type::Tuple(items) => format!("tuple[{}]", types(items)),
        Type::Range => "range".to_string(),
        Type::Iterable(item) => unary("iterable", item),
        Type::Iterator(item) => unary("iterator", item),
        Type::Any => "any".to_string(),
        Type::Never => "never".to_string(),
        Type::Union(items) => format!("union[{}]", types(items)),
        Type::Intersection(items) => format!("intersection[{}]", types(items)),
        Type::LiteralInt(value) => format!("literal-int:{value}"),
        Type::LiteralStr(value) => format!("literal-str:{}", atom(value)),
        Type::LiteralBool(value) => format!("literal-bool:{value}"),
        Type::Alias {
            name,
            type_args,
            body,
        } => format!(
            "alias[name={};args={};body={}]",
            atom(name),
            types(type_args),
            atom(&type_identity(body)),
        ),
        Type::Unknown => "unknown".to_string(),
        Type::Result(value, error) => pair("result", value, error),
        Type::Class {
            identity,
            name,
            type_args,
            ..
        } => nominal("class", identity.as_deref().unwrap_or(name), type_args),
        Type::Protocol { identity, name, .. } => {
            nominal("protocol", identity.as_deref().unwrap_or(name), &[])
        }
        Type::Newtype {
            identity,
            name,
            inner,
        } => format!(
            "newtype[name={};inner={}]",
            atom(identity.as_deref().unwrap_or(name)),
            atom(&type_identity(inner)),
        ),
        Type::TypeVar(name) => format!("type-var:{}", atom(name)),
        Type::Callable(params, conventions, result) => {
            callable("callable", params, conventions, result)
        }
        Type::AsyncCallable(params, conventions, result) => {
            callable("async-callable", params, conventions, result)
        }
        Type::Enum { identity, name, .. } => {
            nominal("enum", identity.as_deref().unwrap_or(name), &[])
        }
        Type::Decimal => "decimal".to_string(),
        Type::BigDecimal => "big-decimal".to_string(),
    }
}

fn nominal(kind: &str, identity: &str, arguments: &[Type]) -> String {
    format!(
        "{kind}[identity={};args={}]",
        atom(identity),
        types(arguments)
    )
}

fn callable(kind: &str, params: &[Type], conventions: &[ParamConvention], result: &Type) -> String {
    format!(
        "{kind}[params={};conventions={};return={}]",
        types(params),
        sequence(conventions.iter().map(|item| convention_identity(*item))),
        atom(&type_identity(result)),
    )
}

fn unary(kind: &str, value: &Type) -> String {
    format!("{kind}[{}]", atom(&type_identity(value)))
}

fn pair(kind: &str, left: &Type, right: &Type) -> String {
    format!(
        "{kind}[{},{}]",
        atom(&type_identity(left)),
        atom(&type_identity(right))
    )
}

fn types(items: &[Type]) -> String {
    sequence(items.iter().map(type_identity))
}

fn convention_identity(convention: ParamConvention) -> String {
    let ownership = match convention.ownership() {
        ParamOwnership::Borrow => "borrow",
        ParamOwnership::Own => "own",
    };
    let mutability = match convention.mutability() {
        ParamMutability::Immutable => "immutable",
        ParamMutability::Mutable => "mutable",
    };
    format!("{ownership}:{mutability}")
}

fn sequence(items: impl IntoIterator<Item = String>) -> String {
    items
        .into_iter()
        .map(|item| atom(&item))
        .collect::<Vec<_>>()
        .join(",")
}

fn atom(value: &str) -> String {
    format!("{}:{value}", value.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_function_identity_is_nominal_and_convention_sensitive() {
        let owned = FunctionType {
            receiver: None,
            params: vec![(
                "value".to_string(),
                Type::Class {
                    identity: Some("fixture.Model".to_string()),
                    type_args: vec![Type::Int],
                    name: "Alias".to_string(),
                    fields: vec![("ignored".to_string(), Type::Str)],
                    methods: Vec::new(),
                    parent_class: None,
                },
                ParamConvention::own(),
            )],
            return_type: Box::new(Type::Bool),
        };
        let mut borrowed = owned.clone();
        borrowed.params[0].2 = ParamConvention::borrow();
        assert!(function_identity(&owned).contains("fixture.Model"));
        assert!(!function_identity(&owned).contains("ignored"));
        assert_ne!(function_identity(&owned), function_identity(&borrowed));
    }
}
