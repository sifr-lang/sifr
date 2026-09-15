use super::ErrorReferences;
use sifr_ir::HirStmt;

pub(super) fn collect_checked_place_stmt_error_refs(
    stmt: &HirStmt,
    referenced: &mut ErrorReferences,
    builtin_error_classes: &[&str],
) -> bool {
    match stmt {
        HirStmt::StarUnpack { value, failure, .. } => {
            collect_expr(value, referenced, builtin_error_classes);
            collect_failure(failure.as_ref(), referenced, builtin_error_classes);
        }
        HirStmt::SubscriptAssign {
            index,
            value,
            failure,
            ..
        }
        | HirStmt::AttributeSubscriptAssign {
            index,
            value,
            failure,
            ..
        }
        | HirStmt::SubscriptAugAssign {
            index,
            value,
            failure,
            ..
        } => {
            collect_expr(index, referenced, builtin_error_classes);
            collect_expr(value, referenced, builtin_error_classes);
            collect_failure(failure.as_ref(), referenced, builtin_error_classes);
        }
        HirStmt::NestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            outer_failure,
            inner_failure,
            ..
        }
        | HirStmt::AttributeNestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            outer_failure,
            inner_failure,
            ..
        } => {
            collect_expr(outer_index, referenced, builtin_error_classes);
            collect_expr(inner_index, referenced, builtin_error_classes);
            collect_expr(value, referenced, builtin_error_classes);
            collect_failure(outer_failure.as_ref(), referenced, builtin_error_classes);
            collect_failure(inner_failure.as_ref(), referenced, builtin_error_classes);
        }
        HirStmt::Delete {
            object,
            index,
            failure,
        } => {
            collect_expr(object, referenced, builtin_error_classes);
            collect_expr(index, referenced, builtin_error_classes);
            collect_failure(failure.as_ref(), referenced, builtin_error_classes);
        }
        _ => return false,
    }
    true
}

fn collect_expr(
    expr: &sifr_ir::HirExpr,
    referenced: &mut ErrorReferences,
    builtin_error_classes: &[&str],
) {
    super::collect_expr_error_refs(expr, referenced, builtin_error_classes);
}

fn collect_failure(
    failure: Option<&sifr_type_system::Type>,
    referenced: &mut ErrorReferences,
    builtin_error_classes: &[&str],
) {
    if let Some(failure) = failure {
        super::collect_type_error_refs(failure, referenced, builtin_error_classes);
    }
}
