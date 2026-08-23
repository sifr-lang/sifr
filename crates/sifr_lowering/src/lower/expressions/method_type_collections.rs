use super::{
    DiagnosticCode, HirExpr, LowerCtx, TextRange, Type, container_literal_diagnostics,
    expression_diagnostics, list_append_argument_type_mismatch, method_count_range,
    reject_exact_method_arg_count, reject_max_method_arg_count, reject_method_arg_count,
    reject_no_method_args, str, validate_dict_update_arg, validate_list_extend_arg,
    validate_set_iterable_arg,
};
use crate::lower::type_bounds::{
    supports_structural_equality_in_context, supports_total_order_in_context,
};
use sifr_type_system::safe_optional_result;
pub(super) fn resolve_list_method_type(
    elem_ty: &Type,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    let requires_clone = matches!(method, "copy" | "extend");
    let requires_equality = matches!(method, "count" | "contains" | "remove" | "index");
    let requires_total_order = method == "sort";
    let requires_trait_capability = requires_clone || requires_equality || requires_total_order;
    if requires_clone && !elem_ty.supports_derived_clone() && !elem_ty.contains_affine_resource() {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!("list.{method}() requires elements with generated Rust Clone support"),
            method_range,
        );
        return None;
    }
    if requires_equality
        && !supports_structural_equality_in_context(elem_ty, ctx)
        && !elem_ty.contains_affine_resource()
    {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!("list.{method}() requires elements with generated Rust PartialEq support"),
            method_range,
        );
        return None;
    }
    if requires_total_order
        && !supports_total_order_in_context(elem_ty, ctx)
        && !elem_ty.contains_affine_resource()
    {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "list.sort() requires elements with generated Rust total Ord support".to_string(),
            method_range,
        );
        return None;
    }
    if elem_ty.contains_affine_resource() && requires_trait_capability {
        ctx.error_with_code_at(
            DiagnosticCode::PYZC_INVALID_DECLARATION,
            format!(
                "list.{method}() is unavailable for elements containing affine Python resources because it requires cloning, comparing, or repeatedly projecting them"
            ),
            method_range,
        );
        return None;
    }
    match method {
        "append" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    "list.append",
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            if !args[0].ty().is_assignable_to(elem_ty) {
                list_append_argument_type_mismatch(ctx, args[0].ty(), elem_ty, arg_ranges[0]);
            }
            Some(Type::None)
        }
        "extend" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    "list.extend",
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            validate_list_extend_arg(elem_ty, args[0].ty(), arg_ranges[0], ctx);
            Some(Type::None)
        }
        "insert" => {
            if args.len() != 2 {
                reject_exact_method_arg_count(
                    ctx,
                    "list.insert",
                    2,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            Some(Type::None)
        }
        "clear" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, "list.clear", arg_ranges, method_range);
                return None;
            }
            Some(Type::None)
        }
        "copy" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, "list.copy", arg_ranges, method_range);
                return None;
            }
            Some(Type::List(Box::new(elem_ty.clone())))
        }
        "reverse" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, "list.reverse", arg_ranges, method_range);
                return None;
            }
            Some(Type::None)
        }
        "sort" => {
            if args.len() > 1 {
                reject_max_method_arg_count(
                    ctx,
                    "list.sort",
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            if let Some(reverse_arg) = args.first() {
                if reverse_arg.ty() != &Type::Bool {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "list.sort() argument 'reverse' must be 'bool', got '{}'",
                            reverse_arg.ty().display_name()
                        ),
                        arg_ranges[0],
                    );
                    return None;
                }
            }
            Some(Type::None)
        }
        "count" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    "list.count",
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            Some(Type::Int)
        }
        "contains" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    "list.contains",
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            Some(Type::Bool)
        }
        "len" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, "list.len", arg_ranges, method_range);
                return None;
            }
            Some(Type::Int)
        }
        "pop" => {
            if args.len() > 1 {
                reject_max_method_arg_count(
                    ctx,
                    "list.pop",
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            if let Some(index_arg) = args.first() {
                if index_arg.ty() != &Type::Int {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "list.pop() index must be 'int', got '{}'",
                            index_arg.ty().display_name()
                        ),
                        arg_ranges[0],
                    );
                }
            }
            // pop() returns Option[T] = T | None
            Some(safe_optional_result(elem_ty.clone()))
        }
        "popleft" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, "list.popleft", arg_ranges, method_range);
                return None;
            }
            Some(safe_optional_result(elem_ty.clone()))
        }
        "appendleft" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    "list.appendleft",
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            Some(Type::None)
        }
        "remove" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    "list.remove",
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            Some(Type::None)
        }
        "index" => {
            if args.is_empty() || args.len() > 3 {
                reject_method_arg_count(
                    ctx,
                    format!("list.index() takes 1 to 3 arguments, got {}", args.len()),
                    method_count_range(args.len(), 3, arg_ranges, method_range),
                );
                return None;
            }
            for (bound_index, bound) in args.iter().enumerate().skip(1) {
                if bound.ty() != &Type::Int {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "list.index() bounds must be 'int', got '{}'",
                            bound.ty().display_name()
                        ),
                        arg_ranges.get(bound_index).copied().unwrap_or(method_range),
                    );
                }
            }
            // Returns Option[int] = int | None (safe: no panic if not found)
            Some(Type::Union(vec![Type::Int, Type::None]))
        }
        _ => {
            ctx.error_with_code_at(
                DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                format!("list has no method '{method}'"),
                method_range,
            );
            None
        }
    }
}

pub(super) fn resolve_dict_method_type(
    key_ty: &Type,
    val_ty: &Type,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    let unresolved_empty_get = method == "get"
        && args.len() == 2
        && matches!(key_ty.resolve_alias(), Type::Any | Type::Unknown)
        && matches!(val_ty.resolve_alias(), Type::Any | Type::Unknown)
        && !matches!(args[0].ty().resolve_alias(), Type::Any | Type::Unknown)
        && !matches!(args[1].ty().resolve_alias(), Type::Any | Type::Unknown);
    if !matches!(method, "len" | "clear")
        && !unresolved_empty_get
        && container_literal_diagnostics::reject_unhashable_container_type(
            ctx,
            "dict key",
            key_ty,
            method_range,
        )
    {
        return None;
    }
    let requires_reusable_values = matches!(
        method,
        "values" | "items" | "update" | "copy" | "get" | "setdefault"
    );
    if requires_reusable_values
        && !unresolved_empty_get
        && !val_ty.supports_derived_clone()
        && !val_ty.contains_affine_resource()
    {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!("dict.{method}() requires values with generated Rust Clone support"),
            method_range,
        );
        return None;
    }
    if val_ty.contains_affine_resource() && requires_reusable_values {
        ctx.error_with_code_at(
            DiagnosticCode::PYZC_INVALID_DECLARATION,
            format!(
                "dict.{method}() is unavailable for values containing affine Python resources because it clones or projects stored values"
            ),
            method_range,
        );
        return None;
    }
    let requires_reusable_keys = matches!(method, "keys" | "items" | "copy");
    if requires_reusable_keys
        && !key_ty.supports_derived_clone()
        && !key_ty.contains_affine_resource()
    {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!("dict.{method}() requires keys with generated Rust Clone support"),
            method_range,
        );
        return None;
    }
    match method {
        "len" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, "dict.len", arg_ranges, method_range);
                return None;
            }
            Some(Type::Int)
        }
        "keys" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, "dict.keys", arg_ranges, method_range);
                return None;
            }
            Some(Type::List(Box::new(key_ty.clone())))
        }
        "values" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, "dict.values", arg_ranges, method_range);
                return None;
            }
            Some(Type::List(Box::new(val_ty.clone())))
        }
        "items" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, "dict.items", arg_ranges, method_range);
                return None;
            }
            Some(Type::List(Box::new(Type::Tuple(vec![
                key_ty.clone(),
                val_ty.clone(),
            ]))))
        }
        "update" => {
            if args.len() > 2 {
                reject_max_method_arg_count(
                    ctx,
                    "dict.update",
                    2,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            if let Some(arg) = args.first() {
                validate_dict_update_arg(key_ty, val_ty, arg.ty(), arg_ranges[0], ctx);
            }
            if let Some(keyword_dict) = args.get(1) {
                validate_dict_update_arg(key_ty, val_ty, keyword_dict.ty(), arg_ranges[1], ctx);
            }
            Some(Type::None)
        }
        "clear" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, "dict.clear", arg_ranges, method_range);
                return None;
            }
            Some(Type::None)
        }
        "copy" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, "dict.copy", arg_ranges, method_range);
                return None;
            }
            Some(Type::Dict(
                Box::new(key_ty.clone()),
                Box::new(val_ty.clone()),
            ))
        }
        "contains" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    "dict.contains",
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            if !args[0].ty().is_assignable_to(key_ty) {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "dict.contains() key type '{}' is not compatible with dict key type '{}'",
                        args[0].ty().display_name(),
                        key_ty.display_name()
                    ),
                    arg_ranges[0],
                );
            }
            Some(Type::Bool)
        }
        "get" => {
            if args.is_empty() || args.len() > 2 {
                reject_method_arg_count(
                    ctx,
                    format!("dict.get() takes 1 or 2 arguments, got {}", args.len()),
                    method_count_range(args.len(), 2, arg_ranges, method_range),
                );
                return None;
            }
            if !args[0].ty().is_assignable_to(key_ty) {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "dict.get() key type '{}' is not compatible with dict key type '{}'",
                        args[0].ty().display_name(),
                        key_ty.display_name()
                    ),
                    arg_ranges[0],
                );
            }
            if args.len() == 2 {
                if !args[1].ty().is_assignable_to(val_ty) {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "dict.get() default type '{}' is not compatible with dict value type '{}'",
                            args[1].ty().display_name(),
                            val_ty.display_name()
                        ),
                        arg_ranges[1],
                    );
                }
                // When V is still unknown/Any (e.g. empty literal before specialization),
                // preserve precision from the provided default instead of leaking `Any`.
                if matches!(val_ty, Type::Any | Type::Unknown) {
                    Some(args[1].ty().clone())
                } else {
                    // dict.get(key, default) -> V (returns default if key not found)
                    Some(val_ty.clone())
                }
            } else {
                // dict.get(key) -> V | None
                Some(safe_optional_result(val_ty.clone()))
            }
        }
        "pop" => {
            if args.is_empty() || args.len() > 2 {
                reject_method_arg_count(
                    ctx,
                    format!("dict.pop() takes 1 or 2 arguments, got {}", args.len()),
                    method_count_range(args.len(), 2, arg_ranges, method_range),
                );
                return None;
            }
            if !args[0].ty().is_assignable_to(key_ty) {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "dict.pop() key type '{}' is not compatible with dict key type '{}'",
                        args[0].ty().display_name(),
                        key_ty.display_name()
                    ),
                    arg_ranges[0],
                );
            }
            if args.len() == 2 {
                if !args[1].ty().is_assignable_to(val_ty) {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "dict.pop() default type '{}' is not compatible with dict value type '{}'",
                            args[1].ty().display_name(),
                            val_ty.display_name()
                        ),
                        arg_ranges[1],
                    );
                }
                Some(val_ty.clone())
            } else {
                // pop() returns Option[V] = V | None
                Some(safe_optional_result(val_ty.clone()))
            }
        }
        "setdefault" => {
            if args.len() != 2 {
                reject_exact_method_arg_count(
                    ctx,
                    "dict.setdefault",
                    2,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            if !args[0].ty().is_assignable_to(key_ty) {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "dict.setdefault() key type '{}' is not compatible with dict key type '{}'",
                        args[0].ty().display_name(),
                        key_ty.display_name()
                    ),
                    arg_ranges[0],
                );
            }
            if !args[1].ty().is_assignable_to(val_ty) {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "dict.setdefault() default type '{}' is not compatible with dict value type '{}'",
                        args[1].ty().display_name(),
                        val_ty.display_name()
                    ),
                    arg_ranges[1],
                );
            }
            Some(val_ty.clone())
        }
        _ => {
            ctx.error_with_code_at(
                DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                format!("dict has no method '{method}'"),
                method_range,
            );
            None
        }
    }
}

pub(super) fn resolve_set_method_type(
    elem_ty: &Type,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    if !matches!(method, "len" | "clear")
        && container_literal_diagnostics::reject_unhashable_container_type(
            ctx,
            "set element",
            elem_ty,
            method_range,
        )
    {
        return None;
    }
    match method {
        "len" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, "set.len", arg_ranges, method_range);
                return None;
            }
            Some(Type::Int)
        }
        "add" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    "set.add",
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            if !matches!(elem_ty.resolve_alias(), Type::Any | Type::Unknown)
                && !args[0].ty().is_assignable_to(elem_ty)
            {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT,
                    format!(
                        "set element type conflict: expected '{}', got '{}'",
                        elem_ty.display_name(),
                        args[0].ty().display_name()
                    ),
                    arg_ranges[0],
                );
                return None;
            }
            Some(Type::None)
        }
        "remove" | "discard" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    &format!("set.{method}"),
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            Some(Type::None)
        }
        "contains" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    "set.contains",
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            Some(Type::Bool)
        }
        "clear" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, "set.clear", arg_ranges, method_range);
                return None;
            }
            Some(Type::None)
        }
        "copy" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, "set.copy", arg_ranges, method_range);
                return None;
            }
            Some(Type::Set(Box::new(elem_ty.clone())))
        }
        "union" | "intersection" | "difference" => {
            for (index, arg) in args.iter().enumerate() {
                validate_set_iterable_arg(elem_ty, arg.ty(), method, arg_ranges[index], ctx);
            }
            Some(Type::Set(Box::new(elem_ty.clone())))
        }
        "symmetric_difference" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    &format!("set.{method}"),
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            validate_set_iterable_arg(elem_ty, args[0].ty(), method, arg_ranges[0], ctx);
            Some(Type::Set(Box::new(elem_ty.clone())))
        }
        "update" | "intersection_update" | "difference_update" => {
            for (index, arg) in args.iter().enumerate() {
                validate_set_iterable_arg(elem_ty, arg.ty(), method, arg_ranges[index], ctx);
            }
            Some(Type::None)
        }
        "symmetric_difference_update" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    &format!("set.{method}"),
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            validate_set_iterable_arg(elem_ty, args[0].ty(), method, arg_ranges[0], ctx);
            Some(Type::None)
        }
        "issubset" | "issuperset" | "isdisjoint" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    &format!("set.{method}"),
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            validate_set_iterable_arg(elem_ty, args[0].ty(), method, arg_ranges[0], ctx);
            Some(Type::Bool)
        }
        "pop" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, "set.pop", arg_ranges, method_range);
                return None;
            }
            // Returns Option[T] = T | None (safe: no panic on empty set)
            Some(safe_optional_result(elem_ty.clone()))
        }
        _ => {
            ctx.error_with_code_at(
                DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                format!("set has no method '{method}'"),
                method_range,
            );
            None
        }
    }
}
