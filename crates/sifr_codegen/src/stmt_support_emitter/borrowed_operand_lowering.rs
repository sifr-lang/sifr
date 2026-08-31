use crate::{RustEmitter, RustExpr};
use sifr_ir::HirExpr;
use sifr_type_system::Type;

impl RustEmitter {
    pub(super) fn comparison_operand_is_shared_borrow(&self, source: &HirExpr) -> bool {
        matches!(
            source,
            HirExpr::Name { name, ty, .. }
                if !crate::helpers::is_copy_type_for_codegen(ty)
                    && (self.borrowed_params.contains(name)
                        || self.mut_borrowed_params.contains(name))
        )
    }

    pub(super) fn borrow_comparison_operand(
        &mut self,
        source: &HirExpr,
        lowered: RustExpr,
        representation_was_wrapped: bool,
    ) -> RustExpr {
        let value = if representation_was_wrapped {
            lowered
        } else {
            self.emit_shared_receiver_path(source).unwrap_or(lowered)
        };
        RustExpr::Ref {
            mutable: false,
            expr: Box::new(value),
        }
    }

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
