use super::{
    CheckedDictReadGuard, RustEmitter, RustStmt, Type, checked_place_expr_token,
    checked_place_read_key, condition_excludes_checked_sequence_read,
    condition_only_excludes_checked_sequence_read, condition_supports_checked_sequence_read,
    expr_mentions_name,
};

impl RustEmitter {
    fn lower_checked_read_guards_branch(
        &mut self,
        body: &[crate::HirStmt],
        guards: &[CheckedDictReadGuard],
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let previous = guards
            .iter()
            .map(|guard| {
                (
                    guard.key.clone(),
                    self.checked_place_read_witnesses
                        .insert(guard.key.clone(), guard.witness()),
                )
            })
            .collect::<Vec<_>>();
        let lowered = self.try_lower_scoped_stmt_block_for_ir(body);
        for (key, previous_binding) in previous {
            if let Some(binding) = previous_binding {
                self.checked_place_read_witnesses.insert(key, binding);
            } else {
                self.checked_place_read_witnesses.remove(&key);
            }
        }
        lowered
    }

    fn lower_checked_read_branch(
        &mut self,
        body: &[crate::HirStmt],
        guard: &CheckedDictReadGuard,
        has_witness: bool,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let previous = has_witness.then(|| {
            self.checked_place_read_witnesses
                .insert(guard.key.clone(), guard.witness())
        });
        let lowered = self.try_lower_scoped_stmt_block_for_ir(body);
        if let Some(previous) = previous {
            match previous {
                Some(binding) => {
                    self.checked_place_read_witnesses
                        .insert(guard.key.clone(), binding);
                }
                None => {
                    self.checked_place_read_witnesses.remove(&guard.key);
                }
            }
        }
        lowered
    }

    pub(crate) fn try_lower_checked_dict_if_for_ir(
        &mut self,
        condition: &crate::HirExpr,
        then_body: &[crate::HirStmt],
        elif_clauses: &[(crate::HirExpr, Vec<crate::HirStmt>)],
        else_body: Option<&[crate::HirStmt]>,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        if !elif_clauses.is_empty() {
            return Ok(None);
        }
        let Some(guard) = self.checked_dict_read_guard_for_ir(condition)? else {
            return Ok(None);
        };
        let empty = Vec::new();
        let else_body = else_body.unwrap_or(&empty);
        let present_hir = if guard.negated { else_body } else { then_body };
        let absent_hir = if guard.negated { then_body } else { else_body };
        let consumes_witness = crate::hir_analysis::queries::proven_collection_reads(present_hir)
            .iter()
            .any(|read| {
                matches!(
                    read,
                    crate::HirExpr::Index { object, index, .. }
                        if checked_place_read_key(object, index).as_ref() == Some(&guard.key)
                )
            });
        if !consumes_witness {
            return Ok(None);
        }
        let Some(present) = self.lower_checked_read_branch(present_hir, &guard, true)? else {
            return Ok(None);
        };
        let Some(absent) = self.lower_checked_read_branch(absent_hir, &guard, false)? else {
            return Ok(None);
        };
        Ok(Some(RustStmt::IfLet {
            pattern: format!("Some({})", guard.binding),
            expr: guard.option,
            then_body: present,
            else_body: (!absent.is_empty()).then_some(absent),
        }))
    }

    pub(crate) fn try_lower_checked_sequence_if_for_ir(
        &mut self,
        condition: &crate::HirExpr,
        then_body: &[crate::HirStmt],
        elif_clauses: &[(crate::HirExpr, Vec<crate::HirStmt>)],
        else_body: Option<&[crate::HirStmt]>,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        if !elif_clauses.is_empty() {
            return Ok(None);
        }
        let negated = matches!(condition, crate::HirExpr::UnaryOp { op, .. } if op == "not");
        let empty = Vec::new();
        let else_body = else_body.unwrap_or(&empty);
        let present_hir = if negated { else_body } else { then_body };
        let absent_hir = if negated { then_body } else { else_body };
        let mut guards = Vec::new();
        for read in crate::hir_analysis::queries::proven_collection_reads(present_hir) {
            let crate::HirExpr::Index { object, index, .. } = &read else {
                continue;
            };
            if matches!(object.ty().resolve_alias(), Type::Dict(_, _))
                || !condition_supports_checked_sequence_read(condition, object, index)
            {
                continue;
            }
            let Some(guard) = self.checked_sequence_read_guard_for_ir(&read)? else {
                continue;
            };
            if guards
                .iter()
                .any(|existing: &CheckedDictReadGuard| existing.key == guard.key)
            {
                continue;
            }
            guards.push(guard);
        }
        if guards.is_empty() {
            return Ok(None);
        }
        let Some(mut present) = self.lower_checked_read_guards_branch(present_hir, &guards)? else {
            return Ok(None);
        };
        for guard in guards.into_iter().rev() {
            present = vec![RustStmt::IfLet {
                pattern: format!("Some({})", guard.binding),
                expr: guard.option,
                then_body: present,
                else_body: None,
            }];
        }
        let Some(absent) = self.try_lower_scoped_stmt_block_for_ir(absent_hir)? else {
            return Ok(None);
        };
        let Some(lowered_condition) = self.lower_condition_expr_for_ir(condition)? else {
            return Ok(None);
        };
        let (then_body, else_body) = if negated {
            (absent, Some(present))
        } else {
            (present, (!absent.is_empty()).then_some(absent))
        };
        Ok(Some(RustStmt::If {
            cond: lowered_condition,
            then_body,
            else_body,
        }))
    }

    pub(crate) fn checked_sequence_loop_guards_for_ir(
        &mut self,
        condition: &crate::HirExpr,
        body: &[crate::HirStmt],
    ) -> Result<Vec<CheckedDictReadGuard>, crate::CodegenError> {
        let mut guards = Vec::new();
        for read in crate::hir_analysis::queries::proven_collection_reads(body) {
            let crate::HirExpr::Index { object, index, .. } = &read else {
                continue;
            };
            if matches!(object.ty().resolve_alias(), Type::Dict(_, _))
                || !condition_supports_checked_sequence_read(condition, object, index)
            {
                continue;
            }
            let Some(guard) = self.checked_sequence_read_guard_for_ir(&read)? else {
                continue;
            };
            if guards
                .iter()
                .any(|existing: &CheckedDictReadGuard| existing.key == guard.key)
            {
                continue;
            }
            guards.push(guard);
        }
        Ok(guards)
    }

    pub(crate) fn lower_checked_sequence_loop_body_for_ir(
        &mut self,
        body: &[crate::HirStmt],
        guards: &[CheckedDictReadGuard],
        missing: &RustStmt,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let Some(mut lowered) = self.lower_checked_read_guards_branch(body, guards)? else {
            return Ok(None);
        };
        for guard in guards.iter().rev() {
            lowered.insert(
                0,
                RustStmt::LetElse {
                    pattern: format!("Some({})", guard.binding),
                    value: guard.option.clone(),
                    else_body: vec![missing.clone()],
                },
            );
        }
        Ok(Some(lowered))
    }

    pub(crate) fn checked_sequence_for_guards_for_ir(
        &mut self,
        target: &str,
        iter: &crate::HirExpr,
        body: &[crate::HirStmt],
    ) -> Result<Vec<CheckedDictReadGuard>, crate::CodegenError> {
        let iter = match iter {
            crate::HirExpr::IteratorCall { op, args, .. }
                if matches!(op, sifr_ir::HirIteratorOp::Iter) && args.len() == 1 =>
            {
                &args[0]
            }
            other => other,
        };
        let crate::HirExpr::RangeLiteral { start, end, .. } = iter else {
            return Ok(Vec::new());
        };
        if !matches!(start.as_ref(), crate::HirExpr::IntLiteral(value) if *value >= 0) {
            return Ok(Vec::new());
        }
        let crate::HirExpr::MethodCall {
            object: range_object,
            method,
            args,
            ..
        } = end.as_ref()
        else {
            return Ok(Vec::new());
        };
        if method != "len" || !args.is_empty() {
            return Ok(Vec::new());
        }
        let range_object_token = checked_place_expr_token(range_object);
        let mut guards = Vec::new();
        for read in crate::hir_analysis::queries::proven_collection_reads(body) {
            let crate::HirExpr::Index { object, index, .. } = &read else {
                continue;
            };
            if checked_place_expr_token(object) != range_object_token
                || !expr_mentions_name(index, target)
            {
                continue;
            }
            let Some(guard) = self.checked_sequence_read_guard_for_ir(&read)? else {
                continue;
            };
            if guards
                .iter()
                .any(|existing: &CheckedDictReadGuard| existing.key == guard.key)
            {
                continue;
            }
            guards.push(guard);
        }
        Ok(guards)
    }

    pub(crate) fn try_lower_checked_dict_exit_guard_for_ir(
        &mut self,
        stmt: &crate::HirStmt,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let crate::HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } = stmt
        else {
            return Ok(None);
        };
        if !elif_clauses.is_empty()
            || else_body.is_some()
            || !crate::hir_analysis::queries::block_control_flow_effect(then_body).always_exits()
        {
            return Ok(None);
        }
        let Some(guard) = self.checked_dict_read_guard_for_ir(condition)? else {
            return Ok(None);
        };
        if !guard.negated {
            return Ok(None);
        }
        let Some(absent_body) = self.lower_checked_read_branch(then_body, &guard, false)? else {
            return Ok(None);
        };
        self.checked_place_read_witnesses
            .insert(guard.key.clone(), guard.witness());
        Ok(Some(RustStmt::LetElse {
            pattern: format!("Some({})", guard.binding),
            value: guard.option,
            else_body: absent_body,
        }))
    }

    pub(crate) fn try_lower_checked_sequence_exit_guards_for_ir(
        &mut self,
        stmt: &crate::HirStmt,
        following_stmts: Option<&[crate::HirStmt]>,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let Some(following_stmts) = following_stmts else {
            return Ok(None);
        };
        let crate::HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } = stmt
        else {
            return Ok(None);
        };
        if !elif_clauses.is_empty()
            || else_body.is_some()
            || !crate::hir_analysis::queries::block_control_flow_effect(then_body).always_exits()
        {
            return Ok(None);
        }
        let reads = crate::hir_analysis::queries::proven_collection_reads(following_stmts);
        let mut guards = Vec::new();
        let mut condition_fully_replaced = true;
        for read in reads {
            let crate::HirExpr::Index { object, index, .. } = &read else {
                continue;
            };
            if matches!(object.ty().resolve_alias(), Type::Dict(_, _))
                || !condition_excludes_checked_sequence_read(condition, object, index)
            {
                continue;
            }
            let Some(guard) = self.checked_sequence_read_guard_for_ir(&read)? else {
                continue;
            };
            if self.checked_place_read_witnesses.contains_key(&guard.key) {
                continue;
            }
            if guards
                .iter()
                .any(|existing: &CheckedDictReadGuard| existing.key == guard.key)
            {
                continue;
            }
            condition_fully_replaced &=
                condition_only_excludes_checked_sequence_read(condition, object, index);
            guards.push(guard);
        }
        if guards.is_empty() {
            return Ok(None);
        }
        let Some(absent_body) = self.try_lower_scoped_stmt_block_for_ir(then_body)? else {
            return Ok(None);
        };
        let mut lowered = Vec::new();
        if !condition_fully_replaced {
            let Some(lowered_condition) = self.lower_condition_expr_for_ir(condition)? else {
                return Ok(None);
            };
            lowered.push(RustStmt::If {
                cond: lowered_condition,
                then_body: absent_body.clone(),
                else_body: None,
            });
        }
        for guard in guards {
            self.checked_place_read_witnesses
                .insert(guard.key.clone(), guard.witness());
            lowered.push(RustStmt::LetElse {
                pattern: format!("Some({})", guard.binding),
                value: guard.option,
                else_body: absent_body.clone(),
            });
        }
        Ok(Some(lowered))
    }
}
