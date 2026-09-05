use super::{
    CheckedDictReadGuard, CheckedPlaceReadWitness, RustEmitter, RustExpr, Type,
    checked_place_dependencies, checked_place_read_key,
};

impl RustEmitter {
    pub(crate) fn lower_condition_with_checked_place_reads_for_ir(
        &mut self,
        condition: &crate::HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let reads = crate::hir_analysis::queries::collection_reads_in_condition(condition);
        if !reads.is_empty()
            && reads.iter().all(|read| {
                let crate::HirExpr::Index { object, ty, .. } = read else {
                    return false;
                };
                !crate::helpers::is_option_type(ty)
                    && self
                        .checked_read_failure_type(
                            if matches!(object.ty().resolve_alias(), Type::Dict(_, _)) {
                                super::CheckedPlaceFailureKind::Key
                            } else {
                                super::CheckedPlaceFailureKind::Index
                            },
                        )
                        .is_some()
            })
        {
            return Ok(None);
        }
        let mut guards = Vec::new();
        let mut previous_witnesses = Vec::new();
        for read in reads {
            let crate::HirExpr::Index { object, index, ty } = &read else {
                continue;
            };
            let Some(key) = checked_place_read_key(object, index) else {
                continue;
            };
            if crate::helpers::is_option_type(ty)
                && !ordering_compare_requires_present_read(condition, &key)
                && !condition_projects_from_read(condition, &key)
            {
                continue;
            }
            if self.checked_place_read_witnesses.contains_key(&key)
                || guards
                    .iter()
                    .any(|guard: &CheckedDictReadGuard| guard.key == key)
            {
                continue;
            }
            let Some(guard) = self.checked_condition_read_guard_for_ir(&read)? else {
                self.restore_checked_condition_witnesses(previous_witnesses);
                return Ok(None);
            };
            previous_witnesses.push((
                guard.key.clone(),
                self.checked_place_read_witnesses
                    .insert(guard.key.clone(), guard.witness()),
            ));
            guards.push(guard);
        }
        if guards.is_empty() {
            return Ok(None);
        }

        let lowered = self.lower_stmt_expr_for_ir(condition);
        self.restore_checked_condition_witnesses(previous_witnesses);
        let Some(mut lowered) = lowered? else {
            return Ok(None);
        };
        for guard in guards.into_iter().rev() {
            lowered = RustExpr::MethodCall {
                receiver: Box::new(guard.option),
                method: "is_some_and".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: guard.binding,
                        ty: crate::RustType::Named("_".to_string()),
                    }],
                    body: Box::new(lowered),
                    is_move: false,
                }],
            };
        }
        Ok(Some(lowered))
    }

    fn restore_checked_condition_witnesses(
        &mut self,
        previous_witnesses: Vec<(String, Option<CheckedPlaceReadWitness>)>,
    ) {
        for (key, previous) in previous_witnesses {
            if let Some(binding) = previous {
                self.checked_place_read_witnesses.insert(key, binding);
            } else {
                self.checked_place_read_witnesses.remove(&key);
            }
        }
    }

    pub(crate) fn checked_condition_read_guard_for_ir(
        &mut self,
        read: &crate::HirExpr,
    ) -> Result<Option<CheckedDictReadGuard>, crate::CodegenError> {
        let crate::HirExpr::Index { object, index, .. } = read else {
            return Ok(None);
        };
        if matches!(object.ty(), Type::Alias { name, .. } if name.starts_with("__sifr_defaultdict_"))
        {
            // Defaultdict indexing is total and intentionally inserts a
            // default. Its dedicated membership lowering performs a borrowed
            // lookup, so neither form needs an outer checked-read witness.
            return Ok(None);
        }
        if !matches!(object.ty().resolve_alias(), Type::Dict(_, _)) {
            return self.checked_sequence_read_guard_for_ir(read);
        }
        let Some(key) = checked_place_read_key(object, index) else {
            return Ok(None);
        };
        let dependencies = checked_place_dependencies(object, index);
        let lowered_object = if let crate::HirExpr::Index {
            object: parent,
            index: parent_index,
            ..
        } = object.as_ref()
        {
            let Some(witness) = self.checked_place_read_borrow_witness(parent, parent_index) else {
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
        let key_arg = self.checked_dict_key_arg_for_ir(index, lowered_index);
        let (order, binding) = self.next_checked_place_read_binding();
        Ok(Some(CheckedDictReadGuard {
            key,
            binding,
            option: RustExpr::MethodCall {
                receiver: Box::new(lowered_object),
                method: "get".to_string(),
                args: vec![key_arg],
            },
            negated: true,
            borrowed: true,
            dependencies,
            order,
        }))
    }
}

fn ordering_compare_requires_present_read(condition: &crate::HirExpr, read_key: &str) -> bool {
    let crate::HirExpr::Compare {
        left,
        ops,
        comparators,
        ..
    } = condition
    else {
        return false;
    };
    let mut operands = Vec::with_capacity(comparators.len() + 1);
    operands.push(left.as_ref());
    operands.extend(comparators.iter());
    ops.iter().enumerate().any(|(index, op)| {
        matches!(op.as_str(), "<" | "<=" | ">" | ">=")
            && operands.get(index..=index + 1).is_some_and(|pair| {
                pair.iter()
                    .any(|operand| expr_has_index_read_key(operand, read_key))
            })
    })
}

fn expr_has_index_read_key(expr: &crate::HirExpr, read_key: &str) -> bool {
    let crate::HirExpr::Index { object, index, .. } = expr else {
        return false;
    };
    super::checked_place_read_key(object, index).as_deref() == Some(read_key)
}

fn condition_projects_from_read(condition: &crate::HirExpr, read_key: &str) -> bool {
    let mut projected = false;
    crate::hir_analysis::traversal::walk_expr(condition, &mut |expr| {
        let parent = match expr {
            crate::HirExpr::Index { object, .. }
            | crate::HirExpr::FieldAccess { object, .. }
            | crate::HirExpr::MethodCall { object, .. } => Some(object.as_ref()),
            _ => None,
        };
        if parent.is_some_and(|parent| expr_has_index_read_key(parent, read_key)) {
            projected = true;
        }
    });
    projected
}
