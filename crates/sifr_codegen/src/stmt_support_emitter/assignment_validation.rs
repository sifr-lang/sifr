use super::{HirExpr, RustEmitter, Type};

impl RustEmitter {
    pub(crate) fn validate_assignment_source_type_for_ir(
        name: &str,
        target_ty: &Type,
        value: &HirExpr,
    ) -> Result<(), crate::CodegenError> {
        if !crate::helpers::is_option_type(target_ty)
            && crate::helpers::is_option_type(value.ty())
            && !value.ty().is_assignable_to(target_ty)
        {
            return Err(crate::CodegenError::new(format!(
                "codegen invariant violated: optional value reached assignment to non-optional local `{name}`"
            )));
        }
        Ok(())
    }
}
