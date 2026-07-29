use super::{HirExpr, HirStmt, RustEmitter, RustStmt, Type};
impl RustEmitter {
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
                Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(crate::RustExpr::Ident(
                        name.clone(),
                    )))),
                    method: "as_str".to_string(),
                    args: vec![],
                })
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
            if right_borrowed
                && crate::resolve_alias_type_for_plain_call(&right_ty).ownership()
                    != sifr_type_system::OwnershipKind::Copy
            {
                lowered_right = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_right))),
                    method: "clone".to_string(),
                    args: vec![],
                };
            }
            lowered_right = crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                args: vec![lowered_right],
            };
        } else if !left_is_option && right_is_option && !left_none_like {
            if left_borrowed
                && crate::resolve_alias_type_for_plain_call(&left_ty).ownership()
                    != sifr_type_system::OwnershipKind::Copy
            {
                lowered_left = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                    method: "clone".to_string(),
                    args: vec![],
                };
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
        if elif_clauses.is_empty()
            && else_body.is_none()
            && crate::helpers::codegen_body_always_exits(then_body)
        {
            let speculative_string_char_cache_vars = self.string_char_cache_vars.clone();
            let Some(lowered_then_body) = self.try_lower_stmt_block_for_ir(then_body)? else {
                self.string_char_cache_vars = speculative_string_char_cache_vars;
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
                self.string_char_cache_vars = speculative_string_char_cache_vars;
                return Ok(Some(RustStmt::LetElse {
                    pattern,
                    value,
                    else_body: lowered_then_body,
                }));
            }
            if let Some(option_var) = crate::helpers::detect_is_none_var(condition)
                .or_else(|| crate::helpers::detect_not_option_truthiness(condition))
            {
                self.string_char_cache_vars = speculative_string_char_cache_vars;
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
                self.string_char_cache_vars = speculative_string_char_cache_vars;
                return Ok(Some(RustStmt::LetElse {
                    pattern,
                    value,
                    else_body: lowered_then_body,
                }));
            }
            self.string_char_cache_vars = speculative_string_char_cache_vars;
        }

        let mut nested_else = if let Some(else_body) = else_body {
            let Some(lowered_else) = self.try_lower_stmt_block_for_ir(else_body)? else {
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

    pub(crate) fn try_lower_if_clause_for_ir(
        &mut self,
        condition: &HirExpr,
        then_body: &[HirStmt],
        nested_else: Option<Vec<RustStmt>>,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Some(lowered_then_body) = self.try_lower_stmt_block_for_ir(then_body)? else {
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
        if vars.len() >= 2 {
            Some(vars)
        } else {
            None
        }
    }
}
