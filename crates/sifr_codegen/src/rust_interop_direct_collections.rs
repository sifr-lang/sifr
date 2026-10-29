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
        Type::Tuple(items) => items.iter().any(composite_conversion_required),
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
        Type::Tuple(items) => {
            let values = items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    if borrowed && matches!(item.resolve_alias(), Type::Int) {
                        return format!(
                            "::sifr_runtime::interop::SifrIntBridge::from(&({value}).{index})"
                        );
                    }
                    sifr_value_to_bridge_expr(
                        &format!("({value}).{index}"),
                        item,
                        borrowed,
                        depth + 1,
                    )
                })
                .collect::<Vec<_>>();
            tuple_expr(&values)
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
                let receiver = if borrowed {
                    format!("{value}.as_ref()")
                } else {
                    value.to_string()
                };
                if !composite_conversion_required(inner) {
                    return receiver;
                }
                let item = format!("__sifr_bridge_item_{depth}");
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
        Type::Int => format!("{value}.into_sifr_int()"),
        Type::List(inner) => {
            let item = format!("__sifr_value_{depth}");
            let converted = bridge_value_to_sifr_expr(&item, inner, depth + 1);
            format!("{value}.into_iter().map(|{item}| {converted}).collect::<Vec<_>>()")
        }
        Type::Dict(key, item) if key.resolve_alias() == &Type::Str => {
            bridge_dict_to_sifr_expr(value, item, depth)
        }
        Type::Tuple(items) => {
            let tuple = format!("__sifr_bridge_tuple_{depth}");
            let values = items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    bridge_value_to_sifr_expr(&format!("{tuple}.{index}"), item, depth + 1)
                })
                .collect::<Vec<_>>();
            format!("{{ let {tuple} = {value}; {} }}", tuple_expr(&values))
        }
        Type::Union(members) => optional_inner(members).map_or_else(
            || value.to_string(),
            |inner| {
                if !composite_conversion_required(inner) {
                    return value.to_string();
                }
                let item = format!("__sifr_value_{depth}");
                let converted = bridge_value_to_sifr_expr(&item, inner, depth + 1);
                format!("{value}.map(|{item}| {converted})")
            },
        ),
        _ => value.to_string(),
    }
}

fn tuple_expr(values: &[String]) -> String {
    match values {
        [value] => format!("({value},)"),
        _ => format!("({})", values.join(", ")),
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
#[path = "../tests/unit/rust_interop_direct_collections/tests.rs"]
mod tests;
