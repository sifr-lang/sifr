use super::{
    CheckedDictReadGuard, CheckedPlaceFailureKind, RustEmitter, RustStmt, Type,
    checked_place_expr_token, checked_place_read_key, condition_excludes_checked_sequence_read,
    condition_only_excludes_checked_sequence_read, condition_supports_checked_sequence_read,
    expr_mentions_name,
};

impl RustEmitter {
    fn checked_place_read_is_used(&self, key: &str, stmts: &[crate::HirStmt]) -> bool {
        self.body_analysis.checked_read_is_used(stmts, key)
    }

    fn checked_place_witness_is_invalidated_by_stmt(
        witness: &super::CheckedPlaceReadWitness,
        stmt: &crate::HirStmt,
    ) -> bool {
        let mut invalidated = false;
        crate::hir_analysis::traversal::walk_stmts(
            std::slice::from_ref(stmt),
            crate::hir_analysis::traversal::TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut |_| {},
            &mut |expr| {
                let (args, mutable_arg_places) = match expr {
                    crate::HirExpr::Call {
                        args,
                        mutable_arg_places,
                        ..
                    }
                    | crate::HirExpr::GenericCall {
                        args,
                        mutable_arg_places,
                        ..
                    }
                    | crate::HirExpr::MethodCall {
                        args,
                        mutable_arg_places,
                        ..
                    } => (args, mutable_arg_places),
                    _ => return,
                };
                for (arg, target) in args.iter().zip(mutable_arg_places) {
                    if target.is_none() {
                        continue;
                    }
                    crate::hir_analysis::traversal::walk_expr(arg, &mut |candidate| {
                        if matches!(candidate, crate::HirExpr::Name { name, .. }
                            if witness.dependencies.contains(name))
                        {
                            invalidated = true;
                        }
                    });
                }
            },
        );
        invalidated
    }

    fn checked_place_refresh_precondition_holds(
        key: &str,
        witness: &super::CheckedPlaceReadWitness,
        stmt: &crate::HirStmt,
    ) -> bool {
        let mut statements_preserve_presence = true;
        let mut expressions_preserve_presence = true;
        crate::hir_analysis::traversal::walk_stmts(
            std::slice::from_ref(stmt),
            crate::hir_analysis::traversal::TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut |stmt| match stmt {
                crate::HirStmt::Assign { name, .. } | crate::HirStmt::AugAssign { name, .. }
                    if witness.dependencies.contains(name) =>
                {
                    statements_preserve_presence = false;
                }
                crate::HirStmt::FieldAssign { object, .. }
                | crate::HirStmt::NestedFieldAssign { object, .. }
                | crate::HirStmt::AttributeAugAssign { object, .. }
                    if witness.dependencies.contains(object) =>
                {
                    statements_preserve_presence = false;
                }
                crate::HirStmt::Delete { object, .. }
                    if witness
                        .dependencies
                        .iter()
                        .any(|name| expr_mentions_name(object, name)) =>
                {
                    statements_preserve_presence = false;
                }
                _ => {}
            },
            &mut |expr| {
                let (args, mutable_arg_places) = match expr {
                    crate::HirExpr::Call {
                        args,
                        mutable_arg_places,
                        ..
                    }
                    | crate::HirExpr::GenericCall {
                        args,
                        mutable_arg_places,
                        ..
                    } => (args, mutable_arg_places),
                    crate::HirExpr::MethodCall {
                        object,
                        method,
                        args,
                        receiver_convention,
                        mutable_arg_places,
                        ..
                    } => {
                        let mutates_dependency = matches!(
                            receiver_convention,
                            Some(sifr_type_system::ReceiverConvention::MutableBorrow)
                        ) && witness
                            .dependencies
                            .iter()
                            .any(|name| expr_mentions_name(object, name));
                        let mutates_value_inside_proven_place = checked_place_expr_token(object)
                            .is_some_and(|token| token.strip_prefix("index:") == Some(key));
                        let preserves_proven_place = mutates_value_inside_proven_place
                            || !matches!(
                                sifr_type_system::receiver_mutation_summary(
                                    object.ty(),
                                    method,
                                    receiver_convention.unwrap_or(
                                        sifr_type_system::ReceiverConvention::SharedBorrow,
                                    ),
                                )
                                .effect,
                                sifr_type_system::ReceiverMutationEffect::Removal
                            );
                        if mutates_dependency && !preserves_proven_place {
                            expressions_preserve_presence = false;
                        }
                        (args, mutable_arg_places)
                    }
                    _ => return,
                };
                for (argument, target) in args.iter().zip(mutable_arg_places) {
                    if target.is_some()
                        && witness
                            .dependencies
                            .iter()
                            .any(|name| expr_mentions_name(argument, name))
                    {
                        expressions_preserve_presence = false;
                    }
                }
            },
        );
        statements_preserve_presence && expressions_preserve_presence
    }

    fn checked_place_witnesses_affected_by_stmts(
        &self,
        stmts: &[crate::HirStmt],
    ) -> Vec<(String, super::CheckedPlaceReadWitness)> {
        let mutated = self.body_analysis.mutated_in(stmts);
        let mut affected = self
            .checked_place_read_witnesses
            .iter()
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
    ) -> Vec<(String, super::CheckedPlaceReadWitness)> {
        self.checked_place_witnesses_affected_by_stmts(std::slice::from_ref(stmt))
    }

    pub(crate) fn prepare_checked_place_witnesses_for_mutation(
        &mut self,
        stmt: &crate::HirStmt,
        following: Option<&[crate::HirStmt]>,
    ) -> Vec<RustStmt> {
        let Some(following) = following else {
            return Vec::new();
        };
        let mut preparations = Vec::new();
        for (key, mut witness) in self.checked_place_witnesses_affected_by_stmt(stmt) {
            if !witness.borrowed
                || Self::checked_place_witness_is_invalidated_by_stmt(&witness, stmt)
                || !self.checked_place_read_is_used(&key, following)
            {
                continue;
            }
            preparations.push(RustStmt::Let {
                mutable: false,
                name: witness.binding.clone(),
                ty: None,
                value: crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(crate::RustExpr::Deref(
                        Box::new(crate::RustExpr::Ident(witness.binding.clone())),
                    )))),
                    method: "clone".to_string(),
                    args: Vec::new(),
                },
            });
            witness.borrowed = false;
            witness.option = crate::RustExpr::MethodCall {
                receiver: Box::new(witness.option),
                method: "cloned".to_string(),
                args: Vec::new(),
            };
            self.checked_place_read_witnesses.insert(key, witness);
        }
        preparations
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
            .checked_place_witnesses_affected_by_stmts(body)
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
    ) -> Result<Vec<RustStmt>, crate::CodegenError> {
        let affected = self.checked_place_witnesses_affected_by_stmt(stmt);
        let mut refreshes = Vec::new();
        for (key, witness) in affected {
            self.checked_place_read_witnesses.remove(&key);
            if Self::checked_place_witness_is_invalidated_by_stmt(&witness, stmt)
                || !following.is_some_and(|tail| self.checked_place_read_is_used(&key, tail))
            {
                continue;
            }
            if !Self::checked_place_refresh_precondition_holds(&key, &witness, stmt) {
                return Err(crate::CodegenError::new(
                    "codegen invariant violated: checked-place refresh fallback reached a mutation that may remove the proven place",
                ));
            }
            self.checked_place_read_witnesses
                .insert(key, witness.clone());
            refreshes.push(RustStmt::Let {
                mutable: false,
                name: witness.binding.clone(),
                ty: None,
                value: crate::RustExpr::MethodCall {
                    receiver: Box::new(witness.option),
                    method: "unwrap_or".to_string(),
                    args: vec![crate::RustExpr::Ident(witness.binding)],
                },
            });
        }
        Ok(refreshes)
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

        let mut preparations =
            self.prepare_checked_place_witnesses_for_mutation(stmt, Some(following));
        let affected = self.checked_place_witnesses_affected_by_stmt(stmt);
        if affected.is_empty() {
            return Ok(None);
        }

        let previous_suppressed_depth = self.checked_place_refresh_suppressed_depth;
        self.checked_place_refresh_suppressed_depth = Some(self.stmt_block_depth + 1);
        let lowered_stmt = self.try_lower_stmt_block_for_ir(std::slice::from_ref(stmt));
        self.checked_place_refresh_suppressed_depth = previous_suppressed_depth;
        let Some(lowered_stmt) = lowered_stmt? else {
            return Err(crate::CodegenError::new(
                "codegen invariant violated: mutation under a checked-place witness was not structurally lowered",
            ));
        };
        preparations.extend(lowered_stmt);
        let mut lowered = preparations;

        for (key, _) in &affected {
            self.checked_place_read_witnesses.remove(key);
        }
        let mut refreshed = Vec::new();
        for (key, witness) in affected {
            if Self::checked_place_witness_is_invalidated_by_stmt(&witness, stmt)
                || !self.checked_place_read_is_used(&key, following)
            {
                continue;
            }
            if !Self::checked_place_refresh_precondition_holds(&key, &witness, stmt) {
                return Err(crate::CodegenError::new(
                    "codegen invariant violated: checked-place refresh fallback reached a mutation that may remove the proven place",
                ));
            }
            refreshed.push((key, witness));
        }
        for (key, witness) in &refreshed {
            self.checked_place_read_witnesses
                .insert(key.clone(), witness.clone());
        }
        for (_, witness) in &refreshed {
            lowered.push(RustStmt::Let {
                mutable: false,
                name: witness.binding.clone(),
                ty: None,
                value: crate::RustExpr::MethodCall {
                    receiver: Box::new(witness.option.clone()),
                    method: "unwrap_or".to_string(),
                    args: vec![crate::RustExpr::Ident(witness.binding.clone())],
                },
            });
        }
        let Some(tail) = self.try_lower_stmt_block_for_ir(following)? else {
            return Err(crate::CodegenError::new(
                "codegen invariant violated: tail after checked-place witness refresh was not structurally lowered",
            ));
        };
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
        if self
            .checked_read_failure_type(CheckedPlaceFailureKind::Key)
            .is_some()
            || !elif_clauses.is_empty()
        {
            return Ok(None);
        }
        let Some(guard) = self.checked_dict_read_guard_for_ir(condition)? else {
            return Ok(None);
        };
        let empty = Vec::new();
        let else_body = else_body.unwrap_or(&empty);
        let present_hir = if guard.negated { else_body } else { then_body };
        let absent_hir = if guard.negated { then_body } else { else_body };
        let consumes_witness = self
            .body_analysis
            .proven_reads_in(present_hir)
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
        if self
            .checked_read_failure_type(CheckedPlaceFailureKind::Index)
            .is_some()
            || !elif_clauses.is_empty()
        {
            return Ok(None);
        }
        let negated = matches!(condition, crate::HirExpr::UnaryOp { op, .. } if op == "not");
        let empty = Vec::new();
        let else_body = else_body.unwrap_or(&empty);
        let present_hir = if negated { else_body } else { then_body };
        let absent_hir = if negated { then_body } else { else_body };
        let mut guards = Vec::new();
        for read in self.body_analysis.proven_reads_in(present_hir) {
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
        if self
            .checked_read_failure_type(CheckedPlaceFailureKind::Index)
            .is_some()
        {
            return Ok(Vec::new());
        }
        let mut guards = Vec::new();
        for read in self.body_analysis.proven_reads_in(body) {
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
            .checked_place_witnesses_affected_by_stmts(body)
            .into_iter()
            .filter(|(key, _)| {
                !guard_keys.contains(key.as_str())
                    && !already_refreshed.contains(key)
                    && self.checked_place_read_is_used(key, body)
            })
            .collect::<Vec<_>>();
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
        if self
            .checked_read_failure_type(CheckedPlaceFailureKind::Index)
            .is_some()
        {
            return Ok(Vec::new());
        }
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
        for read in self.body_analysis.proven_reads_in(body) {
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
        if self
            .checked_read_failure_type(CheckedPlaceFailureKind::Key)
            .is_some()
        {
            return Ok(None);
        }
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
        if self
            .checked_read_failure_type(CheckedPlaceFailureKind::Index)
            .is_some()
        {
            return Ok(None);
        }
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
        let reads = self.body_analysis.proven_reads_in(following_stmts);
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

#[cfg(test)]
#[path = "control_flow_tests.rs"]
mod tests;
