use super::{FunctionType, ParamConvention, Type};

impl Type {
    /// Return the compiler-owned Rust enum name for a non-optional union.
    ///
    /// The encoded identity is deliberately disjoint from every source name in
    /// the ordinary namespace and includes exact emitted nominal identities
    /// rather than display-derived spellings.
    #[must_use]
    pub fn union_enum_name(&self) -> String {
        match self {
            Self::Union(members) => match crate::make_union(members.clone()) {
                canonical @ Self::Union(_) => {
                    compiler_identifier("__SifrUnion_", &canonical.union_identity_key())
                }
                collapsed => collapsed.rust_type(),
            },
            _ => self.rust_type(),
        }
    }

    /// Return an injective compiler-owned variant name for this union member.
    #[must_use]
    pub fn union_variant_name(&self) -> String {
        compiler_identifier("__SifrUnionVariant_", &self.union_identity_key())
    }

    pub(crate) fn union_identity_key(&self) -> String {
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
            Self::PythonArrow(kind) => component("python_arrow", kind.source_name()),
            Self::PythonDlpackTensor(element) => unary("python_dlpack_tensor", element),
            Self::PythonDlpackStream => atom("python_dlpack_stream"),
            Self::List(element) => unary("list", element),
            Self::Dict(key, value) => binary("dict", key, value),
            Self::Set(element) => unary("set", element),
            Self::Tuple(elements) => sequence("tuple", elements),
            Self::Range => atom("range"),
            Self::Iterable(element) => unary("iterable", element),
            Self::Iterator(element) => unary("iterator", element),
            Self::Any => atom("any"),
            Self::Never => atom("never"),
            Self::Union(members) => union_sequence("union", members),
            Self::Intersection(members) => sequence("intersection", members),
            Self::LiteralInt(value) => component("literal_int", &value.to_string()),
            Self::LiteralStr(value) => component("literal_str", value),
            Self::LiteralBool(value) => component("literal_bool", if *value { "1" } else { "0" }),
            Self::Alias {
                name,
                type_args,
                body,
            } if matches!(body.as_ref(), Self::Unknown) => {
                named_sequence("recursive_alias", name, type_args)
            }
            Self::Alias { body, .. } => body.union_identity_key(),
            Self::Unknown => atom("unknown"),
            Self::Result(ok, error) => binary("result", ok, error),
            Self::Class {
                identity,
                name,
                type_args,
                ..
            } => named_sequence("class", identity.as_deref().unwrap_or(name), type_args),
            Self::Protocol {
                identity,
                name,
                methods,
            } => nominal_methods("protocol", identity.as_deref().unwrap_or(name), methods),
            Self::Newtype {
                identity,
                name,
                inner,
            } => {
                let mut key = component("newtype", identity.as_deref().unwrap_or(name));
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
            Self::Enum {
                identity,
                name,
                variants,
            } => enum_key(identity.as_deref().unwrap_or(name), variants),
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
    target.push_str(&value.len().to_string());
    target.push(':');
    target.push_str(value);
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

fn union_sequence(tag: &str, values: &[Type]) -> String {
    let mut identities = Vec::new();
    collect_union_identities(values, &mut identities);
    identities.sort_unstable();
    identities.dedup();
    if identities.len() == 1 {
        if let Some(identity) = identities.pop() {
            return identity;
        }
    }
    let mut key = component("sequence", tag);
    append(&mut key, &identities.len().to_string());
    for identity in identities {
        append(&mut key, &identity);
    }
    key
}

fn collect_union_identities(values: &[Type], identities: &mut Vec<String>) {
    for value in values {
        match value {
            Type::Union(nested) => collect_union_identities(nested, identities),
            Type::Alias { body, .. } if matches!(body.as_ref(), Type::Union(_)) => {
                collect_union_identities(std::slice::from_ref(body.as_ref()), identities);
            }
            other => identities.push(other.union_identity_key()),
        }
    }
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
    append(
        &mut key,
        match function.receiver {
            None => "receiver:none",
            Some(super::ReceiverConvention::SharedBorrow) => "receiver:shared",
            Some(super::ReceiverConvention::MutableBorrow) => "receiver:mutable",
            Some(super::ReceiverConvention::Owned) => "receiver:owned",
        },
    );
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
    super::source_names::compiler_owned_identifier(prefix, identity)
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
        let unqualified_local_int = class("Int", None);
        let aliased_local_int = class("Integer", Some("pkg.Int"));
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
        assert_ne!(
            local_int.union_variant_name(),
            unqualified_local_int.union_variant_name()
        );
        assert_eq!(
            local_int.union_variant_name(),
            aliased_local_int.union_variant_name()
        );

        let callable =
            |conventions| Type::Callable(vec![Type::Int], conventions, Box::new(Type::Str));
        assert_ne!(
            callable(vec![ParamConvention::own()]).union_variant_name(),
            callable(vec![ParamConvention::own(), ParamConvention::borrow()]).union_variant_name()
        );
        assert_eq!(
            Type::Union(vec![Type::Int, Type::Str]).union_enum_name(),
            Type::Union(vec![Type::Str, Type::Int]).union_enum_name()
        );

        let newtype = |name: &str, identity: &str| Type::Newtype {
            identity: Some(identity.to_string()),
            name: name.to_string(),
            inner: Box::new(Type::Int),
        };
        let enumeration = |name: &str, identity: &str| Type::Enum {
            identity: Some(identity.to_string()),
            name: name.to_string(),
            variants: vec![("READY".to_string(), Some(1))],
        };
        let protocol = |name: &str, identity: &str| Type::Protocol {
            identity: Some(identity.to_string()),
            name: name.to_string(),
            methods: Vec::new(),
        };
        for (left, right) in [
            (
                newtype("Token", "left.Token"),
                newtype("Token", "right.Token"),
            ),
            (
                enumeration("Status", "left.Status"),
                enumeration("Status", "right.Status"),
            ),
            (
                protocol("Readable", "left.Readable"),
                protocol("Readable", "right.Readable"),
            ),
        ] {
            assert_ne!(
                Type::Union(vec![Type::Int, left]).union_enum_name(),
                Type::Union(vec![Type::Int, right]).union_enum_name()
            );
        }
        for (original, alias) in [
            (
                newtype("Token", "left.Token"),
                newtype("PublicToken", "left.Token"),
            ),
            (
                enumeration("Status", "left.Status"),
                enumeration("PublicStatus", "left.Status"),
            ),
            (
                protocol("Readable", "left.Readable"),
                protocol("PublicReadable", "left.Readable"),
            ),
        ] {
            assert_eq!(
                Type::Union(vec![Type::Int, original]).union_enum_name(),
                Type::Union(vec![Type::Int, alias]).union_enum_name()
            );
        }
    }

    #[test]
    fn raw_identity_duplicate_union_renders_as_its_canonical_member() {
        let class = Type::Class {
            identity: Some("pkg.Item".to_string()),
            type_args: Vec::new(),
            name: "Item".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        };
        let snapshot = Type::Class {
            identity: Some("pkg.Item".to_string()),
            type_args: Vec::new(),
            name: "Item".to_string(),
            fields: vec![("value".to_string(), Type::Int)],
            methods: Vec::new(),
            parent_class: None,
        };
        assert_eq!(
            Type::Union(vec![class.clone(), snapshot]).union_enum_name(),
            class.rust_type()
        );
        assert_eq!(
            Type::Union(vec![class.clone(), class.clone()]).union_identity_key(),
            class.union_identity_key()
        );
    }
}
