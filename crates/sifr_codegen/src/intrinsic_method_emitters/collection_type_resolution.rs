use super::{HirExpr, RustEmitter, Type};

impl RustEmitter {
    pub(crate) fn effective_method_object_ty(&self, object: &HirExpr) -> Type {
        self.effective_collection_expr_ty(object)
    }

    pub(crate) fn effective_registry_expr_ty(&self, expr: &HirExpr) -> Type {
        self.effective_collection_expr_ty(expr)
    }

    fn effective_collection_expr_ty(&self, expr: &HirExpr) -> Type {
        if let HirExpr::Index {
            object, index, ty, ..
        } = expr
            && self.has_checked_place_read_witness(object, index)
            && let Some(inner) = ty.optional_member_type()
        {
            return inner;
        }
        if let HirExpr::Name { name, ty, .. } = expr {
            if self.option_unwrapped_vars.contains(name)
                && let Some(inner) = ty.optional_member_type()
            {
                return inner;
            }
            if self.none_widened_local_bindings.contains(name) {
                if let Some(bound_ty) = self.local_binding_types.get(name) {
                    return bound_ty.clone();
                }
            }
            if matches!(
                crate::resolve_alias_type_for_plain_call(ty),
                Type::Any | Type::Unknown
            ) {
                if let Some(bound_ty) = self.local_binding_types.get(name) {
                    return bound_ty.clone();
                }
            }
        }
        expr.ty().clone()
    }
}
