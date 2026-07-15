use super::{HirExpr, LowerCtx, Type};

pub(super) fn consume_affine_collection_method_arguments(
    object_ty: &Type,
    method: &str,
    args: &[HirExpr],
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
    let Some(HirExpr::Name { name, ty }) = consumed_index.and_then(|index| args.get(index)) else {
        return;
    };
    if ty.contains_affine_resource() {
        ctx.mark_moved_with_flow(name);
    }
}
