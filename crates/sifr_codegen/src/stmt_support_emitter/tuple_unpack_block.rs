use super::{HirStmt, RustEmitter, RustExpr, RustStmt};

impl RustEmitter {
    pub(crate) fn try_lower_tuple_unpack_stmt_for_block(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        if let HirStmt::StarUnpack {
            before,
            star,
            after,
            value,
            failure,
        } = stmt
        {
            return self.lower_star_unpack_stmt_for_block(
                before,
                star,
                after,
                value,
                failure.as_ref(),
            );
        }
        let HirStmt::TupleUnpack { targets, value } = stmt else {
            return Ok(None);
        };
        let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };
        let source_is_borrowed = crate::tuple_unpack_source_is_borrowed(
            value,
            &self.borrowed_params,
            &self.mut_borrowed_params,
        );
        let mut lowered = crate::lower_tuple_unpack_targets(
            targets,
            value,
            lowered_value,
            &self.mutated_vars,
            source_is_borrowed,
        );
        for target in targets {
            let sifr_ir::HirTupleTargetBinding::Name(name) = &target.binding else {
                continue;
            };
            let cache_stmt = if target.rebind_existing {
                self.string_char_cache_rebuild_stmt_for_local(name)
            } else {
                self.force_string_char_cache_init_stmt_for_local(name, &target.ty)
            };
            if let Some(cache_stmt) = cache_stmt {
                lowered.push(cache_stmt);
            }
        }
        Ok(Some(lowered))
    }

    fn lower_star_unpack_stmt_for_block(
        &mut self,
        before: &[sifr_ir::HirTupleTarget],
        star: &sifr_ir::HirTupleTarget,
        after: &[sifr_ir::HirTupleTarget],
        value: &crate::HirExpr,
        failure: Option<&crate::Type>,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let Some(failure) = failure else {
            return Err(crate::CodegenError::new(
                "star unpack reached code generation without a typed cardinality failure",
            ));
        };
        let Some(mut lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };
        if matches!(value, crate::HirExpr::FieldAccess { .. }) {
            if let Some(storage) = self.emit_storage_path(value) {
                lowered_value = RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(storage),
                };
            }
        } else if matches!(value, crate::HirExpr::Name { .. })
            && Self::rust_expr_is_reusable_place_for_ir(&lowered_value)
        {
            lowered_value = RustExpr::Ref {
                mutable: false,
                expr: Box::new(lowered_value),
            };
        }
        let mut pattern_parts = before
            .iter()
            .enumerate()
            .map(|(index, _)| format!("__sifr_before_{index}"))
            .collect::<Vec<_>>();
        pattern_parts.push("__sifr_star @ ..".to_string());
        pattern_parts.extend(
            after
                .iter()
                .enumerate()
                .map(|(index, _)| format!("__sifr_after_{index}")),
        );
        let mut lowered = vec![
            RustStmt::Let {
                mutable: false,
                name: "__sifr_unpack_source".to_string(),
                ty: None,
                value: lowered_value,
            },
            RustStmt::LetElse {
                pattern: format!("[{}]", pattern_parts.join(", ")),
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__sifr_unpack_source".to_string())),
                    method: "as_slice".to_string(),
                    args: Vec::new(),
                },
                else_body: vec![self.checked_place_failure_return(
                    failure,
                    crate::checked_place::CheckedPlaceFailureKind::Unpack,
                )],
            },
        ];
        for (index, target) in before.iter().enumerate() {
            lowered.push(lower_star_unpack_target(
                target,
                clone_or_copy_ref(format!("__sifr_before_{index}"), &target.ty),
                &self.mutated_vars,
            )?);
        }
        lowered.push(lower_star_unpack_target(
            star,
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__sifr_star".to_string())),
                method: "to_vec".to_string(),
                args: Vec::new(),
            },
            &self.mutated_vars,
        )?);
        for (index, target) in after.iter().enumerate() {
            lowered.push(lower_star_unpack_target(
                target,
                clone_or_copy_ref(format!("__sifr_after_{index}"), &target.ty),
                &self.mutated_vars,
            )?);
        }
        for target in before.iter().chain(std::iter::once(star)).chain(after) {
            let sifr_ir::HirTupleTargetBinding::Name(name) = &target.binding else {
                return Err(crate::CodegenError::new(
                    "star unpack reached code generation with a non-name target",
                ));
            };
            let cache_stmt = if target.rebind_existing {
                self.string_char_cache_rebuild_stmt_for_local(name)
            } else {
                self.force_string_char_cache_init_stmt_for_local(name, &target.ty)
            };
            if let Some(cache_stmt) = cache_stmt {
                lowered.push(cache_stmt);
            }
        }
        Ok(Some(lowered))
    }
}

fn lower_star_unpack_target(
    target: &sifr_ir::HirTupleTarget,
    value: RustExpr,
    mutated_vars: &std::collections::HashSet<String>,
) -> Result<RustStmt, crate::CodegenError> {
    let sifr_ir::HirTupleTargetBinding::Name(name) = &target.binding else {
        return Err(crate::CodegenError::new(
            "star unpack reached code generation with a non-name target",
        ));
    };
    if target.rebind_existing {
        Ok(RustStmt::Assign {
            target: RustExpr::Ident(name.clone()),
            value,
        })
    } else {
        Ok(RustStmt::Let {
            mutable: mutated_vars.contains(name),
            name: name.clone(),
            ty: None,
            value,
        })
    }
}

fn clone_or_copy_ref(name: String, ty: &crate::Type) -> RustExpr {
    let ident = RustExpr::Ident(name);
    if crate::helpers::is_copy_type_for_codegen(ty) {
        RustExpr::Deref(Box::new(ident))
    } else {
        RustExpr::Clone(Box::new(ident))
    }
}
