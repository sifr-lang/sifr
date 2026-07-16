use super::{consume_owned_value, HirExpr, LowerCtx, TextRange, Type};

pub(in crate::lower) fn affine_value_references_name(expr: &HirExpr, target: &str) -> bool {
    if !expr.ty().contains_affine_resource() {
        return false;
    }
    match expr {
        HirExpr::Name { name, .. } => name == target,
        HirExpr::IfExpr {
            then_expr,
            else_expr,
            ..
        } => {
            affine_value_references_name(then_expr, target)
                || affine_value_references_name(else_expr, target)
        }
        HirExpr::OkWrap { value, .. }
        | HirExpr::ErrWrap { value, .. }
        | HirExpr::QuestionMark { expr: value, .. }
        | HirExpr::WalrusExpr { value, .. } => affine_value_references_name(value, target),
        _ => false,
    }
}

pub(super) fn consume_affine_collection_method_arguments(
    object_ty: &Type,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    ctx: &mut LowerCtx,
) {
    let consumed_index = match object_ty.resolve_alias() {
        Type::List(element) if element.contains_affine_resource() => match method {
            "append" | "appendleft" => Some(0),
            "insert" => Some(1),
            _ => None,
        },
        Type::Dict(_, value) if value.contains_affine_resource() && method == "pop" => Some(1),
        _ => None,
    };
    let Some(index) = consumed_index else {
        return;
    };
    if let (Some(arg), Some(range)) = (args.get(index), arg_ranges.get(index)) {
        consume_owned_value(arg, *range, ctx);
    }
}
