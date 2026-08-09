use super::*;

pub(super) fn simple_type(
    sifr_name: &str,
    rust_name: &str,
    kind: RustBridgeTypeKind,
) -> RustBridgeTypeContract {
    RustBridgeTypeContract {
        sifr_type: sifr_name.to_string(),
        rust_borrowed_type: Some(rust_name.to_string()),
        rust_owned_type: Some(rust_name.to_string()),
        rust_return_type: Some(rust_name.to_string()),
        kind,
        unsupported_reason: None,
    }
}

pub(super) fn bridge_list_type(
    inner: &Type,
    module_name: Option<&String>,
    module_catalogs: &BTreeMap<Option<String>, ModuleCatalog>,
    catalog: Option<&ModuleCatalog>,
    generated_types: &mut GeneratedTypeCollector,
    position: BridgeTypePosition,
    structural_type_param: Option<&str>,
) -> RustBridgeTypeContract {
    let inner_ty = bridge_type_contract(
        inner,
        module_name,
        module_catalogs,
        catalog,
        generated_types,
        position.nested(),
        structural_type_param,
    );
    let Some(inner_owned) = inner_ty.rust_owned_type.clone() else {
        return unsupported_type(
            &Type::List(Box::new(inner.clone())),
            "list element type is not Rust bridge-compatible",
        );
    };
    RustBridgeTypeContract {
        sifr_type: format!("list[{}]", inner_ty.sifr_type),
        rust_borrowed_type: Some(format!("&[{inner_owned}]")),
        rust_owned_type: Some(format!("Vec<{inner_owned}>")),
        rust_return_type: Some(format!("Vec<{inner_owned}>")),
        kind: RustBridgeTypeKind::List,
        unsupported_reason: inner_ty.unsupported_reason,
    }
}

pub(super) fn bridge_dict_type(
    key: &Type,
    value: &Type,
    module_name: Option<&String>,
    module_catalogs: &BTreeMap<Option<String>, ModuleCatalog>,
    catalog: Option<&ModuleCatalog>,
    generated_types: &mut GeneratedTypeCollector,
    position: BridgeTypePosition,
    structural_type_param: Option<&str>,
) -> RustBridgeTypeContract {
    if !matches!(key.resolve_alias(), Type::Str) {
        return unsupported_type(
            &Type::Dict(Box::new(key.clone()), Box::new(value.clone())),
            "only dict[str, T] is Rust bridge-compatible",
        );
    }
    let value_ty = bridge_type_contract(
        value,
        module_name,
        module_catalogs,
        catalog,
        generated_types,
        position.nested(),
        structural_type_param,
    );
    let Some(value_owned) = value_ty.rust_owned_type.clone() else {
        return unsupported_type(
            &Type::Dict(Box::new(key.clone()), Box::new(value.clone())),
            "dict value type is not Rust bridge-compatible",
        );
    };
    let rust_type = format!("::sifr_runtime::interop::IndexMap<String, {value_owned}>");
    RustBridgeTypeContract {
        sifr_type: format!("dict[str, {}]", value_ty.sifr_type),
        rust_borrowed_type: Some(format!("&{rust_type}")),
        rust_owned_type: Some(rust_type.clone()),
        rust_return_type: Some(rust_type),
        kind: RustBridgeTypeKind::Dict,
        unsupported_reason: value_ty.unsupported_reason,
    }
}

pub(super) fn bridge_union_type(
    members: &[Type],
    module_name: Option<&String>,
    module_catalogs: &BTreeMap<Option<String>, ModuleCatalog>,
    catalog: Option<&ModuleCatalog>,
    generated_types: &mut GeneratedTypeCollector,
    position: BridgeTypePosition,
    structural_type_param: Option<&str>,
) -> RustBridgeTypeContract {
    if members.len() == 2 {
        let non_none = members
            .iter()
            .find(|member| !matches!(member.resolve_alias(), Type::None));
        if members
            .iter()
            .any(|member| matches!(member.resolve_alias(), Type::None))
        {
            if let Some(non_none) = non_none {
                let inner = bridge_type_contract(
                    non_none,
                    module_name,
                    module_catalogs,
                    catalog,
                    generated_types,
                    position.nested(),
                    structural_type_param,
                );
                let Some(inner_owned) = inner.rust_owned_type.clone() else {
                    return unsupported_type(
                        &Type::Union(members.to_vec()),
                        "Option inner type is not Rust bridge-compatible",
                    );
                };
                let rust_type = format!("Option<{inner_owned}>");
                return RustBridgeTypeContract {
                    sifr_type: format!("Option[{}]", inner.sifr_type),
                    rust_borrowed_type: Some(rust_type.clone()),
                    rust_owned_type: Some(rust_type.clone()),
                    rust_return_type: Some(rust_type),
                    kind: RustBridgeTypeKind::Option,
                    unsupported_reason: inner.unsupported_reason,
                };
            }
        }
    }
    unsupported_type(
        &Type::Union(members.to_vec()),
        "only Option[T] unions are Rust bridge-compatible",
    )
}

pub(super) fn bridge_tuple_type(
    items: &[Type],
    sifr_type: String,
    module_name: Option<&String>,
    module_catalogs: &BTreeMap<Option<String>, ModuleCatalog>,
) -> RustBridgeTypeContract {
    let Some(rust_items) = items
        .iter()
        .map(|item| tuple_item_rust_type(item, module_name, module_catalogs))
        .collect::<Option<Vec<_>>>()
    else {
        return RustBridgeTypeContract {
            sifr_type,
            rust_borrowed_type: None,
            rust_owned_type: None,
            rust_return_type: None,
            kind: RustBridgeTypeKind::Unsupported,
            unsupported_reason: Some(
                "tuple element type is not Rust bridge-compatible".to_string(),
            ),
        };
    };
    let rust_type = if rust_items.len() == 1 {
        format!("({},)", rust_items[0])
    } else {
        format!("({})", rust_items.join(", "))
    };
    RustBridgeTypeContract {
        sifr_type,
        rust_borrowed_type: Some(format!("&{rust_type}")),
        rust_owned_type: Some(rust_type.clone()),
        rust_return_type: Some(rust_type),
        kind: RustBridgeTypeKind::Tuple,
        unsupported_reason: None,
    }
}

pub(super) fn tuple_item_rust_type(
    ty: &Type,
    module_name: Option<&String>,
    module_catalogs: &BTreeMap<Option<String>, ModuleCatalog>,
) -> Option<String> {
    match ty.resolve_alias() {
        Type::Bool => Some("bool".to_string()),
        Type::FixedInt(fixed) => Some(fixed.rust_name().to_string()),
        Type::Int => Some("i64".to_string()),
        Type::Float => Some("f64".to_string()),
        Type::Str => Some("String".to_string()),
        Type::Bytes => Some("Vec<u8>".to_string()),
        Type::None => Some("()".to_string()),
        Type::Class { name, .. } => opaque_type_definition(name, module_name, module_catalogs)
            .ok()
            .flatten()
            .map(|target| {
                format!(
                    "::sifr_runtime::interop::Handle<{}>",
                    absolute_runtime_target(&target)
                )
            }),
        _ => None,
    }
}

pub(super) fn combine_generic_type(
    name: &str,
    sifr_type: String,
    kind: RustBridgeTypeKind,
    parts: &[RustBridgeTypeContract],
) -> RustBridgeTypeContract {
    let unsupported_reason = parts
        .iter()
        .find_map(|part| part.unsupported_reason.clone());
    let rust_parts = parts
        .iter()
        .map(|part| part.rust_return_type.clone())
        .collect::<Option<Vec<_>>>();
    let rust_type = rust_parts.map(|parts| format!("{name}<{}>", parts.join(", ")));
    RustBridgeTypeContract {
        sifr_type,
        rust_borrowed_type: None,
        rust_owned_type: None,
        rust_return_type: rust_type,
        kind,
        unsupported_reason,
    }
}
