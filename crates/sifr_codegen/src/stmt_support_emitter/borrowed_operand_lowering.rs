use crate::{RustEmitter, RustExpr};
use sifr_ir::HirExpr;
use sifr_type_system::Type;

impl RustEmitter {
    pub(super) fn clone_borrowed_generic_operand(
        &self,
        source: &HirExpr,
        resolved_ty: &Type,
        lowered: RustExpr,
    ) -> RustExpr {
        let is_borrowed_name = matches!(
            source,
            HirExpr::Name { name, .. }
                if self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name)
        );
        if matches!(resolved_ty, Type::TypeVar(_)) && is_borrowed_name {
            RustExpr::Clone(Box::new(lowered))
        } else {
            lowered
        }
    }
}
