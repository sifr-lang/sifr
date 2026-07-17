use super::{FunctionType, ParamConvention, Type};
use std::fmt::Write as _;

impl Type {
    /// Return the compiler-owned Rust enum name for a non-optional union.
    ///
    /// The encoded identity is deliberately disjoint from every source name in
    /// the ordinary namespace and includes exact emitted nominal identities
    /// rather than display-derived spellings.
    #[must_use]
    pub fn union_enum_name(&self) -> String {
        match self {
            Self::Union(_) => compiler_identifier("__SifrUnion_", &self.union_identity_key()),
            _ => self.rust_type(),
        }
    }

    /// Return an injective compiler-owned variant name for this union member.
    #[must_use]
    pub fn union_variant_name(&self) -> String {
        compiler_identifier("__SifrUnionVariant_", &self.union_identity_key())
    }

    fn union_identity_key(&self) -> String {
        match self {
            Self::Int => atom("int"),
            Self::FixedInt(fixed) => component("fixed", fixed.source_name()),
            Self::Float => atom("float"),
            Self::Bool => atom("bool"),
            Self::Str => atom("str"),
            Self::Bytes => atom("bytes"),
            Self::None => atom("none"),
            Self::Function(function) => function_key("function", function),
            Self::AsyncFunction(function) => function_key("async_function", function),
            Self::Coroutine(ok, error) => binary("coroutine", ok, error),
            Self::Task(ok, error) => binary("task", ok, error),
            Self::TaskResult(ok, error) => binary("task_result", ok, error),
            Self::Failure(error) => unary("failure", error),
            Self::TimeoutResult(error) => unary("timeout_result", error),
            Self::Select2(first, second) => binary("select2", first, second),
            Self::BlockingTask(ok, error) => binary("blocking_task", ok, error),
            Self::JoinSet(ok, error) => binary("join_set", ok, error),
            Self::Awaitable(result) => unary("awaitable", result),
            Self::AsyncIterator(item, error) => binary("async_iterator", item, error),
            Self::AsyncGenerator(item, error) => binary("async_generator", item, error),
            Self::PythonBuffer(element) => unary("python_buffer", element),
            Self::List(element) => unary("list", element),
            Self::Dict(key, value) => binary("dict", key, value),
            Self::Set(element) => unary("set", element),
            Self::Tuple(elements) => sequence("tuple", elements),
            Self::Range => atom("range"),
            Self::Iterable(element) => unary("iterable", element),
            Self::Iterator(element) => unary("iterator", element),
            Self::Any => atom("any"),
            Self::Never => atom("never"),
            Self::Union(members) => sequence("union", members),
            Self::Intersection(members) => sequence("intersection", members),
            Self::LiteralInt(value) => component("literal_int", &value.to_string()),
            Self::LiteralStr(value) => component("literal_str", value),
            Self::LiteralBool(value) => component("literal_bool", if *value { "1" } else { "0" }),
            Self::Alias {
                name,
                type_args,
                body,
            } => {
                let mut key = named_sequence("alias", name, type_args);
                append(&mut key, &body.union_identity_key());
                key
            }
            Self::Unknown => atom("unknown"),
            Self::Result(ok, error) => binary("result", ok, error),
            class @ Self::Class { type_args, .. } => {
                named_sequence("class", &class.rust_type(), type_args)
            }
            Self::Protocol { name, methods } => nominal_methods("protocol", name, methods),
            Self::Newtype { name, inner } => {
                let mut key = component("newtype", name);
                append(&mut key, &inner.union_identity_key());
                key
            }
            Self::TypeVar(name) => component("type_var", name),
            Self::Callable(params, conventions, result) => {
                callable_key("callable", params, conventions, result)
            }
            Self::AsyncCallable(params, conventions, result) => {
                callable_key("async_callable", params, conventions, result)
            }
            Self::Enum { name, variants } => enum_key(name, variants),
            Self::BigInt => atom("bigint"),
            Self::Decimal => atom("decimal"),
            Self::BigDecimal => atom("bigdecimal"),
        }
    }
}

fn atom(tag: &str) -> String {
    component("atom", tag)
}

fn component(tag: &str, value: &str) -> String {
    let mut key = String::new();
    append(&mut key, tag);
    append(&mut key, value);
    key
}

fn append(target: &mut String, value: &str) {
    let _ = write!(target, "{}:{value}", value.len());
}

fn unary(tag: &str, value: &Type) -> String {
    let mut key = component("unary", tag);
    append(&mut key, &value.union_identity_key());
    key
}

fn binary(tag: &str, left: &Type, right: &Type) -> String {
    let mut key = component("binary", tag);
    append(&mut key, &left.union_identity_key());
    append(&mut key, &right.union_identity_key());
    key
}

fn sequence(tag: &str, values: &[Type]) -> String {
    let mut key = component("sequence", tag);
    append(&mut key, &values.len().to_string());
    for value in values {
        append(&mut key, &value.union_identity_key());
    }
    key
}

fn named_sequence(tag: &str, name: &str, values: &[Type]) -> String {
    let mut key = component(tag, name);
    append(&mut key, &values.len().to_string());
    for value in values {
        append(&mut key, &value.union_identity_key());
    }
    key
}

fn function_key(tag: &str, function: &FunctionType) -> String {
    let mut key = component("function_kind", tag);
    append(&mut key, &function.params.len().to_string());
    for (name, parameter, convention) in &function.params {
        append(&mut key, name);
        append(&mut key, &convention_key(*convention));
        append(&mut key, &parameter.union_identity_key());
    }
    append(&mut key, &function.return_type.union_identity_key());
    key
}

fn callable_key(
    tag: &str,
    params: &[Type],
    conventions: &[ParamConvention],
    result: &Type,
) -> String {
    let mut key = component("callable_kind", tag);
    append(&mut key, &params.len().to_string());
    for parameter in params {
        append(&mut key, &parameter.union_identity_key());
    }
    append(&mut key, &conventions.len().to_string());
    for convention in conventions {
        append(&mut key, &convention_key(*convention));
    }
    append(&mut key, &result.union_identity_key());
    key
}

fn nominal_methods(tag: &str, name: &str, methods: &[(String, FunctionType)]) -> String {
    let mut key = component(tag, name);
    append(&mut key, &methods.len().to_string());
    for (method_name, function) in methods {
        append(&mut key, method_name);
        append(&mut key, &function_key("method", function));
    }
    key
}

fn enum_key(name: &str, variants: &[(String, Option<i64>)]) -> String {
    let mut key = component("enum", name);
    append(&mut key, &variants.len().to_string());
    for (variant, value) in variants {
        append(&mut key, variant);
        append(
            &mut key,
            &value.map_or_else(|| "implicit".to_string(), |value| format!("value:{value}")),
        );
    }
    key
}

fn convention_key(convention: ParamConvention) -> String {
    format!(
        "{}{}",
        if convention.is_owned() { 'o' } else { 'b' },
        if convention.is_mutable() { 'm' } else { 'i' }
    )
}

fn compiler_identifier(prefix: &str, identity: &str) -> String {
    let mut name = String::with_capacity(prefix.len() + identity.len() * 2);
    name.push_str(prefix);
    for byte in identity.bytes() {
        let _ = write!(name, "{byte:02x}");
    }
    name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_names_and_variants_are_compiler_owned_and_nominally_injective() {
        let class = |name: &str, identity: Option<&str>| Type::Class {
            identity: identity.map(str::to_string),
            type_args: Vec::new(),
            name: name.to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        };
        let local_int = class("Int", Some("pkg.Int"));
        let canonicalized_local_int = class("Int", None);
        let aliased_other_int = class("OtherInt", Some("other.Int"));
        let union = Type::Union(vec![Type::Int, local_int.clone()]);

        assert!(union.union_enum_name().starts_with("__SifrUnion_"));
        assert_ne!(
            Type::Int.union_variant_name(),
            local_int.union_variant_name()
        );
        assert_ne!(
            local_int.union_variant_name(),
            aliased_other_int.union_variant_name()
        );
        assert_eq!(
            local_int.union_variant_name(),
            canonicalized_local_int.union_variant_name()
        );

        let callable =
            |conventions| Type::Callable(vec![Type::Int], conventions, Box::new(Type::Str));
        assert_ne!(
            callable(vec![ParamConvention::own()]).union_variant_name(),
            callable(vec![ParamConvention::own(), ParamConvention::borrow()]).union_variant_name()
        );
    }
}
