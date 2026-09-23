use super::{
    CheckedDictReadGuard, CheckedPlaceFailureKind, RustEmitter, RustStmt, Type,
    checked_place_expr_token, checked_place_read_key, condition_excludes_checked_sequence_read,
    condition_only_excludes_checked_sequence_read,
};

impl RustEmitter {
    // A prior checked read of sequence[n - 1] proves len(sequence) >= n.
    // On that path, an early `if len(sequence) == n: return ...` is exactly
    // the absent branch for a later read of sequence[n].
    fn existing_prefix_witness_excludes_read(
        &self,
        condition: &crate::HirExpr,
        object: &crate::HirExpr,
        index: &crate::HirExpr,
    ) -> bool {
        let crate::HirExpr::IntLiteral(position) = index else {
            return false;
        };
        let Some(previous) = position.checked_sub(1) else {
            return false;
        };
        if previous < 0 {
            return false;
        }
        let Some(previous_key) =
            checked_place_read_key(object, &crate::HirExpr::IntLiteral(previous))
        else {
            return false;
        };
        if !self
            .checked_place_read_witnesses
            .contains_key(&previous_key)
        {
            return false;
        }
        let Some(object_token) = checked_place_expr_token(object) else {
            return false;
        };
        let crate::HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } = condition
        else {
            return false;
        };
        if ops.as_slice() != ["=="] || comparators.len() != 1 {
            return false;
        }
        let is_length = |candidate: &crate::HirExpr| {
            matches!(candidate, crate::HirExpr::MethodCall { object: length_object, method, args, .. }
                if method == "len" && args.is_empty()
                    && checked_place_expr_token(length_object).as_deref() == Some(object_token.as_str()))
        };
        let is_position = |candidate: &crate::HirExpr| matches!(candidate, crate::HirExpr::IntLiteral(value) if value == position);
        (is_length(left) && is_position(&comparators[0]))
            || (is_position(left) && is_length(&comparators[0]))
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
        let mut reads = self
            .body_analysis
            .proven_reads_in(following_stmts)
            .into_iter()
            .map(|read| (read.clone(), read))
            .collect::<Vec<_>>();
        reads.extend(self.aliased_exit_guard_reads(following_stmts));
        let mut guards = Vec::new();
        let mut condition_fully_replaced = true;
        for (original, read) in reads {
            let crate::HirExpr::Index { object, index, .. } = &read else {
                continue;
            };
            let prefix_proves_exact_length =
                self.existing_prefix_witness_excludes_read(condition, object, index);
            if matches!(object.ty().resolve_alias(), Type::Dict(_, _))
                || !(condition_excludes_checked_sequence_read(condition, object, index)
                    || prefix_proves_exact_length)
            {
                continue;
            }
            let Some(mut guard) = self.checked_sequence_read_guard_for_ir(&read)? else {
                continue;
            };
            // The guard evaluates the proved alias value before its pure
            // declaration, but the witness belongs to the source read and must
            // still be invalidated when that source index changes.
            if let crate::HirExpr::Index { object, index, .. } = &original {
                let Some(key) = checked_place_read_key(object, index) else {
                    continue;
                };
                guard.key = key;
                guard.dependencies = super::checked_place_dependencies(object, index);
            }
            if self.checked_place_read_witnesses.contains_key(&guard.key) {
                continue;
            }
            if guards
                .iter()
                .any(|existing: &CheckedDictReadGuard| existing.key == guard.key)
            {
                continue;
            }
            condition_fully_replaced &= prefix_proves_exact_length
                || condition_only_excludes_checked_sequence_read(condition, object, index);
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
