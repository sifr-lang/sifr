use crate::RustExpr;
use sifr_ir::HirExpr;
use sifr_type_system::Type;

use super::flatten_option_value_for_target;

/// Adapt a collection operation whose HIR type was contextually narrowed to its target.
///
/// Safe reads still produce a representation-significant outer `Option` at runtime. Collection
/// literal inference can replace the expression's HIR type with the enclosing element type, so
/// recover the operation payload before adapting the lowered value.
pub(crate) fn adapt_collection_value_for_target(
    target: &Type,
    source_expr: &HirExpr,
    value: RustExpr,
) -> RustExpr {
    flatten_option_value_for_target(target, source_expr.ty(), value)
}

/// Adapt representation-significant optional wrappers inside an owned collection value.
pub(crate) fn adapt_collection_storage_for_target(
    target: &Type,
    source: &Type,
    value: RustExpr,
) -> RustExpr {
    let (target, source) = (
        crate::resolve_alias_type_for_plain_call(target),
        crate::resolve_alias_type_for_plain_call(source),
    );
    match (target, source) {
        (Type::List(target_element), Type::List(source_element)) => adapt_iterable_storage(
            target_element.as_ref(),
            source_element.as_ref(),
            value,
            "collect::<Vec<_>>",
        ),
        (Type::Set(target_element), Type::Set(source_element)) => adapt_iterable_storage(
            target_element.as_ref(),
            source_element.as_ref(),
            value,
            "collect::<std::collections::HashSet<_>>",
        ),
        (Type::Dict(target_key, target_value), Type::Dict(source_key, source_value)) => {
            let key_ident = RustExpr::Ident("__sifr_collection_key".to_string());
            let value_ident = RustExpr::Ident("__sifr_collection_value".to_string());
            let adapted_key = flatten_option_value_for_target(
                target_key.as_ref(),
                source_key.as_ref(),
                key_ident.clone(),
            );
            let adapted_value = flatten_option_value_for_target(
                target_value.as_ref(),
                source_value.as_ref(),
                value_ident.clone(),
            );
            if adapted_key == key_ident && adapted_value == value_ident {
                return value;
            }
            collect_mapped_storage(
                value,
                "(__sifr_collection_key, __sifr_collection_value)",
                RustExpr::Tuple(vec![adapted_key, adapted_value]),
                "collect::<std::collections::HashMap<_, _>>",
            )
        }
        _ => value,
    }
}

fn adapt_iterable_storage(
    target_element: &Type,
    source_element: &Type,
    value: RustExpr,
    collect_method: &str,
) -> RustExpr {
    let element_ident = RustExpr::Ident("__sifr_collection_value".to_string());
    let adapted =
        flatten_option_value_for_target(target_element, source_element, element_ident.clone());
    if adapted == element_ident {
        return value;
    }
    collect_mapped_storage(value, "__sifr_collection_value", adapted, collect_method)
}

fn collect_mapped_storage(
    value: RustExpr,
    parameter: &str,
    body: RustExpr,
    collect_method: &str,
) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(value),
                method: "into_iter".to_string(),
                args: Vec::new(),
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![crate::RustParam::Named {
                    name: parameter.to_string(),
                    ty: crate::RustType::Named("_".to_string()),
                }],
                body: Box::new(body),
                is_move: false,
            }],
        }),
        method: collect_method.to_string(),
        args: Vec::new(),
    }
}
