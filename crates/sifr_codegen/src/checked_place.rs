use crate::{RustEmitter, RustExpr, RustLiteral, RustStmt, Type};

mod condition_reads;
mod control_flow;
mod fallible_reads;
mod nonempty_lists;
mod option_reads;
mod witnesses;

pub(crate) use witnesses::{CheckedDictReadGuard, CheckedPlaceReadWitness, checked_place_read_key};
use witnesses::{checked_place_dependencies, checked_place_expr_token};

fn condition_supports_checked_sequence_read(
    condition: &crate::HirExpr,
    object: &crate::HirExpr,
    index: &crate::HirExpr,
) -> bool {
    if let crate::HirExpr::BoolOp { op, values, .. } = condition {
        return match op.as_str() {
            "or" => {
                !values.is_empty()
                    && values
                        .iter()
                        .all(|value| condition_supports_checked_sequence_read(value, object, index))
            }
            "and" => values
                .iter()
                .any(|value| condition_supports_checked_sequence_read(value, object, index)),
            _ => false,
        };
    }
    let Some(object_token) = checked_place_expr_token(object) else {
        return false;
    };
    let Some(index_token) = checked_place_expr_token(index) else {
        return false;
    };
    let mut mentions_length = false;
    let mut mentions_index = matches!(
        index,
        crate::HirExpr::IntLiteral(_) | crate::HirExpr::LargeIntLiteral(_)
    );
    let direct_truthiness = checked_place_expr_token(condition).as_deref()
        == Some(object_token.as_str())
        || matches!(
            condition,
            crate::HirExpr::UnaryOp { op, operand, .. }
                if op == "not"
                    && checked_place_expr_token(operand).as_deref()
                        == Some(object_token.as_str())
        );
    crate::hir_analysis::traversal::walk_expr(condition, &mut |expr| {
        if checked_place_expr_token(expr).as_deref() == Some(index_token.as_str()) {
            mentions_index = true;
        }
        match expr {
            crate::HirExpr::MethodCall {
                object: len_object,
                method,
                args,
                ..
            } if method == "len"
                && args.is_empty()
                && checked_place_expr_token(len_object).as_deref()
                    == Some(object_token.as_str()) =>
            {
                mentions_length = true;
            }
            _ => {}
        }
    });
    (mentions_length || direct_truthiness) && mentions_index
}

fn condition_excludes_checked_sequence_read(
    condition: &crate::HirExpr,
    object: &crate::HirExpr,
    index: &crate::HirExpr,
) -> bool {
    fn is_len_of(candidate: &crate::HirExpr, object_token: &str) -> bool {
        matches!(
            candidate,
            crate::HirExpr::MethodCall {
                object,
                method,
                args,
                ..
            } if method == "len"
                && args.is_empty()
                && checked_place_expr_token(object).as_deref() == Some(object_token)
        )
    }

    fn is_zero(candidate: &crate::HirExpr) -> bool {
        matches!(candidate, crate::HirExpr::IntLiteral(0))
            || matches!(candidate, crate::HirExpr::LargeIntLiteral(value) if value == "0")
    }

    fn integer_literal(candidate: &crate::HirExpr) -> Option<i128> {
        match candidate {
            crate::HirExpr::IntLiteral(value) => Some(i128::from(*value)),
            crate::HirExpr::LargeIntLiteral(value) => value.parse().ok(),
            _ => None,
        }
    }

    let Some(object_token) = checked_place_expr_token(object) else {
        return false;
    };
    let Some(index_token) = checked_place_expr_token(index) else {
        return false;
    };
    match condition {
        crate::HirExpr::BoolOp { op, values, .. } if op == "or" => values
            .iter()
            .any(|value| condition_excludes_checked_sequence_read(value, object, index)),
        crate::HirExpr::UnaryOp { op, operand, .. } if op == "not" => {
            if checked_place_expr_token(operand).as_deref() == Some(object_token.as_str()) {
                return is_zero(index);
            }
            let crate::HirExpr::Compare {
                left,
                ops,
                comparators,
                ..
            } = operand.as_ref()
            else {
                return false;
            };
            ops.as_slice() == ["<"]
                && comparators.len() == 1
                && checked_place_expr_token(left).as_deref() == Some(index_token.as_str())
                && is_len_of(&comparators[0], &object_token)
        }
        crate::HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } if ops.len() == 1 && comparators.len() == 1 => {
            let right = &comparators[0];
            let left_is_index =
                checked_place_expr_token(left).as_deref() == Some(index_token.as_str());
            let right_is_index =
                checked_place_expr_token(right).as_deref() == Some(index_token.as_str());
            let literal_index = integer_literal(index).filter(|value| *value >= 0);
            let right_bound = integer_literal(right);
            let left_bound = integer_literal(left);
            (left_is_index && ops[0] == ">=" && is_len_of(right, &object_token))
                || (is_len_of(left, &object_token) && ops[0] == "<=" && right_is_index)
                || (is_len_of(left, &object_token)
                    && literal_index
                        .zip(right_bound)
                        .is_some_and(|(index, bound)| {
                            (ops[0] == "<" && index < bound) || (ops[0] == "<=" && index <= bound)
                        }))
                || (is_len_of(right, &object_token)
                    && literal_index.zip(left_bound).is_some_and(|(index, bound)| {
                        (ops[0] == ">" && index < bound) || (ops[0] == ">=" && index <= bound)
                    }))
                || (is_zero(index)
                    && is_len_of(left, &object_token)
                    && ops[0] == "=="
                    && is_zero(right))
        }
        _ => false,
    }
}

fn condition_only_excludes_checked_sequence_read(
    condition: &crate::HirExpr,
    object: &crate::HirExpr,
    index: &crate::HirExpr,
) -> bool {
    match condition {
        crate::HirExpr::BoolOp { op, values, .. } if op == "or" && !values.is_empty() => values
            .iter()
            .all(|value| condition_only_excludes_checked_sequence_read(value, object, index)),
        _ => condition_excludes_checked_sequence_read(condition, object, index),
    }
}

fn expr_mentions_name(expr: &crate::HirExpr, target: &str) -> bool {
    let mut found = false;
    crate::hir_analysis::traversal::walk_expr(expr, &mut |candidate| {
        if matches!(candidate, crate::HirExpr::Name { name, .. } if name == target) {
            found = true;
        }
    });
    found
}

fn checked_sequence_get_option(
    object: RustExpr,
    object_is_borrowed: bool,
    index: RustExpr,
    prefix: &str,
) -> RustExpr {
    let object_name = format!("{prefix}_collection");
    let index_name = format!("{prefix}_index");
    let normalized_name = format!("{prefix}_normalized");
    RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: object_name.clone(),
                ty: None,
                value: if object_is_borrowed {
                    object
                } else {
                    RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(object),
                    }
                },
            },
            RustStmt::Let {
                mutable: false,
                name: index_name.clone(),
                ty: None,
                value: index,
            },
            RustStmt::Let {
                mutable: false,
                name: normalized_name.clone(),
                ty: None,
                value: crate::build_normalized_index_expr(
                    &index_name,
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(object_name.clone())),
                        method: "len".to_string(),
                        args: Vec::new(),
                    },
                ),
            },
        ],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(object_name)),
                method: "get".to_string(),
                args: vec![RustExpr::Ident(normalized_name)],
            }),
            method: "cloned".to_string(),
            args: Vec::new(),
        })),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckedPlaceFailureKind {
    Index,
    Key,
    Unpack,
}

impl CheckedPlaceFailureKind {
    fn error_name(self) -> &'static str {
        match self {
            Self::Index => "IndexError",
            Self::Key => "KeyError",
            Self::Unpack => "ValueError",
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::Index => "collection index out of range",
            Self::Key => "key not found",
            Self::Unpack => "not enough values to unpack",
        }
    }
}

impl RustEmitter {
    pub(crate) fn next_checked_place_read_binding(&mut self) -> (usize, String) {
        let id = self.next_checked_place_read_witness;
        self.next_checked_place_read_witness += 1;
        (id, format!("__sifr_checked_value_{id}"))
    }

    pub(crate) fn try_lower_nonempty_pop_tail_for_ir(
        &mut self,
        stmt: &crate::HirStmt,
        following: &[crate::HirStmt],
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let crate::HirStmt::Let {
            name,
            ty: target_ty,
            value,
            ..
        } = stmt
        else {
            return Ok(None);
        };
        let crate::HirExpr::MethodCall {
            object,
            method,
            args,
            receiver_convention,
            receiver_target,
            mutable_arg_places,
            source,
            ty,
        } = value
        else {
            return Ok(None);
        };
        if crate::helpers::is_option_type(target_ty)
            || crate::helpers::is_option_type(ty)
            || !crate::intrinsic_method_emitters::supports_nonempty_pop_narrowing_type_for_codegen(
                object.ty(),
            )
            || !matches!(
                (method.as_str(), args.as_slice()),
                ("pop" | "popleft", []) | ("pop", [crate::HirExpr::IntLiteral(0)])
            )
        {
            return Ok(None);
        }

        let optional_call = crate::HirExpr::MethodCall {
            object: object.clone(),
            method: method.clone(),
            args: args.clone(),
            receiver_convention: *receiver_convention,
            receiver_target: receiver_target.clone(),
            mutable_arg_places: mutable_arg_places.clone(),
            source: source.clone(),
            ty: sifr_type_system::make_union(vec![ty.clone(), Type::None]),
        };
        let Some(lowered_call) = self.lower_stmt_expr_for_ir(&optional_call)? else {
            return Ok(None);
        };
        let Some(mut tail) = self.try_lower_scoped_stmt_block_for_ir(following)? else {
            return Ok(None);
        };
        if let Some(cache_stmt) = self.string_char_cache_init_stmt_for_local(name, target_ty) {
            tail.insert(0, cache_stmt);
        }
        let pattern = if self.mutated_vars.contains(name)
            || crate::stmt_support_emitter::should_force_mutable_binding(
                target_ty,
                &self.recursive_fields,
            ) {
            format!("Some(mut {name})")
        } else {
            format!("Some({name})")
        };
        Ok(Some(RustStmt::IfLet {
            pattern,
            expr: lowered_call,
            then_body: tail,
            else_body: None,
        }))
    }

    pub(crate) fn lower_checked_if_expr_for_ir(
        &mut self,
        condition: &crate::HirExpr,
        then_expr: &crate::HirExpr,
        else_expr: &crate::HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let mut guards = Vec::new();
        let mut previous_witnesses = Vec::new();
        for read in crate::hir_analysis::queries::collection_reads_in_condition(then_expr) {
            let crate::HirExpr::Index {
                object, index, ty, ..
            } = &read
            else {
                continue;
            };
            if crate::helpers::is_option_type(ty)
                || !condition_supports_checked_sequence_read(condition, object, index)
            {
                continue;
            }
            let Some(key) = checked_place_read_key(object, index) else {
                continue;
            };
            if self.checked_place_read_witnesses.contains_key(&key)
                || guards
                    .iter()
                    .any(|guard: &CheckedDictReadGuard| guard.key == key)
            {
                continue;
            }
            let Some(guard) = self.checked_condition_read_guard_for_ir(&read)? else {
                self.restore_atomic_checked_read_witnesses(previous_witnesses);
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

        let lowered_then = self.lower_stmt_expr_for_ir(then_expr);
        self.restore_atomic_checked_read_witnesses(previous_witnesses);
        let Some(mut lowered_then) = lowered_then? else {
            return Ok(None);
        };
        let Some(lowered_else) = self.lower_stmt_expr_for_ir(else_expr)? else {
            return Ok(None);
        };
        let Some(lowered_condition) = self.lower_condition_expr_for_ir(condition)? else {
            return Ok(None);
        };
        for guard in guards.into_iter().rev() {
            lowered_then = RustExpr::MethodCall {
                receiver: Box::new(guard.option),
                method: "map_or_else".to_string(),
                args: vec![
                    RustExpr::Closure {
                        params: Vec::new(),
                        body: Box::new(lowered_else.clone()),
                        is_move: false,
                    },
                    RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: guard.binding,
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(lowered_then),
                        is_move: false,
                    },
                ],
            };
        }
        Ok(Some(RustExpr::If {
            cond: Box::new(lowered_condition),
            then_expr: Box::new(lowered_then),
            else_expr: Some(Box::new(lowered_else)),
        }))
    }

    pub(crate) fn try_lower_atomic_checked_read_stmt_for_ir(
        &mut self,
        stmt: &crate::HirStmt,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        if self.checked_place_atomic_guard_suppressed
            || self
                .checked_read_failure_type(CheckedPlaceFailureKind::Index)
                .is_some()
            || self
                .checked_read_failure_type(CheckedPlaceFailureKind::Key)
                .is_some()
            || self.loop_else_stack.is_empty()
            || self.stmt_uses_checked_option_target(stmt)
            || matches!(
                stmt,
                crate::HirStmt::If { .. }
                    | crate::HirStmt::While { .. }
                    | crate::HirStmt::For { .. }
                    | crate::HirStmt::AsyncFor { .. }
                    | crate::HirStmt::TryExcept { .. }
                    | crate::HirStmt::TryFinally { .. }
                    | crate::HirStmt::With { .. }
                    | crate::HirStmt::AsyncWith { .. }
                    | crate::HirStmt::Match { .. }
                    | crate::HirStmt::NestedFunction { .. }
            )
        {
            return Ok(None);
        }

        let reads = self
            .body_analysis
            .proven_reads_in(std::slice::from_ref(stmt));
        let mut guards = Vec::new();
        let mut previous_witnesses = Vec::new();
        for read in reads {
            let crate::HirExpr::Index { object, index, .. } = &read else {
                continue;
            };
            let Some(key) = checked_place_read_key(object, index) else {
                continue;
            };
            if self.checked_place_read_witnesses.contains_key(&key)
                || guards
                    .iter()
                    .any(|guard: &CheckedDictReadGuard| guard.key == key)
            {
                continue;
            }
            let Some(guard) = self.checked_condition_read_guard_for_ir(&read)? else {
                self.restore_atomic_checked_read_witnesses(previous_witnesses);
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

        self.checked_place_atomic_guard_suppressed = true;
        let lowered = self.try_lower_stmt_block_for_ir(std::slice::from_ref(stmt));
        self.checked_place_atomic_guard_suppressed = false;
        self.restore_atomic_checked_read_witnesses(previous_witnesses);
        let Some(mut lowered) = lowered? else {
            return Ok(None);
        };
        for guard in guards.into_iter().rev() {
            lowered.insert(
                0,
                RustStmt::LetElse {
                    pattern: format!("Some({})", guard.binding),
                    value: guard.option,
                    else_body: vec![RustStmt::Break],
                },
            );
        }
        Ok(Some(lowered))
    }

    fn restore_atomic_checked_read_witnesses(
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

    pub(crate) fn try_lower_checked_place_if_for_ir(
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
        if let Some(lowered) = self.try_lower_checked_dict_if_for_ir(
            condition,
            then_body,
            elif_clauses,
            else_body.as_deref(),
        )? {
            return Ok(Some(lowered));
        }
        self.try_lower_checked_sequence_if_for_ir(
            condition,
            then_body,
            elif_clauses,
            else_body.as_deref(),
        )
    }

    pub(crate) fn stmt_uses_checked_place_read_witness(&self, stmt: &crate::HirStmt) -> bool {
        if self.checked_place_read_witnesses.is_empty() {
            return false;
        }
        let mut uses_witness = false;
        crate::hir_analysis::traversal::walk_stmts(
            std::slice::from_ref(stmt),
            crate::hir_analysis::traversal::TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut |_| {},
            &mut |expr| {
                let crate::HirExpr::Index { object, index, .. } = expr else {
                    return;
                };
                if checked_place_read_key(object, index)
                    .is_some_and(|key| self.checked_place_read_witnesses.contains_key(&key))
                {
                    uses_witness = true;
                }
            },
        );
        uses_witness
    }

    pub(crate) fn checked_dict_key_arg_for_ir(
        &self,
        index: &crate::HirExpr,
        lowered: RustExpr,
    ) -> RustExpr {
        if let crate::HirExpr::StringLiteral(value) = index {
            return RustExpr::Verbatim(format!("{value:?}"));
        }
        if matches!(index, crate::HirExpr::Name { name, .. } if self.borrowed_params.contains(name))
        {
            return lowered;
        }
        RustExpr::Ref {
            mutable: false,
            expr: Box::new(lowered),
        }
    }

    pub(crate) fn checked_place_read_witness(
        &self,
        object: &crate::HirExpr,
        index: &crate::HirExpr,
        result_ty: &Type,
    ) -> Option<RustExpr> {
        let key = checked_place_read_key(object, index)?;
        let witness = self.checked_place_read_witnesses.get(&key)?.clone();
        if let Some(uses) = self.checked_place_read_witness_uses.borrow_mut().as_mut() {
            uses.insert(key);
        }
        let value = RustExpr::Ident(witness.binding);
        let value = if witness.borrowed {
            RustExpr::Deref(Box::new(value))
        } else {
            value
        };
        let witnessed_ty = result_ty
            .optional_member_type()
            .unwrap_or_else(|| result_ty.clone());
        if crate::helpers::is_copy_type_for_codegen(&witnessed_ty) {
            Some(value)
        } else {
            Some(RustExpr::MethodCall {
                receiver: Box::new(if witness.borrowed {
                    RustExpr::Paren(Box::new(value))
                } else {
                    value
                }),
                method: "clone".to_string(),
                args: Vec::new(),
            })
        }
    }

    pub(crate) fn has_checked_place_read_witness(
        &self,
        object: &crate::HirExpr,
        index: &crate::HirExpr,
    ) -> bool {
        checked_place_read_key(object, index)
            .is_some_and(|key| self.checked_place_read_witnesses.contains_key(&key))
    }

    pub(crate) fn checked_place_read_borrow_witness(
        &self,
        object: &crate::HirExpr,
        index: &crate::HirExpr,
    ) -> Option<RustExpr> {
        let key = checked_place_read_key(object, index)?;
        let witness = self.checked_place_read_witnesses.get(&key)?.clone();
        if let Some(uses) = self.checked_place_read_witness_uses.borrow_mut().as_mut() {
            uses.insert(key);
        }
        let value = RustExpr::Ident(witness.binding);
        Some(if witness.borrowed {
            value
        } else {
            RustExpr::Ref {
                mutable: false,
                expr: Box::new(value),
            }
        })
    }

    pub(crate) fn checked_dict_read_guard_for_ir(
        &mut self,
        condition: &crate::HirExpr,
    ) -> Result<Option<CheckedDictReadGuard>, crate::CodegenError> {
        let (contains, negated) = match condition {
            crate::HirExpr::ContainsOp { .. } => (condition, false),
            crate::HirExpr::UnaryOp { op, operand, .. }
                if op == "not" && matches!(operand.as_ref(), crate::HirExpr::ContainsOp { .. }) =>
            {
                (operand.as_ref(), true)
            }
            _ => return Ok(None),
        };
        let crate::HirExpr::ContainsOp {
            element,
            collection,
            ..
        } = contains
        else {
            return Ok(None);
        };
        let dictionary = match collection.as_ref() {
            crate::HirExpr::MethodCall {
                object,
                method,
                args,
                ..
            } if method == "keys"
                && args.is_empty()
                && matches!(object.ty().resolve_alias(), Type::Dict(_, _)) =>
            {
                object.as_ref()
            }
            collection => collection,
        };
        let Type::Dict(_, _) = dictionary.ty().resolve_alias() else {
            return Ok(None);
        };
        let Some(key) = checked_place_read_key(dictionary, element) else {
            return Ok(None);
        };
        let dependencies = checked_place_dependencies(dictionary, element);
        let lowered_collection = if let Some(path) = self.emit_shared_receiver_path(dictionary) {
            path
        } else if let Some(lowered) = self.lower_stmt_expr_for_ir(dictionary)? {
            lowered
        } else {
            return Ok(None);
        };
        let Some(lowered_element) = self.lower_stmt_expr_for_ir(element)? else {
            return Ok(None);
        };
        let key_arg = self.checked_dict_key_arg_for_ir(element, lowered_element);
        let option = RustExpr::MethodCall {
            receiver: Box::new(lowered_collection),
            method: "get".to_string(),
            args: vec![key_arg],
        };
        let (order, binding) = self.next_checked_place_read_binding();
        Ok(Some(CheckedDictReadGuard {
            key,
            binding,
            option,
            negated,
            borrowed: true,
            dependencies,
            order,
        }))
    }

    fn checked_sequence_read_guard_for_ir(
        &mut self,
        read: &crate::HirExpr,
    ) -> Result<Option<CheckedDictReadGuard>, crate::CodegenError> {
        let crate::HirExpr::Index { object, index, .. } = read else {
            return Ok(None);
        };
        let Some(key) = checked_place_read_key(object, index) else {
            return Ok(None);
        };
        let dependencies = checked_place_dependencies(object, index);
        let (lowered_object, object_is_borrowed) = if let crate::HirExpr::Index {
            object: parent,
            index: parent_index,
            ..
        } = object.as_ref()
        {
            let Some(witness) = self.checked_place_read_borrow_witness(parent, parent_index) else {
                return Ok(None);
            };
            (witness, true)
        } else if let Some(path) = self.emit_shared_receiver_path(object) {
            (path, false)
        } else {
            let Some(lowered) = self.lower_stmt_expr_for_ir(object)? else {
                return Ok(None);
            };
            (lowered, false)
        };
        let lowered_object = self.rewrite_stdlib_constant_idents_in_expr(lowered_object);
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        let lowered_index = Self::clone_non_copy_name_expr_for_ir(index, lowered_index);
        let option = match object.ty().resolve_alias() {
            Type::List(_) | Type::Bytes => checked_sequence_get_option(
                lowered_object,
                object_is_borrowed,
                lowered_index,
                "__sifr_checked_read",
            ),
            Type::Str => {
                self.lower_string_index_option_with_cache(object, lowered_object, lowered_index)
            }
            _ => return Ok(None),
        };
        let (order, binding) = self.next_checked_place_read_binding();
        Ok(Some(CheckedDictReadGuard {
            key,
            binding,
            option,
            negated: true,
            borrowed: false,
            dependencies,
            order,
        }))
    }

    pub(crate) fn try_capture_checked_place_control_stmt(
        &mut self,
        stmt: &crate::HirStmt,
        following_stmts: Option<&[crate::HirStmt]>,
    ) -> Result<bool, crate::CodegenError> {
        if let Some(guard) = self.try_lower_checked_dict_exit_guard_for_ir(stmt)? {
            self.capture_checked_place_stmts(std::slice::from_ref(&guard));
            return Ok(true);
        }
        if let Some(guards) =
            self.try_lower_checked_sequence_exit_guards_for_ir(stmt, following_stmts)?
        {
            self.capture_checked_place_stmts(&guards);
            return Ok(true);
        }
        let lowered = self.try_lower_checked_place_if_for_ir(stmt)?;
        let Some(lowered) = lowered else {
            return Ok(false);
        };
        self.capture_checked_place_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    fn capture_checked_place_stmts(&mut self, stmts: &[RustStmt]) {
        for stmt in stmts {
            self.push_captured_stmt(stmt);
        }
        self.lowering_stats.stmt_structured += 1;
        self.lowering_stats.stmt_candidate_structured += 1;
    }

    pub(crate) fn checked_place_failure_return(
        &mut self,
        failure: &Type,
        kind: CheckedPlaceFailureKind,
    ) -> RustStmt {
        let error = RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                kind.error_name().to_string(),
                "new".to_string(),
            ])),
            args: vec![RustExpr::Literal(RustLiteral::Str(
                kind.message().to_string(),
            ))],
        };
        let error = self.coerce_error_type_for_ir(failure, error);
        RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
            args: vec![error],
        }))
    }
}
