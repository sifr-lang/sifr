use super::{HirExpr, HirStmt, RustEmitter, RustStmt, Type};
impl RustEmitter {
    pub(crate) fn try_lower_isinstance_union_chain_for_ir(
        &mut self,
        condition: &HirExpr,
        then_body: &[HirStmt],
        elif_clauses: &[(HirExpr, Vec<HirStmt>)],
        else_body: Option<&[HirStmt]>,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Some((var_name, first_variant, first_enum_name, _)) =
            crate::helpers::detect_isinstance_union(condition)
        else {
            return Ok(None);
        };
        let mut branch_specs: Vec<(String, &[HirStmt])> = vec![(first_variant, then_body)];
        let mut needed_variants = vec![branch_specs[0].0.clone()];
        for (elif_cond, elif_body) in elif_clauses {
            let Some((elif_var, elif_variant, _, _)) =
                crate::helpers::detect_isinstance_union(elif_cond)
            else {
                return Ok(None);
            };
            if elif_var != var_name || needed_variants.contains(&elif_variant) {
                return Ok(None);
            }
            needed_variants.push(elif_variant.clone());
            branch_specs.push((elif_variant, elif_body.as_slice()));
        }

        let enum_name = self.resolve_union_enum_name(&first_enum_name, &needed_variants);
        let all_variants = self
            .union_enums
            .get(&enum_name)
            .ok_or_else(|| {
                crate::CodegenError::new(format!(
                    "internal codegen invariant violated: union narrowing has no definition for {enum_name}"
                ))
            })?
            .iter()
            .map(Type::union_variant_name)
            .collect::<Vec<_>>();
        let mut arms = Vec::with_capacity(branch_specs.len() + usize::from(else_body.is_some()));
        for (variant_name, body) in branch_specs {
            let mutated = self.body_analysis.mutated_in(body);
            let binding = if mutated.contains(&var_name) {
                format!("mut {var_name}")
            } else {
                var_name.clone()
            };
            let Some(lowered_body) = self.try_lower_if_branch_for_ir(body)? else {
                return Ok(None);
            };
            arms.push(crate::RustMatchArm {
                pattern: format!("{enum_name}::{variant_name}({binding})"),
                bindings: vec![var_name.clone()],
                guard: None,
                body: lowered_body,
            });
        }

        let remaining_variants = all_variants
            .iter()
            .filter(|variant| !needed_variants.contains(variant))
            .cloned()
            .collect::<Vec<_>>();
        if let Some(else_body) = else_body {
            if !remaining_variants.is_empty() {
                let else_mutated = self.body_analysis.mutated_in(else_body);
                let else_binding = if else_mutated.contains(&var_name) {
                    format!("mut {var_name}")
                } else {
                    var_name.clone()
                };
                let Some(lowered_else_body) = self.try_lower_if_branch_for_ir(else_body)? else {
                    return Ok(None);
                };
                let pattern = if remaining_variants.len() == 1 {
                    format!("{enum_name}::{}({else_binding})", remaining_variants[0])
                } else {
                    else_binding
                };
                arms.push(crate::RustMatchArm {
                    pattern,
                    bindings: vec![var_name.clone()],
                    guard: None,
                    body: lowered_else_body,
                });
            }
        } else if !remaining_variants.is_empty() {
            arms.push(crate::RustMatchArm {
                pattern: "_".to_string(),
                bindings: vec![],
                guard: None,
                body: vec![],
            });
        }

        Ok(Some(RustStmt::Match {
            expr: crate::RustExpr::Ident(var_name),
            arms,
        }))
    }

    pub(crate) fn try_lower_borrowed_name_compare_condition_for_ir(
        &self,
        expr: &HirExpr,
    ) -> Option<crate::RustExpr> {
        let HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } = expr
        else {
            return None;
        };
        if ops.len() != 1 || comparators.len() != 1 {
            return None;
        }
        let rhs = comparators.first()?;
        let lowered_op = match ops[0].as_str() {
            "==" | "!=" | "<" | "<=" | ">" | ">=" => ops[0].as_str(),
            "is" => "==",
            "is not" => "!=",
            _ => return None,
        };
        let borrowed_string_literal_operand =
            |operand: &HirExpr, emitter: &Self| -> Option<crate::RustExpr> {
                let HirExpr::Name { name, ty, .. } = operand else {
                    return None;
                };
                if !emitter.borrowed_params.contains(name)
                    && !emitter.mut_borrowed_params.contains(name)
                {
                    return None;
                }
                if !matches!(
                    crate::resolve_alias_type_for_plain_call(ty),
                    Type::Str | Type::LiteralStr(_)
                ) {
                    return None;
                }
                Some(emitter.string_view_expr(operand, crate::RustExpr::Ident(name.clone())))
            };
        match (left.as_ref(), rhs) {
            (name_expr, HirExpr::StringLiteral(literal)) => {
                if let Some(lowered_name) = borrowed_string_literal_operand(name_expr, self) {
                    return Some(crate::RustExpr::BinOp {
                        left: Box::new(lowered_name),
                        op: lowered_op.to_string(),
                        right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Str(
                            literal.clone(),
                        ))),
                    });
                }
            }
            (HirExpr::StringLiteral(literal), name_expr) => {
                if let Some(lowered_name) = borrowed_string_literal_operand(name_expr, self) {
                    return Some(crate::RustExpr::BinOp {
                        left: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Str(
                            literal.clone(),
                        ))),
                        op: lowered_op.to_string(),
                        right: Box::new(lowered_name),
                    });
                }
            }
            _ => {}
        }
        let effective_name_ty = |operand: &HirExpr, emitter: &Self| -> Option<Type> {
            let HirExpr::Name { name, ty, .. } = operand else {
                return None;
            };
            if matches!(
                crate::resolve_alias_type_for_plain_call(ty),
                Type::Any | Type::Unknown
            ) {
                if let Some(bound_ty) = emitter.local_binding_types.get(name) {
                    return Some(bound_ty.clone());
                }
            }
            Some(ty.clone())
        };

        let lower_operand =
            |operand: &HirExpr, emitter: &Self| -> Option<(crate::RustExpr, bool, Type)> {
                let HirExpr::Name { name, .. } = operand else {
                    return None;
                };
                let borrowed = emitter.borrowed_params.contains(name)
                    || emitter.mut_borrowed_params.contains(name);
                let effective_ty = effective_name_ty(operand, emitter)?;
                let ident = crate::RustExpr::Ident(name.clone());
                let lowered = if borrowed {
                    crate::RustExpr::Deref(Box::new(ident))
                } else {
                    ident
                };
                Some((lowered, borrowed, effective_ty))
            };

        let (mut lowered_left, left_borrowed, left_ty) = lower_operand(left, self)?;
        let (mut lowered_right, right_borrowed, right_ty) = lower_operand(rhs, self)?;
        if !left_borrowed && !right_borrowed {
            return None;
        }
        let left_is_option = crate::helpers::is_option_type(&left_ty);
        let right_is_option = crate::helpers::is_option_type(&right_ty);
        let left_none_like = matches!(
            crate::resolve_alias_type_for_plain_call(&left_ty),
            Type::None
        );
        let right_none_like = matches!(
            crate::resolve_alias_type_for_plain_call(&right_ty),
            Type::None
        );

        if left_is_option && !right_is_option && !right_none_like {
            if !crate::helpers::is_copy_type_for_codegen(&right_ty) {
                lowered_right = crate::ownership_plan::materialize_owned_value(
                    &right_ty,
                    crate::RustExpr::Paren(Box::new(lowered_right)),
                );
            }
            lowered_right = crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                args: vec![lowered_right],
            };
        } else if !left_is_option && right_is_option && !left_none_like {
            if !crate::helpers::is_copy_type_for_codegen(&left_ty) {
                lowered_left = crate::ownership_plan::materialize_owned_value(
                    &left_ty,
                    crate::RustExpr::Paren(Box::new(lowered_left)),
                );
            }
            lowered_left = crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                args: vec![lowered_left],
            };
        }

        Some(crate::RustExpr::BinOp {
            left: Box::new(lowered_left),
            op: lowered_op.to_string(),
            right: Box::new(lowered_right),
        })
    }

    pub(crate) fn condition_uses_borrowed_name_for_ir(&self, expr: &HirExpr) -> bool {
        match expr {
            HirExpr::Name { name, .. } => {
                self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name)
            }
            HirExpr::Compare {
                left, comparators, ..
            } => {
                self.condition_uses_borrowed_name_for_ir(left)
                    || comparators
                        .iter()
                        .any(|expr| self.condition_uses_borrowed_name_for_ir(expr))
            }
            HirExpr::BoolOp { values, .. } => values
                .iter()
                .any(|expr| self.condition_uses_borrowed_name_for_ir(expr)),
            HirExpr::BinOp { left, right, .. } => {
                self.condition_uses_borrowed_name_for_ir(left)
                    || self.condition_uses_borrowed_name_for_ir(right)
            }
            HirExpr::UnaryOp { operand, .. } => self.condition_uses_borrowed_name_for_ir(operand),
            HirExpr::Index { object, index, .. } => {
                self.condition_uses_borrowed_name_for_ir(object)
                    || self.condition_uses_borrowed_name_for_ir(index)
            }
            HirExpr::FieldAccess { object, .. } => self.condition_uses_borrowed_name_for_ir(object),
            HirExpr::MethodCall { object, args, .. } => {
                self.condition_uses_borrowed_name_for_ir(object)
                    || args
                        .iter()
                        .any(|expr| self.condition_uses_borrowed_name_for_ir(expr))
            }
            HirExpr::Call { args, .. } | HirExpr::IteratorCall { args, .. } => args
                .iter()
                .any(|expr| self.condition_uses_borrowed_name_for_ir(expr)),
            HirExpr::TupleLiteral { elements, .. } | HirExpr::ListLiteral { elements, .. } => {
                elements
                    .iter()
                    .any(|expr| self.condition_uses_borrowed_name_for_ir(expr))
            }
            HirExpr::DictLiteral { keys, values, .. } => {
                keys.iter()
                    .any(|expr| self.condition_uses_borrowed_name_for_ir(expr))
                    || values
                        .iter()
                        .any(|expr| self.condition_uses_borrowed_name_for_ir(expr))
            }
            HirExpr::SetLiteral { elements, .. } => elements
                .iter()
                .any(|expr| self.condition_uses_borrowed_name_for_ir(expr)),
            HirExpr::IfExpr {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                self.condition_uses_borrowed_name_for_ir(condition)
                    || self.condition_uses_borrowed_name_for_ir(then_expr)
                    || self.condition_uses_borrowed_name_for_ir(else_expr)
            }
            HirExpr::WalrusExpr { value, .. } => self.condition_uses_borrowed_name_for_ir(value),
            HirExpr::GeneratorExpr {
                expr, iter, filter, ..
            } => {
                self.condition_uses_borrowed_name_for_ir(expr)
                    || self.condition_uses_borrowed_name_for_ir(iter)
                    || filter
                        .as_ref()
                        .is_some_and(|cond| self.condition_uses_borrowed_name_for_ir(cond))
            }
            HirExpr::ListComp {
                expr, generators, ..
            } => {
                self.condition_uses_borrowed_name_for_ir(expr)
                    || generators.iter().any(|(_, iter, cond)| {
                        self.condition_uses_borrowed_name_for_ir(iter)
                            || cond
                                .as_ref()
                                .is_some_and(|cond| self.condition_uses_borrowed_name_for_ir(cond))
                    })
            }
            HirExpr::DictComp {
                key_expr,
                val_expr,
                generators,
                ..
            } => {
                self.condition_uses_borrowed_name_for_ir(key_expr)
                    || self.condition_uses_borrowed_name_for_ir(val_expr)
                    || generators.iter().any(|(_, iter, cond)| {
                        self.condition_uses_borrowed_name_for_ir(iter)
                            || cond
                                .as_ref()
                                .is_some_and(|cond| self.condition_uses_borrowed_name_for_ir(cond))
                    })
            }
            HirExpr::SetComp {
                expr, generators, ..
            } => {
                self.condition_uses_borrowed_name_for_ir(expr)
                    || generators.iter().any(|(_, iter, cond)| {
                        self.condition_uses_borrowed_name_for_ir(iter)
                            || cond
                                .as_ref()
                                .is_some_and(|cond| self.condition_uses_borrowed_name_for_ir(cond))
                    })
            }
            HirExpr::RangeLiteral {
                start, end, step, ..
            } => {
                self.condition_uses_borrowed_name_for_ir(start)
                    || self.condition_uses_borrowed_name_for_ir(end)
                    || step
                        .as_ref()
                        .is_some_and(|step| self.condition_uses_borrowed_name_for_ir(step))
            }
            HirExpr::ContainsOp {
                element,
                collection,
                ..
            } => {
                self.condition_uses_borrowed_name_for_ir(element)
                    || self.condition_uses_borrowed_name_for_ir(collection)
            }
            HirExpr::Slice {
                object,
                start,
                stop,
                step,
                ..
            } => {
                self.condition_uses_borrowed_name_for_ir(object)
                    || start
                        .as_ref()
                        .is_some_and(|start| self.condition_uses_borrowed_name_for_ir(start))
                    || stop
                        .as_ref()
                        .is_some_and(|stop| self.condition_uses_borrowed_name_for_ir(stop))
                    || step
                        .as_ref()
                        .is_some_and(|step| self.condition_uses_borrowed_name_for_ir(step))
            }
            HirExpr::Lambda { body, .. } => self.condition_uses_borrowed_name_for_ir(body),
            HirExpr::QuestionMark { expr, .. } => self.condition_uses_borrowed_name_for_ir(expr),
            HirExpr::OkWrap { value, .. } | HirExpr::ErrWrap { value, .. } => {
                self.condition_uses_borrowed_name_for_ir(value)
            }
            HirExpr::SuperCall { args, .. } => args
                .iter()
                .any(|expr| self.condition_uses_borrowed_name_for_ir(expr)),
            _ => false,
        }
    }

    pub(crate) fn try_lower_if_stmt_for_ir(
        &mut self,
        condition: &HirExpr,
        then_body: &[HirStmt],
        elif_clauses: &[(HirExpr, Vec<HirStmt>)],
        else_body: Option<&[HirStmt]>,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let speculative_string_char_cache_vars = self.string_char_cache_vars.clone();
        if let Some(lowered_union) = self.try_lower_isinstance_union_chain_for_ir(
            condition,
            then_body,
            elif_clauses,
            else_body,
        )? {
            self.string_char_cache_vars = speculative_string_char_cache_vars;
            return Ok(Some(lowered_union));
        }
        self.string_char_cache_vars = speculative_string_char_cache_vars;

        if elif_clauses.is_empty()
            && let Some(condition_is_true) = self.unwrapped_none_compare_truth_for_ir(condition)
        {
            let selected_body = if condition_is_true {
                then_body
            } else {
                else_body.unwrap_or_default()
            };
            let Some(lowered) = self.try_lower_scoped_stmt_block_for_ir(selected_body)? else {
                return Ok(None);
            };
            return Ok(Some(RustStmt::Block(lowered)));
        }

        if elif_clauses.is_empty()
            && else_body.is_none()
            && crate::helpers::codegen_body_always_exits(then_body)
        {
            let Some(lowered_then_body) = self.try_lower_scoped_stmt_block_for_ir(then_body)?
            else {
                return Ok(None);
            };
            if let Some(option_vars) = self.detect_or_is_none_vars_with_bindings_for_ir(condition) {
                let pattern = format!(
                    "({})",
                    option_vars
                        .iter()
                        .map(|option_var| self.option_binding_pattern_for_ir(option_var))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let value = crate::RustExpr::Tuple(
                    option_vars
                        .iter()
                        .map(|option_var| self.option_binding_value_expr_for_ir(option_var))
                        .collect(),
                );
                self.option_unwrapped_vars.extend(option_vars);
                return Ok(Some(RustStmt::LetElse {
                    pattern,
                    value,
                    else_body: lowered_then_body,
                }));
            }
            if let Some(option_var) = crate::helpers::detect_is_none_var(condition)
                .or_else(|| crate::helpers::detect_not_option_truthiness(condition))
            {
                self.option_unwrapped_vars.insert(option_var.clone());
                return Ok(Some(RustStmt::LetElse {
                    pattern: self.option_binding_pattern_for_ir(&option_var),
                    value: self.option_binding_value_expr_for_ir(&option_var),
                    else_body: lowered_then_body,
                }));
            }
            if let Some(option_vars) =
                crate::helpers::detect_or_not_option_truthiness_vars(condition)
            {
                let pattern = format!(
                    "({})",
                    option_vars
                        .iter()
                        .map(|option_var| self.option_binding_pattern_for_ir(option_var))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let value = crate::RustExpr::Tuple(
                    option_vars
                        .iter()
                        .map(|option_var| self.option_binding_value_expr_for_ir(option_var))
                        .collect(),
                );
                self.option_unwrapped_vars.extend(option_vars);
                return Ok(Some(RustStmt::LetElse {
                    pattern,
                    value,
                    else_body: lowered_then_body,
                }));
            }
        }

        let mut nested_else = if let Some(else_body) = else_body {
            let Some(lowered_else) = self.try_lower_if_branch_for_ir(else_body)? else {
                return Ok(None);
            };
            Some(lowered_else)
        } else {
            None
        };

        for (elif_cond, elif_body) in elif_clauses.iter().rev() {
            let Some(lowered_elif) =
                self.try_lower_if_clause_for_ir(elif_cond, elif_body, nested_else)?
            else {
                return Ok(None);
            };
            nested_else = Some(vec![lowered_elif]);
        }

        self.try_lower_if_clause_for_ir(condition, then_body, nested_else)
    }

    fn unwrapped_none_compare_truth_for_ir(&self, condition: &HirExpr) -> Option<bool> {
        let HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } = condition
        else {
            return None;
        };
        if ops.len() != 1
            || comparators.len() != 1
            || !matches!(comparators[0], HirExpr::NoneLiteral)
        {
            return None;
        }
        let HirExpr::Name { name, ty, .. } = left.as_ref() else {
            return None;
        };
        if crate::helpers::is_option_type(ty) || !self.option_unwrapped_vars.contains(name) {
            return None;
        }
        match ops[0].as_str() {
            "is not" | "!=" => Some(true),
            "is" | "==" => Some(false),
            _ => None,
        }
    }

    pub(crate) fn try_lower_if_clause_for_ir(
        &mut self,
        condition: &HirExpr,
        then_body: &[HirStmt],
        nested_else: Option<Vec<RustStmt>>,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let narrowed_then_vars = crate::helpers::detect_is_not_none_var(condition)
            .into_iter()
            .chain(crate::helpers::detect_option_truthiness(condition))
            .chain(
                crate::helpers::detect_and_not_none_vars(condition)
                    .into_iter()
                    .flatten(),
            )
            .collect::<Vec<_>>();
        let Some(lowered_then_body) =
            self.try_lower_option_narrowed_branch_for_ir(then_body, &narrowed_then_vars)?
        else {
            return Ok(None);
        };

        if let Some(option_var) = crate::helpers::detect_is_not_none_var(condition) {
            return Ok(Some(RustStmt::IfLet {
                pattern: self.option_binding_pattern_for_ir(&option_var),
                expr: self.option_binding_value_expr_for_ir(&option_var),
                then_body: lowered_then_body,
                else_body: nested_else,
            }));
        }

        if let Some(option_vars) = crate::helpers::detect_and_not_none_vars(condition) {
            let mut chain_then = lowered_then_body;
            for option_var in option_vars.iter().rev() {
                chain_then = vec![RustStmt::IfLet {
                    pattern: self.option_binding_pattern_for_ir(option_var),
                    expr: self.option_binding_value_expr_for_ir(option_var),
                    then_body: chain_then,
                    else_body: None,
                }];
            }
            let Some(mut chain_root) = chain_then.into_iter().next() else {
                return Ok(None);
            };
            if let RustStmt::IfLet { else_body, .. } = &mut chain_root {
                *else_body = nested_else;
            }
            return Ok(Some(chain_root));
        }

        if let Some(option_var) = crate::helpers::detect_option_truthiness(condition) {
            return Ok(Some(RustStmt::IfLet {
                pattern: self.option_binding_pattern_for_ir(&option_var),
                expr: self.option_binding_value_expr_for_ir(&option_var),
                then_body: lowered_then_body,
                else_body: nested_else,
            }));
        }

        if let Some(option_var) = crate::helpers::detect_is_none_var(condition) {
            let Some(lowered_cond) = self.lower_condition_expr_for_ir(condition)? else {
                return Ok(None);
            };
            let lowered_else = nested_else.map(|else_body| {
                vec![RustStmt::IfLet {
                    pattern: self.option_binding_pattern_for_ir(&option_var),
                    expr: self.option_binding_value_expr_for_ir(&option_var),
                    then_body: else_body,
                    else_body: None,
                }]
            });
            return Ok(Some(RustStmt::If {
                cond: lowered_cond,
                then_body: lowered_then_body,
                else_body: lowered_else,
            }));
        }

        let Some(lowered_cond) = self.lower_condition_expr_for_ir(condition)? else {
            return Ok(None);
        };
        Ok(Some(RustStmt::If {
            cond: lowered_cond,
            then_body: lowered_then_body,
            else_body: nested_else,
        }))
    }

    pub(crate) fn try_lower_if_branch_for_ir(
        &mut self,
        body: &[HirStmt],
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        self.try_lower_scoped_stmt_block_for_ir(body)
    }

    fn try_lower_option_narrowed_branch_for_ir(
        &mut self,
        body: &[HirStmt],
        names: &[String],
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let previous = self.option_unwrapped_vars.clone();
        self.option_unwrapped_vars.extend(names.iter().cloned());
        let lowered = self.try_lower_scoped_stmt_block_for_ir(body);
        self.option_unwrapped_vars = previous;
        lowered
    }

    pub(crate) fn detect_or_is_none_vars_with_bindings_for_ir(
        &self,
        expr: &HirExpr,
    ) -> Option<Vec<String>> {
        let HirExpr::BoolOp { op, values, .. } = expr else {
            return crate::helpers::detect_or_is_none_vars(expr);
        };
        if op != "or" {
            return crate::helpers::detect_or_is_none_vars(expr);
        }
        let mut vars = Vec::new();
        for value in values {
            let HirExpr::Compare {
                left,
                ops,
                comparators,
                ..
            } = value
            else {
                return None;
            };
            if ops.len() != 1
                || !(ops[0] == "is" || ops[0] == "==")
                || !matches!(comparators[0], HirExpr::NoneLiteral)
            {
                return None;
            }
            let HirExpr::Name { name, ty, .. } = left.as_ref() else {
                return None;
            };
            let option_like = crate::helpers::is_option_type(ty)
                || self
                    .local_binding_types
                    .get(name)
                    .is_some_and(crate::helpers::is_option_type);
            if !option_like {
                return None;
            }
            vars.push(name.clone());
        }
        if vars.len() >= 2 { Some(vars) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn isinstance_union_condition(target: &str) -> HirExpr {
        HirExpr::Call {
            func: "isinstance".to_string(),
            args: vec![
                HirExpr::Name {
                    name: "value".to_string(),
                    binding_id: None,
                    ty: Type::Union(vec![Type::Int, Type::Str]),
                },
                HirExpr::StringLiteral(target.to_string()),
            ],
            mutable_arg_places: vec![None, None],
            ty: Type::Bool,
        }
    }

    #[test]
    fn declined_speculative_branch_restores_string_cache_state() {
        let mut emitter = RustEmitter::new();
        emitter
            .string_char_cache_required_names
            .insert("text".to_string());
        emitter
            .string_char_cache_vars
            .insert("existing".to_string(), "__sifr_existing_chars".to_string());
        let before = emitter.string_char_cache_vars.clone();
        let body = vec![
            HirStmt::Let {
                name: "text".to_string(),
                ty: Type::Str,
                value: HirExpr::StringLiteral("value".to_string()),
                is_mutable: false,
            },
            HirStmt::AttributeAugAssign {
                object: "self".to_string(),
                field: "label".to_string(),
                op: "+=".to_string(),
                value: HirExpr::StringLiteral("suffix".to_string()),
            },
        ];

        let lowered = emitter
            .try_lower_if_branch_for_ir(&body)
            .expect("speculative lowering must not error");

        assert!(lowered.is_none());
        assert_eq!(emitter.string_char_cache_vars, before);
    }

    #[test]
    fn successful_speculative_branch_restores_string_cache_state() {
        let mut emitter = RustEmitter::new();
        emitter
            .string_char_cache_required_names
            .insert("text".to_string());
        let before = emitter.string_char_cache_vars.clone();
        let body = vec![HirStmt::Let {
            name: "text".to_string(),
            ty: Type::Str,
            value: HirExpr::StringLiteral("value".to_string()),
            is_mutable: false,
        }];

        let lowered = emitter
            .try_lower_if_branch_for_ir(&body)
            .expect("speculative lowering must not error");

        assert!(lowered.is_some());
        assert_eq!(emitter.string_char_cache_vars, before);
    }

    #[test]
    fn declined_isinstance_union_restores_successful_sibling_cache_state() {
        let mut emitter = RustEmitter::new();
        emitter.register_union_type(&Type::Union(vec![Type::Int, Type::Str]));
        emitter
            .string_char_cache_required_names
            .insert("text".to_string());
        emitter
            .string_char_cache_vars
            .insert("existing".to_string(), "__sifr_existing_chars".to_string());
        let before = emitter.string_char_cache_vars.clone();
        let declined_then_body = vec![HirStmt::AttributeAugAssign {
            object: "self".to_string(),
            field: "label".to_string(),
            op: "+=".to_string(),
            value: HirExpr::StringLiteral("suffix".to_string()),
        }];
        let successful_elif_body = vec![HirStmt::Let {
            name: "text".to_string(),
            ty: Type::Str,
            value: HirExpr::StringLiteral("value".to_string()),
            is_mutable: false,
        }];

        let lowered = emitter
            .try_lower_if_stmt_for_ir(
                &isinstance_union_condition("int"),
                &declined_then_body,
                &[(isinstance_union_condition("str"), successful_elif_body)],
                None,
            )
            .expect("union lowering must not error");

        assert!(lowered.is_none());
        assert_eq!(emitter.string_char_cache_vars, before);
    }
}
