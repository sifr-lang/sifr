use crate::hir_nodes::HirExpr;
use sifr_type_system::Type;

pub(super) fn specialized_indexed_storage_base<'a>(
    expr: &'a HirExpr,
    method: &str,
) -> Option<&'a HirExpr> {
    let HirExpr::Index { object, .. } = expr else {
        return None;
    };
    if let Type::Alias { name, .. } = object.ty() {
        // The caller has already established a mutable receiver convention.
        // Any mutating method on these compiler-owned aliases is lowered
        // through the defaultdict entry path, so its checked place is the
        // backing map rather than the temporary bucket value.
        if matches!(
            name.as_str(),
            "__sifr_defaultdict_list" | "__sifr_defaultdict_set"
        ) {
            return Some(object);
        }
    }
    if !matches!(method, "append" | "pop") {
        return None;
    }
    let Type::Dict(_, value_ty) = object.ty().resolve_alias() else {
        return None;
    };
    if !matches!(value_ty.as_ref().resolve_alias(), Type::List(_)) {
        return None;
    }
    Some(object)
}

pub(super) fn indexed_storage_borrow_follows_argument_evaluation(
    expr: &HirExpr,
    method: &str,
) -> bool {
    let HirExpr::Index { object, .. } = expr else {
        return false;
    };
    let Type::Alias { name, .. } = object.ty() else {
        return false;
    };
    (name == "__sifr_defaultdict_list" && method == "extend")
        || (name == "__sifr_defaultdict_set"
            && matches!(
                method,
                "update"
                    | "intersection_update"
                    | "difference_update"
                    | "symmetric_difference_update"
            ))
}
