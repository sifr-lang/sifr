use super::{
    CheckedPlaceFailureKind, RustEmitter, RustExpr, RustStmt, Type, checked_sequence_get_option,
};

fn accepts_read_failure(ty: &Type, kind: CheckedPlaceFailureKind) -> bool {
    match ty.resolve_alias() {
        Type::Class { identity, name, .. }
            if (name == kind.error_name() || name == "Error")
                && identity
                    .as_deref()
                    .is_none_or(|identity| identity == format!("sifr.builtin.{name}")) =>
        {
            true
        }
        Type::Union(members) => members
            .iter()
            .any(|member| accepts_read_failure(member, kind)),
        _ => false,
    }
}

impl RustEmitter {
    pub(crate) fn checked_read_failure_type(&self, kind: CheckedPlaceFailureKind) -> Option<Type> {
        let carrier = self
            .try_closure_error_type_info
            .last()
            .and_then(Option::as_ref)
            .or_else(
                || match self.current_return_type.as_ref()?.resolve_alias() {
                    Type::Result(_, error) => Some(error.as_ref()),
                    _ => None,
                },
            )?;
        accepts_read_failure(carrier, kind).then(|| Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: kind.error_name().to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        })
    }

    /// Keep a proven read at its expression position. The existing typed error
    /// carrier supplies the checked failure path; no loop-wide read is hoisted.
    pub(crate) fn lower_proven_read_with_error_carrier(
        &mut self,
        object: &crate::HirExpr,
        index: &crate::HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let kind = if matches!(object.ty().resolve_alias(), Type::Dict(_, _)) {
            CheckedPlaceFailureKind::Key
        } else {
            CheckedPlaceFailureKind::Index
        };
        let Some(failure) = self.checked_read_failure_type(kind) else {
            return Ok(None);
        };
        if !matches!(
            object.ty().resolve_alias(),
            Type::List(_) | Type::Bytes | Type::Str | Type::Dict(_, _)
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
        let option = if matches!(object.ty().resolve_alias(), Type::Dict(_, _)) {
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(lowered_object),
                    method: "get".to_string(),
                    args: vec![self.checked_dict_key_arg_for_ir(index, lowered_index)],
                }),
                method: "cloned".to_string(),
                args: Vec::new(),
            }
        } else if matches!(object.ty().resolve_alias(), Type::Str) {
            self.lower_string_index_option_with_cache(object, lowered_object, lowered_index)
        } else {
            checked_sequence_get_option(lowered_object, false, lowered_index, "__sifr_proven_read")
        };
        let (_, binding) = self.next_checked_place_read_binding();
        Ok(Some(RustExpr::Block {
            stmts: vec![RustStmt::LetElse {
                pattern: format!("Some({binding})"),
                value: option,
                else_body: vec![self.checked_place_failure_return(&failure, kind)],
            }],
            expr: Some(Box::new(RustExpr::Ident(binding))),
        }))
    }
}
