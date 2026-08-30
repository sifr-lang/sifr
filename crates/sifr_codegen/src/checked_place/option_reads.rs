use super::{RustEmitter, RustExpr, Type};

impl RustEmitter {
    fn checked_index_result_needs_flatten_for_target(object_ty: &Type) -> bool {
        match object_ty.resolve_alias() {
            Type::List(element) => element.optional_member_type().is_some(),
            Type::Dict(_, value) => value.optional_member_type().is_some(),
            _ => false,
        }
    }

    fn flatten_checked_index_option_for_target(object_ty: &Type, option: RustExpr) -> RustExpr {
        if Self::checked_index_result_needs_flatten_for_target(object_ty) {
            RustExpr::MethodCall {
                receiver: Box::new(option),
                method: "flatten".to_string(),
                args: Vec::new(),
            }
        } else {
            option
        }
    }

    pub(crate) fn stmt_uses_checked_option_target(&self, stmt: &crate::HirStmt) -> bool {
        match stmt {
            crate::HirStmt::Let { ty, value, .. } => {
                ty.optional_member_type().is_some() && matches!(value, crate::HirExpr::Index { .. })
            }
            crate::HirStmt::Assign { name, value } => {
                self.local_binding_types
                    .get(name)
                    .is_some_and(|ty| ty.optional_member_type().is_some())
                    && matches!(value, crate::HirExpr::Index { .. })
            }
            _ => false,
        }
    }

    /// Lower an index read directly to the optional representation required by
    /// its destination. This preserves the source-level `T | None = values[i]`
    /// contract even when contextual typing records the index expression as
    /// `T` in HIR.
    pub(crate) fn lower_checked_place_option_value_for_target(
        &mut self,
        target_ty: &Type,
        value: &crate::HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        if target_ty.optional_member_type().is_none() {
            return Ok(None);
        }
        let crate::HirExpr::Index { object, index, .. } = value else {
            return Ok(None);
        };

        match object.ty().resolve_alias() {
            Type::List(_) | Type::Bytes | Type::Str => Ok(self
                .checked_sequence_read_guard_for_ir(value)?
                .map(|guard| {
                    Self::flatten_checked_index_option_for_target(object.ty(), guard.option)
                })),
            Type::Dict(_, _) => {
                let lowered_object = if let crate::HirExpr::Index {
                    object: parent,
                    index: parent_index,
                    ..
                } = object.as_ref()
                {
                    let Some(witness) =
                        self.checked_place_read_borrow_witness(parent, parent_index)
                    else {
                        return Ok(None);
                    };
                    witness
                } else if let Some(path) = self.emit_shared_receiver_path(object) {
                    path
                } else if let Some(lowered) = self.lower_stmt_expr_for_ir(object)? {
                    lowered
                } else {
                    return Ok(None);
                };
                let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
                    return Ok(None);
                };
                let option = RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(lowered_object),
                        method: "get".to_string(),
                        args: vec![self.checked_dict_key_arg_for_ir(index, lowered_index)],
                    }),
                    method: "cloned".to_string(),
                    args: Vec::new(),
                };
                Ok(Some(Self::flatten_checked_index_option_for_target(
                    object.ty(),
                    option,
                )))
            }
            _ => Ok(None),
        }
    }
}
