use super::{
    CheckedDictReadGuard, RustEmitter, RustStmt, Type, checked_place_expr_token,
    checked_place_read_key, condition_excludes_checked_sequence_read,
    condition_only_excludes_checked_sequence_read, condition_supports_checked_sequence_read,
    expr_mentions_name,
};

impl RustEmitter {
    fn checked_place_read_is_used(key: &str, stmts: &[crate::HirStmt]) -> bool {
        let mut used = false;
        crate::hir_analysis::traversal::walk_stmts(
            stmts,
            crate::hir_analysis::traversal::TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut |_| {},
            &mut |expr| {
                let crate::HirExpr::Index { object, index, .. } = expr else {
                    return;
                };
                if checked_place_read_key(object, index).as_deref() == Some(key) {
                    used = true;
                }
            },
        );
        used
    }

    fn checked_place_witnesses_affected_by_stmts(
        &self,
        stmts: &[crate::HirStmt],
        require_missing_body: bool,
    ) -> Vec<(String, super::CheckedPlaceReadWitness)> {
        let mutated =
            crate::hir_analysis::queries::collect_mutated_vars(stmts, Some(&self.func_signatures));
        let mut affected = self
            .checked_place_read_witnesses
            .iter()
            .filter(|(_, witness)| !require_missing_body || witness.missing.is_some())
            .filter(|(_, witness)| {
                witness
                    .dependencies
                    .iter()
                    .any(|dependency| mutated.contains(dependency))
            })
            .map(|(key, witness)| (key.clone(), witness.clone()))
            .collect::<Vec<_>>();
        affected.sort_by_key(|(_, witness)| witness.order);
        affected
    }

    fn checked_place_witnesses_affected_by_stmt(
        &self,
        stmt: &crate::HirStmt,
        require_missing_body: bool,
    ) -> Vec<(String, super::CheckedPlaceReadWitness)> {
        self.checked_place_witnesses_affected_by_stmts(
            std::slice::from_ref(stmt),
            require_missing_body,
        )
    }

    pub(crate) fn checked_place_loop_condition_refreshes_for_ir(
        &self,
        condition: &crate::HirExpr,
        body: &[crate::HirStmt],
        missing: &RustStmt,
    ) -> (Vec<String>, Vec<RustStmt>) {
        let condition_reads =
            crate::hir_analysis::queries::collection_reads_in_condition(condition)
                .into_iter()
                .filter_map(|read| {
                    let crate::HirExpr::Index { object, index, .. } = read else {
                        return None;
                    };
                    checked_place_read_key(&object, &index)
                })
                .collect::<std::collections::BTreeSet<_>>();
        let refreshed = self
            .checked_place_witnesses_affected_by_stmts(body, false)
            .into_iter()
            .filter(|(key, _)| condition_reads.contains(key))
            .collect::<Vec<_>>();
        let keys = refreshed.iter().map(|(key, _)| key.clone()).collect();
        let guards = refreshed
            .into_iter()
            .map(|(_, witness)| RustStmt::LetElse {
                pattern: format!("Some({})", witness.binding),
                value: witness.option,
                else_body: vec![missing.clone()],
            })
            .collect();
        (keys, guards)
    }

    pub(crate) fn checked_place_while_stmt_for_ir(
        condition: crate::RustExpr,
        body: Vec<RustStmt>,
        mut condition_refreshes: Vec<RustStmt>,
    ) -> RustStmt {
        if condition_refreshes.is_empty() {
            return RustStmt::While {
                cond: condition,
                body,
            };
        }
        condition_refreshes.push(RustStmt::If {
            cond: crate::RustExpr::UnaryOp {
                op: "!".to_string(),
                operand: Box::new(crate::RustExpr::Paren(Box::new(condition))),
            },
            then_body: vec![RustStmt::Break],
            else_body: None,
        });
        condition_refreshes.extend(body);
        RustStmt::Loop {
            body: condition_refreshes,
        }
    }

    pub(crate) fn refresh_checked_place_witnesses_after_emitted_stmt(
        &mut self,
        stmt: &crate::HirStmt,
        following: Option<&[crate::HirStmt]>,
    ) -> Vec<RustStmt> {
        let affected = self.checked_place_witnesses_affected_by_stmt(stmt, true);
        let mut refreshes = Vec::new();
        for (key, witness) in affected {
            self.checked_place_read_witnesses.remove(&key);
            if !following.is_some_and(|tail| Self::checked_place_read_is_used(&key, tail)) {
                continue;
            }
            let Some(missing) = witness.missing.clone() else {
                continue;
            };
            self.checked_place_read_witnesses
                .insert(key, witness.clone());
            refreshes.push(RustStmt::LetElse {
                pattern: format!("Some({})", witness.binding),
                value: witness.option,
                else_body: missing,
            });
        }
        refreshes
    }

    pub(crate) fn try_lower_checked_place_mutation_tail_for_ir(
        &mut self,
        stmt: &crate::HirStmt,
        following: &[crate::HirStmt],
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        if self.checked_place_refresh_suppressed_depth == Some(self.stmt_block_depth)
            || self.checked_place_read_witnesses.is_empty()
        {
            return Ok(None);
        }

        let affected = self.checked_place_witnesses_affected_by_stmt(stmt, false);
        if affected.is_empty() {
            return Ok(None);
        }

        let previous_suppressed_depth = self.checked_place_refresh_suppressed_depth;
        self.checked_place_refresh_suppressed_depth = Some(self.stmt_block_depth + 1);
        let lowered_stmt = self.try_lower_stmt_block_for_ir(std::slice::from_ref(stmt));
        self.checked_place_refresh_suppressed_depth = previous_suppressed_depth;
        let Some(mut lowered) = lowered_stmt? else {
            return Err(crate::CodegenError::new(
                "codegen invariant violated: mutation under a checked-place witness was not structurally lowered",
            ));
        };

        for (key, _) in &affected {
            self.checked_place_read_witnesses.remove(key);
        }
        let refreshed = affected
            .into_iter()
            .filter(|(key, _)| Self::checked_place_read_is_used(key, following))
            .collect::<Vec<_>>();
        for (key, witness) in &refreshed {
            self.checked_place_read_witnesses
                .insert(key.clone(), witness.clone());
        }
        let Some(mut tail) = self.try_lower_stmt_block_for_ir(following)? else {
            return Err(crate::CodegenError::new(
                "codegen invariant violated: tail after checked-place witness refresh was not structurally lowered",
            ));
        };
        for (_, witness) in refreshed.into_iter().rev() {
            tail = if let Some(missing) = witness.missing {
                let mut guarded = vec![RustStmt::LetElse {
                    pattern: format!("Some({})", witness.binding),
                    value: witness.option,
                    else_body: missing,
                }];
                guarded.extend(tail);
                guarded
            } else {
                vec![RustStmt::IfLet {
                    pattern: format!("Some({})", witness.binding),
                    expr: witness.option,
                    then_body: tail,
                    else_body: None,
                }]
            };
        }
        lowered.extend(tail);
        Ok(Some(lowered))
    }

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
        already_refreshed: &[String],
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let guard_keys = guards
            .iter()
            .map(|guard| guard.key.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        let loop_carried_candidates = self
            .checked_place_witnesses_affected_by_stmts(body, false)
            .into_iter()
            .filter(|(key, _)| {
                !guard_keys.contains(key.as_str())
                    && !already_refreshed.contains(key)
                    && Self::checked_place_read_is_used(key, body)
            })
            .collect::<Vec<_>>();
        let previous = guards
            .iter()
            .map(|guard| {
                (
                    guard.key.clone(),
                    self.checked_place_read_witnesses.insert(
                        guard.key.clone(),
                        guard.witness_with_missing(vec![missing.clone()]),
                    ),
                )
            })
            .collect::<Vec<_>>();
        let parent_witness_uses = self
            .checked_place_read_witness_uses
            .replace(Some(std::collections::HashSet::new()));
        let lowered = self.try_lower_scoped_stmt_block_for_ir(body);
        let local_witness_uses = self
            .checked_place_read_witness_uses
            .replace(parent_witness_uses)
            .unwrap_or_default();
        for (key, previous_binding) in previous {
            if let Some(binding) = previous_binding {
                self.checked_place_read_witnesses.insert(key, binding);
            } else {
                self.checked_place_read_witnesses.remove(&key);
            }
        }
        let Some(mut lowered) = lowered? else {
            return Ok(None);
        };
        let loop_carried = loop_carried_candidates
            .into_iter()
            .filter(|(key, _)| local_witness_uses.contains(key))
            .collect::<Vec<_>>();
        let used_guard_keys = guards
            .iter()
            .filter(|guard| local_witness_uses.contains(&guard.key))
            .map(|guard| guard.key.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        let locally_satisfied = loop_carried
            .iter()
            .map(|(key, _)| key.as_str())
            .chain(used_guard_keys.iter().copied())
            .collect::<std::collections::BTreeSet<_>>();
        if let Some(parent_uses) = self.checked_place_read_witness_uses.borrow_mut().as_mut() {
            parent_uses.extend(
                local_witness_uses
                    .iter()
                    .filter(|key| !locally_satisfied.contains(key.as_str()))
                    .cloned(),
            );
        }
        drop(locally_satisfied);
        for (_, witness) in loop_carried.into_iter().rev() {
            let mut guarded = vec![RustStmt::LetElse {
                pattern: format!("Some({})", witness.binding),
                value: witness.option,
                else_body: vec![missing.clone()],
            }];
            guarded.extend(lowered);
            lowered = guarded;
        }
        for guard in guards
            .iter()
            .filter(|guard| used_guard_keys.contains(guard.key.as_str()))
            .rev()
        {
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
        self.checked_place_read_witnesses.insert(
            guard.key.clone(),
            guard.witness_with_missing(absent_body.clone()),
        );
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
            self.checked_place_read_witnesses.insert(
                guard.key.clone(),
                guard.witness_with_missing(absent_body.clone()),
            );
            lowered.push(RustStmt::LetElse {
                pattern: format!("Some({})", guard.binding),
                value: guard.option,
                else_body: absent_body.clone(),
            });
        }
        Ok(Some(lowered))
    }
}
