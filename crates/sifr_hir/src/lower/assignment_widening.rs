use super::LowerCtx;
use sifr_type_system::{make_union, union_contains_none, Type};

pub(super) fn reconcile_optional_reassignment(
    ctx: &mut LowerCtx,
    name: &str,
    current_ty: &Type,
    incoming_ty: &Type,
    can_widen: bool,
) -> bool {
    if incoming_ty.is_assignable_to(current_ty) {
        return true;
    }
    if !can_widen {
        return false;
    }
    if !(current_ty == &Type::None
        || incoming_ty == &Type::None
        || union_contains_none(current_ty)
        || union_contains_none(incoming_ty))
    {
        return false;
    }
    let widened = make_union(vec![current_ty.clone(), incoming_ty.clone()]);
    ctx.scope.set_type(name, widened)
}
