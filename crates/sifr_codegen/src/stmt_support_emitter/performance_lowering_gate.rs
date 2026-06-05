use super::{HirExpr, HirStmt, Type};
use crate::hir_analysis::traversal::{self, TraversalConfig, TraversalControl};
use std::cell::Cell;

pub(crate) fn stmt_needs_performance_lowering(stmt: &HirStmt) -> bool {
    let needs_perf_lowering = Cell::new(false);
    let mut on_stmt = |node: &HirStmt| {
        if stmt_shape_needs_performance_lowering(node) {
            needs_perf_lowering.set(true);
            TraversalControl::Stop
        } else {
            TraversalControl::Continue
        }
    };
    let mut on_expr = |expr: &HirExpr| {
        if expr_shape_needs_performance_lowering(expr) {
            needs_perf_lowering.set(true);
            TraversalControl::Stop
        } else {
            TraversalControl::Continue
        }
    };
    let _ = traversal::walk_stmt_until(
        stmt,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    needs_perf_lowering.get()
}

fn stmt_shape_needs_performance_lowering(stmt: &HirStmt) -> bool {
    matches!(
        stmt,
        HirStmt::TupleUnpack { targets, .. }
            if targets.iter().any(|target| matches!(
                crate::resolve_alias_type_for_plain_call(&target.ty),
                Type::Str | Type::LiteralStr(_)
            ))
    ) || matches!(
        stmt,
        HirStmt::AugAssign { value, .. }
            if matches!(
                crate::resolve_alias_type_for_plain_call(value.ty()),
                Type::Str | Type::LiteralStr(_)
            )
    ) || matches!(
        stmt,
        HirStmt::Delete {
            object,
            ..
        } if matches!(
            object,
            HirExpr::FieldAccess { ty, .. }
                if matches!(
                    crate::resolve_alias_type_for_plain_call(ty),
                    Type::Dict(_, _) | Type::List(_)
                )
        )
    )
}

fn expr_shape_needs_performance_lowering(expr: &HirExpr) -> bool {
    match expr {
        HirExpr::Index { object, .. } => {
            matches!(
                crate::resolve_alias_type_for_plain_call(object.ty()),
                Type::Str | Type::LiteralStr(_)
            ) || nested_index_can_borrow(expr)
        }
        HirExpr::Slice { object, .. } => matches!(
            crate::resolve_alias_type_for_plain_call(object.ty()),
            Type::Str | Type::LiteralStr(_)
        ),
        HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } => {
            (method == "len"
                && args.is_empty()
                && matches!(
                    crate::resolve_alias_type_for_plain_call(object.ty()),
                    Type::Str | Type::LiteralStr(_)
                ))
                || dict_indexed_list_method_can_borrow(object, method, args)
        }
        HirExpr::ContainsOp { collection, .. } => {
            list_indexed_dict_contains_can_borrow(collection)
                || defaultdict_index_contains_can_borrow(collection)
        }
        _ => false,
    }
}

fn dict_indexed_list_method_can_borrow(object: &HirExpr, method: &str, args: &[HirExpr]) -> bool {
    let method_shape_matches =
        matches!(method, "pop" | "len") || (method == "append" && args.len() == 1);
    if !method_shape_matches {
        return false;
    }
    let HirExpr::Index {
        object: dict_object,
        ..
    } = object
    else {
        return false;
    };
    dict_value_is_list(dict_object.ty())
}

fn nested_index_can_borrow(expr: &HirExpr) -> bool {
    let HirExpr::Index { object, .. } = expr else {
        return false;
    };
    let HirExpr::Index {
        object: inner_object,
        ..
    } = object.as_ref()
    else {
        return false;
    };
    match crate::resolve_alias_type_for_plain_call(inner_object.ty()) {
        Type::List(inner) => matches!(
            crate::resolve_alias_type_for_plain_call(inner.as_ref()),
            Type::List(_) | Type::Dict(_, _)
        ),
        Type::Dict(_, value) => matches!(
            crate::resolve_alias_type_for_plain_call(value.as_ref()),
            Type::List(_)
        ),
        _ => false,
    }
}

fn list_indexed_dict_contains_can_borrow(collection: &HirExpr) -> bool {
    let HirExpr::Index {
        object: list_object,
        ..
    } = collection
    else {
        return false;
    };
    match crate::resolve_alias_type_for_plain_call(list_object.ty()) {
        Type::List(row) => matches!(
            crate::resolve_alias_type_for_plain_call(row.as_ref()),
            Type::Dict(_, _)
        ),
        _ => false,
    }
}

fn defaultdict_index_contains_can_borrow(collection: &HirExpr) -> bool {
    let HirExpr::Index { object, .. } = collection else {
        return false;
    };
    matches!(
        object.ty(),
        Type::Alias { name, .. } if matches!(
            name.as_str(),
            "__sifr_defaultdict_list" | "__sifr_defaultdict_set"
        )
    )
}

fn dict_value_is_list(ty: &Type) -> bool {
    match crate::resolve_alias_type_for_plain_call(ty) {
        Type::Dict(_, value) => matches!(
            crate::resolve_alias_type_for_plain_call(value.as_ref()),
            Type::List(_)
        ),
        _ => false,
    }
}
