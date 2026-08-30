use super::{HirExpr, HirStmt, RustEmitter, RustStmt, Type};
impl RustEmitter {
    pub(crate) fn lower_subscript_augassign_stmt_for_ir(
        &mut self,
        object: &str,
        index: &HirExpr,
        op: &str,
        value: &HirExpr,
        object_ty: &Type,
        failure: Option<&Type>,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        if !matches!(
            op,
            "+=" | "-=" | "*=" | "/=" | "%=" | "//=" | "**=" | "&=" | "|=" | "^=" | "<<=" | ">>="
        ) {
            return Ok(None);
        }
        if matches!(
            object_ty,
            Type::Alias { name: alias_name, .. } if alias_name == "__sifr_defaultdict_int"
        ) {
            let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
                return Ok(None);
            };
            let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
                return Ok(None);
            };
            let lowered_index = self.clone_field_storage_name_expr_for_ir(index, lowered_index);
            let lowered_value = Self::clone_non_copy_name_expr_for_ir(value, lowered_value);
            let lowered_value = self.coerce_typed_expr_to_sifr_int_value(lowered_value, value.ty());
            let Some(lowered_body_stmt) =
                Self::build_subscript_augassign_elem_stmt_for_ir(op, value, lowered_value, true)
            else {
                return Ok(None);
            };
            return Ok(Some(RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__elem".to_string(),
                    ty: None,
                    value: crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident(object.to_string())),
                            method: "entry".to_string(),
                            args: vec![lowered_index],
                        }),
                        method: "or_insert".to_string(),
                        args: vec![crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "SifrInt".to_string(),
                                "from_i64".to_string(),
                            ])),
                            args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                        }],
                    },
                },
                lowered_body_stmt,
            ])));
        }
        self.lower_checked_single_mutation_for_ir(
            crate::RustExpr::Ident(object.to_string()),
            object_ty,
            index,
            value,
            &sifr_ir::HirCollectionMutation::AugAssign(op.to_string()),
            failure,
        )
    }

    pub(crate) fn lower_delete_stmt_for_ir(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
        failure: Option<&Type>,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let lowered_object = if matches!(object, HirExpr::FieldAccess { .. }) {
            self.emit_storage_path(object)
        } else {
            self.lower_stmt_expr_for_ir(object)?
        };
        let Some(lowered_object) = lowered_object else {
            return Ok(None);
        };
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        match Self::resolve_alias_type_for_loop_iter(object.ty()) {
            Type::List(_) => Ok(Some(RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__delete_target".to_string(),
                    ty: None,
                    value: crate::RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(lowered_object),
                    },
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_raw".to_string(),
                    ty: None,
                    value: lowered_index,
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_norm".to_string(),
                    ty: None,
                    value: crate::build_normalized_list_index_i64_expr(
                        crate::RustExpr::Ident("__delete_target".to_string()),
                        "__idx_raw",
                    ),
                },
                RustStmt::If {
                    cond: crate::RustExpr::BinOp {
                        left: Box::new(crate::RustExpr::Ident("__idx_norm".to_string())),
                        op: "<".to_string(),
                        right: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident(
                                "__delete_target".to_string(),
                            )),
                            method: "len".to_string(),
                            args: vec![],
                        }),
                    },
                    then_body: vec![RustStmt::Let {
                        mutable: false,
                        name: "_".to_string(),
                        ty: None,
                        value: crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident(
                                "__delete_target".to_string(),
                            )),
                            method: "remove".to_string(),
                            args: vec![crate::RustExpr::Ident("__idx_norm".to_string())],
                        },
                    }],
                    else_body: failure.map(|failure| {
                        vec![self.checked_place_failure_return(
                            failure,
                            crate::checked_place::CheckedPlaceFailureKind::Index,
                        )]
                    }),
                },
            ]))),
            Type::Dict(_, _) => Ok(Some(RustStmt::IfLet {
                pattern: "Some(_)".to_string(),
                expr: crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_object),
                    method: "remove".to_string(),
                    args: vec![self.build_dict_lookup_key_arg_for_hir(index, lowered_index)],
                },
                then_body: Vec::new(),
                else_body: failure.map(|failure| {
                    vec![self.checked_place_failure_return(
                        failure,
                        crate::checked_place::CheckedPlaceFailureKind::Key,
                    )]
                }),
            })),
            _ => Ok(None),
        }
    }

    pub(crate) fn try_lower_structured_subscript_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::SubscriptAssign {
            object,
            index,
            value,
            object_ty,
            failure,
        } = stmt
        else {
            return Ok(false);
        };

        let Some(lowered) = self.lower_subscript_assign_stmt_for_ir(
            object,
            index,
            value,
            object_ty,
            failure.as_ref(),
        )?
        else {
            return Ok(false);
        };

        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn try_lower_structured_nested_subscript_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::NestedSubscriptAssign {
            object,
            outer_index,
            inner_index,
            value,
            object_ty,
            outer_failure,
            inner_failure,
            operation,
        } = stmt
        else {
            return Ok(false);
        };

        let lowered = self.lower_checked_nested_mutation_for_ir(
            crate::checked_place_mutation::CheckedNestedMutationPlan {
                root: crate::RustExpr::Ident(object.clone()),
                root_ty: object_ty,
                outer_index,
                inner_index,
                value,
                operation,
                outer_failure: outer_failure.as_ref(),
                inner_failure: inner_failure.as_ref(),
            },
        )?;
        let Some(lowered) = lowered else {
            return Ok(false);
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn try_lower_structured_subscript_augassign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::SubscriptAugAssign {
            object,
            index,
            op,
            value,
            object_ty,
            failure,
        } = stmt
        else {
            return Ok(false);
        };
        let Some(lowered) = self.lower_subscript_augassign_stmt_for_ir(
            object,
            index,
            op,
            value,
            object_ty,
            failure.as_ref(),
        )?
        else {
            return Ok(false);
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn try_lower_structured_delete_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::Delete {
            object,
            index,
            failure,
        } = stmt
        else {
            return Ok(false);
        };
        let Some(lowered) = self.lower_delete_stmt_for_ir(object, index, failure.as_ref())? else {
            return Ok(false);
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }
}
