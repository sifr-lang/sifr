use super::{RustEmitter, RustExpr, RustStmt, Type, checked_place_read_key};

impl RustEmitter {
    /// Insertion returns a reference to the value whose presence it establishes.
    /// Carry that reference through the ordinary witness invalidation contract;
    /// do not turn a proven read into an optional lookup or a panicking index.
    pub(crate) fn try_lower_dict_assignment_witness_for_ir(
        &mut self,
        stmt: &crate::HirStmt,
        following: Option<&[crate::HirStmt]>,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        if !matches!(stmt, crate::HirStmt::SubscriptAssign { .. }) || following.is_none() {
            return Ok(None);
        }
        let previous = self.checked_place_read_witnesses.clone();
        let result = self.lower_dict_assignment_witness_for_ir(stmt, following);
        if !matches!(&result, Ok(Some(_))) {
            self.checked_place_read_witnesses = previous;
        }
        result
    }

    fn lower_dict_assignment_witness_for_ir(
        &mut self,
        stmt: &crate::HirStmt,
        following: Option<&[crate::HirStmt]>,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let Some(following) = following else {
            return Ok(None);
        };
        let crate::HirStmt::SubscriptAssign {
            object,
            index,
            value,
            object_ty,
            ..
        } = stmt
        else {
            return Ok(None);
        };
        let Type::Dict(_, target_ty) = object_ty.resolve_alias() else {
            return Ok(None);
        };
        if crate::helpers::is_option_type(index.ty()) {
            return Ok(None);
        }
        let source_object = crate::HirExpr::Name {
            name: object.clone(),
            binding_id: None,
            ty: object_ty.clone(),
        };
        let Some(key) = checked_place_read_key(&source_object, index) else {
            return Ok(None);
        };
        let consumes_witness = self
            .body_analysis
            .proven_reads_in(following)
            .iter()
            .any(|read| {
                matches!(read, crate::HirExpr::Index { object, index, .. }
                if checked_place_read_key(object, index).as_ref() == Some(&key))
            });
        if !consumes_witness {
            return Ok(None);
        }
        let read = crate::HirExpr::Index {
            object: Box::new(source_object),
            index: Box::new(index.clone()),
            ty: target_ty.as_ref().clone(),
        };
        let Some(guard) = self.checked_condition_read_guard_for_ir(&read)? else {
            return Ok(None);
        };
        let previous = self.checked_place_read_witnesses.remove(&key);
        let mut lowered = self.prepare_checked_place_witnesses_for_mutation(stmt, Some(following));
        if let Some(previous) = previous {
            self.checked_place_read_witnesses
                .insert(key.clone(), previous);
        }
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };
        let lowered_value = self.materialize_reusable_value_for_ir(value, lowered_value);
        let lowered_value =
            crate::helpers::flatten_option_value_for_target(target_ty, value.ty(), lowered_value);
        // An older witness into this same map may need refreshing after insert.
        // Its shared lookup cannot overlap the insertion's exclusive borrow.
        let owned = self
            .checked_place_witnesses_affected_by_stmt(stmt)
            .iter()
            .any(|(old_key, witness)| {
                old_key != &key
                    && witness.dependencies.contains(object)
                    && self.body_analysis.checked_read_is_used(following, old_key)
            });
        let entry_value = RustExpr::Deref(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(object.clone())),
                    method: "entry".to_string(),
                    args: vec![RustExpr::Ident("__assign_key".to_string())],
                }),
                method: "insert_entry".to_string(),
                args: vec![RustExpr::Ident("__assign_value".to_string())],
            }),
            method: "into_mut".to_string(),
            args: Vec::new(),
        }));
        let entry_value = if owned && !crate::helpers::is_copy_type_for_codegen(target_ty) {
            crate::ownership_plan::materialize_owned_value(
                target_ty,
                RustExpr::Paren(Box::new(entry_value)),
            )
        } else if owned {
            entry_value
        } else {
            RustExpr::Ref {
                mutable: false,
                expr: Box::new(entry_value),
            }
        };
        lowered.push(RustStmt::Let {
            mutable: false,
            name: guard.binding.clone(),
            ty: Some(if owned {
                crate::sifr_type_to_rust_type(target_ty)
            } else {
                crate::RustType::Ref {
                    mutable: false,
                    inner: Box::new(crate::sifr_type_to_rust_type(target_ty)),
                }
            }),
            value: RustExpr::Block {
                stmts: vec![
                    RustStmt::Let {
                        mutable: false,
                        name: "__assign_value".to_string(),
                        ty: None,
                        value: lowered_value,
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__assign_key".to_string(),
                        ty: None,
                        value: Self::clone_non_copy_name_expr_for_ir(index, lowered_index),
                    },
                ],
                expr: Some(Box::new(entry_value)),
            },
        });
        self.checked_place_read_witnesses.remove(&key);
        lowered.extend(
            self.refresh_checked_place_witnesses_after_emitted_stmt(stmt, Some(following))?,
        );
        let mut witness = guard.witness();
        if owned {
            witness.borrowed = false;
            witness.option = RustExpr::MethodCall {
                receiver: Box::new(witness.option),
                method: if witness.copy_value {
                    "copied"
                } else {
                    "cloned"
                }
                .to_string(),
                args: Vec::new(),
            };
        } else {
            witness.exclusive_owner = Some(object.clone());
        }
        self.checked_place_read_witnesses.insert(key, witness);
        Ok(Some(lowered))
    }
}
