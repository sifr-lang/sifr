use sifr_type_system::Type;

use crate::{RustExpr, render_expr};

pub(super) fn argument_composite_conversion_required(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::Union(members) if optional_inner(members).is_some())
        || composite_conversion_required(ty)
}

pub(super) fn composite_conversion_required(ty: &Type) -> bool {
    match ty.resolve_alias() {
        Type::Int => true,
        Type::List(inner) => composite_conversion_required(inner),
        Type::Dict(key, _) => key.resolve_alias() == &Type::Str,
        Type::Union(members) => optional_inner(members).is_some_and(composite_conversion_required),
        _ => false,
    }
}

pub(super) fn sifr_composite_to_bridge_expr(
    value: &RustExpr,
    ty: &Type,
    borrowed: bool,
) -> RustExpr {
    let value = render_expr(value);
    let converted = RustExpr::Verbatim(sifr_value_to_bridge_expr(&value, ty, borrowed, 0));
    if borrowed && matches!(ty.resolve_alias(), Type::List(_) | Type::Dict(_, _)) {
        RustExpr::Ref {
            mutable: false,
            expr: Box::new(converted),
        }
    } else {
        converted
    }
}

pub(super) fn bridge_composite_to_sifr_expr(value: &RustExpr, ty: &Type) -> RustExpr {
    let value = format!("({})", render_expr(value));
    RustExpr::Verbatim(bridge_value_to_sifr_expr(&value, ty, 0))
}

pub(super) fn hash_map_to_bridge_index_map_expr(
    name: &str,
    item_type: &Type,
    borrowed: bool,
) -> RustExpr {
    let value = RustExpr::Ident(name.to_string());
    sifr_composite_to_bridge_expr(
        &value,
        &Type::Dict(Box::new(Type::Str), Box::new(item_type.clone())),
        borrowed,
    )
}

pub(super) fn bridge_index_map_to_hash_map_expr(value: &RustExpr, item_type: &Type) -> RustExpr {
    bridge_composite_to_sifr_expr(
        value,
        &Type::Dict(Box::new(Type::Str), Box::new(item_type.clone())),
    )
}

fn sifr_value_to_bridge_expr(value: &str, ty: &Type, borrowed: bool, depth: usize) -> String {
    match ty.resolve_alias() {
        Type::Int => {
            let value = if borrowed {
                format!("*{value}")
            } else {
                value.to_string()
            };
            format!("::sifr_runtime::interop::SifrIntBridge::from({value})")
        }
        Type::List(inner) => {
            let item = format!("__sifr_bridge_item_{depth}");
            let iter = value_iter(value, borrowed);
            let converted = sifr_value_to_bridge_expr(&item, inner, borrowed, depth + 1);
            format!("{iter}.map(|{item}| {converted}).collect::<Vec<_>>()")
        }
        Type::Dict(key, item) if key.resolve_alias() == &Type::Str => {
            sifr_dict_to_bridge_expr(value, item, borrowed, depth)
        }
        Type::Union(members) => optional_inner(members).map_or_else(
            || {
                if borrowed {
                    format!("{value}.clone()")
                } else {
                    value.to_string()
                }
            },
            |inner| {
                let item = format!("__sifr_bridge_item_{depth}");
                let receiver = if borrowed {
                    format!("{value}.as_ref()")
                } else {
                    value.to_string()
                };
                let converted = sifr_value_to_bridge_expr(&item, inner, borrowed, depth + 1);
                format!("{receiver}.map(|{item}| {converted})")
            },
        ),
        _ if borrowed => format!("{value}.clone()"),
        _ => value.to_string(),
    }
}

fn sifr_dict_to_bridge_expr(value: &str, item_type: &Type, borrowed: bool, depth: usize) -> String {
    let key = format!("__sifr_bridge_key_{depth}");
    let item = format!("__sifr_bridge_value_{depth}");
    let iter = value_iter(value, borrowed);
    let key_expr = if borrowed {
        format!("{key}.clone()")
    } else {
        key.clone()
    };
    let item_expr = sifr_value_to_bridge_expr(&item, item_type, borrowed, depth + 1);
    format!(
        "{iter}.map(|({key}, {item})| ({key_expr}, {item_expr}))\
         .collect::<::sifr_runtime::interop::IndexMap<_, _>>()"
    )
}

fn bridge_value_to_sifr_expr(value: &str, ty: &Type, depth: usize) -> String {
    match ty.resolve_alias() {
        Type::Int => format!("{value}.to_i64_saturating()"),
        Type::List(inner) => {
            let item = format!("__sifr_value_{depth}");
            let converted = bridge_value_to_sifr_expr(&item, inner, depth + 1);
            format!("{value}.into_iter().map(|{item}| {converted}).collect::<Vec<_>>()")
        }
        Type::Dict(key, item) if key.resolve_alias() == &Type::Str => {
            bridge_dict_to_sifr_expr(value, item, depth)
        }
        Type::Union(members) => optional_inner(members).map_or_else(
            || value.to_string(),
            |inner| {
                let item = format!("__sifr_value_{depth}");
                let converted = bridge_value_to_sifr_expr(&item, inner, depth + 1);
                format!("{value}.map(|{item}| {converted})")
            },
        ),
        _ => value.to_string(),
    }
}

fn bridge_dict_to_sifr_expr(value: &str, item_type: &Type, depth: usize) -> String {
    let key = format!("__sifr_key_{depth}");
    let item = if depth == 0 {
        "__sifr_bridge_value".to_string()
    } else {
        format!("__sifr_value_{depth}")
    };
    let converted = bridge_value_to_sifr_expr(&item, item_type, depth + 1);
    format!(
        "{value}.into_iter().map(|({key}, {item})| ({key}, {converted}))\
         .collect::<::std::collections::HashMap<_, _>>()"
    )
}

fn value_iter(value: &str, borrowed: bool) -> String {
    if borrowed {
        format!("{value}.iter()")
    } else {
        format!("{value}.into_iter()")
    }
}

fn optional_inner(members: &[Type]) -> Option<&Type> {
    (members.len() == 2
        && members
            .iter()
            .any(|member| member.resolve_alias() == &Type::None))
    .then(|| {
        members
            .iter()
            .find(|member| member.resolve_alias() != &Type::None)
    })
    .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_expr;

    #[test]
    fn borrowed_nested_list_values_are_cloned_into_bridge_maps() {
        let expr =
            hash_map_to_bridge_index_map_expr("values", &Type::List(Box::new(Type::Str)), true);
        let rendered = render_expr(&expr);

        assert!(rendered.starts_with("&values.iter()"));
        assert!(rendered.contains("__sifr_bridge_key_0.clone()"));
        assert!(rendered.contains("__sifr_bridge_value_0.iter()"));
        assert!(rendered.contains("__sifr_bridge_item_1.clone()"));
        assert!(rendered.contains("IndexMap<_, _>"));
    }

    #[test]
    fn owned_integer_dict_values_convert_to_exact_bridge_ints() {
        let expr = hash_map_to_bridge_index_map_expr("values", &Type::Int, false);
        let rendered = render_expr(&expr);

        assert!(rendered.contains("values.into_iter()"));
        assert!(rendered.contains("SifrIntBridge::from(__sifr_bridge_value_0)"));
        assert!(rendered.contains("IndexMap<_, _>"));
    }

    #[test]
    fn nested_integer_lists_convert_recursively_in_both_directions() {
        let item = Type::List(Box::new(Type::Int));
        let argument = render_expr(&hash_map_to_bridge_index_map_expr("values", &item, true));
        let returned = render_expr(&bridge_index_map_to_hash_map_expr(
            &RustExpr::Ident("values".to_string()),
            &item,
        ));

        assert!(argument.contains("__sifr_bridge_item_1"));
        assert!(argument.contains("SifrIntBridge::from(*__sifr_bridge_item_1)"));
        assert!(returned.contains("__sifr_value_1.to_i64_saturating()"));
    }

    #[test]
    fn nested_dicts_convert_between_hash_map_and_index_map_recursively() {
        let item = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));
        let argument = render_expr(&hash_map_to_bridge_index_map_expr("values", &item, false));
        let returned = render_expr(&bridge_index_map_to_hash_map_expr(
            &RustExpr::Ident("values".to_string()),
            &item,
        ));

        assert_eq!(argument.matches("IndexMap<_, _>").count(), 2);
        assert_eq!(returned.matches("HashMap<_, _>").count(), 2);
        assert!(returned.contains("to_i64_saturating()"));
    }

    #[test]
    fn top_level_lists_convert_nested_collections_and_exact_ints() {
        let dict_list = Type::List(Box::new(Type::Dict(
            Box::new(Type::Str),
            Box::new(Type::Str),
        )));
        let int_lists = Type::List(Box::new(Type::List(Box::new(Type::Int))));
        let values = RustExpr::Ident("values".to_string());

        let borrowed_dicts = render_expr(&sifr_composite_to_bridge_expr(&values, &dict_list, true));
        let borrowed_ints = render_expr(&sifr_composite_to_bridge_expr(&values, &int_lists, true));
        let returned_dicts = render_expr(&bridge_composite_to_sifr_expr(
            &RustExpr::Ident("values".to_string()),
            &dict_list,
        ));
        let returned_ints = render_expr(&bridge_composite_to_sifr_expr(
            &RustExpr::Ident("values".to_string()),
            &int_lists,
        ));

        assert!(borrowed_dicts.starts_with("&values.iter()"));
        assert!(borrowed_dicts.contains("IndexMap<_, _>"));
        assert!(borrowed_ints.contains("SifrIntBridge::from(*__sifr_bridge_item_1)"));
        assert!(returned_dicts.contains("HashMap<_, _>"));
        assert!(returned_ints.contains("__sifr_value_1.to_i64_saturating()"));
    }

    #[test]
    fn options_recursively_convert_collection_payloads_without_outer_references() {
        let optional_list = Type::Union(vec![Type::List(Box::new(Type::Int)), Type::None]);
        let values = RustExpr::Ident("values".to_string());

        let argument = render_expr(&sifr_composite_to_bridge_expr(
            &values,
            &optional_list,
            true,
        ));
        let returned = render_expr(&bridge_composite_to_sifr_expr(
            &RustExpr::Ident("values".to_string()),
            &optional_list,
        ));

        assert!(argument.starts_with("values.as_ref().map("));
        assert!(!argument.starts_with('&'));
        assert!(argument.contains("SifrIntBridge::from(*__sifr_bridge_item_1)"));
        assert!(returned.contains("__sifr_value_1.to_i64_saturating()"));
    }

    #[test]
    fn composite_root_identifiers_escape_rust_keywords() {
        let keyword = RustExpr::Ident("type".to_string());
        let dict = Type::Dict(Box::new(Type::Str), Box::new(Type::Str));

        let argument = render_expr(&sifr_composite_to_bridge_expr(&keyword, &dict, true));

        assert!(argument.starts_with("&r#type.iter()"));
    }
}
