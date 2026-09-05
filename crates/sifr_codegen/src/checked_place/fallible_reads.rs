use super::{
    CheckedPlaceFailureKind, RustEmitter, RustExpr, RustStmt, Type, checked_sequence_get_option,
};

fn index_error_member(ty: &Type) -> Option<Type> {
    match ty.resolve_alias() {
        Type::Class { identity, name, .. }
            if name == "IndexError"
                && identity
                    .as_deref()
                    .is_none_or(|identity| identity == "sifr.builtin.IndexError") =>
        {
            Some(ty.clone())
        }
        Type::Union(members) => members.iter().find_map(index_error_member),
        _ => None,
    }
}

impl RustEmitter {
    /// Keep a proven read at its expression position. The existing typed error
    /// carrier supplies the checked failure path; no loop-wide read is hoisted.
    pub(crate) fn lower_proven_read_with_error_carrier(
        &mut self,
        object: &crate::HirExpr,
        index: &crate::HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let carrier = self
            .try_closure_error_type_info
            .last()
            .and_then(Option::as_ref)
            .or_else(
                || match self.current_return_type.as_ref()?.resolve_alias() {
                    Type::Result(_, error) => Some(error.as_ref()),
                    _ => None,
                },
            );
        let Some(failure) = carrier.and_then(index_error_member) else {
            return Ok(None);
        };
        if !matches!(
            object.ty().resolve_alias(),
            Type::List(_) | Type::Bytes | Type::Str
        ) {
            return Ok(None);
        }
        let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
            return Ok(None);
        };
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        let lowered_index = self.materialize_reusable_value_for_ir(index, lowered_index);
        let option = if matches!(object.ty().resolve_alias(), Type::Str) {
            self.lower_string_index_option_with_cache(object, lowered_object, lowered_index)
        } else {
            checked_sequence_get_option(lowered_object, false, lowered_index, "__sifr_proven_read")
        };
        let (_, binding) = self.next_checked_place_read_binding();
        Ok(Some(RustExpr::Block {
            stmts: vec![RustStmt::LetElse {
                pattern: format!("Some({binding})"),
                value: option,
                else_body: vec![
                    self.checked_place_failure_return(&failure, CheckedPlaceFailureKind::Index),
                ],
            }],
            expr: Some(Box::new(RustExpr::Ident(binding))),
        }))
    }
}
