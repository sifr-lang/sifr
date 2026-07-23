use super::{HirExpr, RustEmitter, RustStmt, Type};

impl RustEmitter {
    pub(crate) fn try_lower_self_string_concat_assign_for_ir(
        &mut self,
        name: &str,
        value: &HirExpr,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let mut parts = Vec::new();
        if !Self::collect_self_string_concat_parts(name, value, &mut parts) {
            return Ok(None);
        }
        if !parts.iter().all(|part| {
            matches!(
                crate::resolve_alias_type_for_plain_call(part.ty()),
                Type::Str | Type::LiteralStr(_)
            )
        }) {
            return Ok(None);
        }

        let cache_name = self.string_char_cache_vars.get(name).cloned();
        let mut stmts = Vec::with_capacity(parts.len() * 3);
        for (index, part) in parts.into_iter().enumerate() {
            let mentions_target = Self::expr_mentions_name(part, name);
            let materialize_for_cache =
                cache_name.is_some() && !matches!(part, HirExpr::StringLiteral(_));
            let single_literal_char = match part {
                HirExpr::StringLiteral(value) => Self::single_char_string_literal(value),
                _ => None,
            };
            let (push_method, push_arg, cache_chars_source) =
                if mentions_target || materialize_for_cache {
                    let Some(part_expr) = self.lower_stmt_expr_for_ir(part)? else {
                        return Err(crate::CodegenError::new(
                            "could not lower string concat assignment part",
                        ));
                    };
                    let temp_name = format!("__sifr_string_concat_{name}_{index}");
                    stmts.push(RustStmt::Let {
                        mutable: false,
                        name: temp_name.clone(),
                        ty: None,
                        value: if mentions_target {
                            crate::RustExpr::Clone(Box::new(part_expr))
                        } else {
                            part_expr
                        },
                    });
                    let as_str = crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(
                            crate::RustExpr::Ident(temp_name.clone()),
                        ))),
                        method: "as_str".to_string(),
                        args: vec![],
                    };
                    ("push_str".to_string(), as_str.clone(), as_str)
                } else {
                    let (method, arg) = self.lower_string_push_method_and_arg_for_ir(part)?;
                    (method, arg.clone(), arg)
                };
            stmts.push(RustStmt::Expr(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Ident(name.to_string())),
                method: push_method,
                args: vec![push_arg],
            }));
            if let Some(cache_name) = &cache_name {
                if let Some(ch) = single_literal_char {
                    stmts.push(RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(cache_name.clone())),
                        method: "push".to_string(),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Char(ch))],
                    }));
                } else {
                    stmts.push(RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(cache_name.clone())),
                        method: "extend".to_string(),
                        args: vec![crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                cache_chars_source,
                            ))),
                            method: "chars".to_string(),
                            args: vec![],
                        }],
                    }));
                }
            }
        }
        Ok(Some(stmts))
    }

    fn collect_self_string_concat_parts<'a>(
        name: &str,
        expr: &'a HirExpr,
        parts: &mut Vec<&'a HirExpr>,
    ) -> bool {
        let HirExpr::BinOp {
            left, op, right, ..
        } = expr
        else {
            return false;
        };
        if op != "+" {
            return false;
        }

        if matches!(left.as_ref(), HirExpr::Name { name: left_name, .. } if left_name == name) {
            parts.push(right);
            return true;
        }
        if Self::collect_self_string_concat_parts(name, left, parts) {
            parts.push(right);
            return true;
        }
        false
    }

    fn expr_mentions_name(expr: &HirExpr, needle: &str) -> bool {
        match expr {
            HirExpr::Name { name, .. } => name == needle,
            HirExpr::BinOp { left, right, .. } => {
                Self::expr_mentions_name(left, needle) || Self::expr_mentions_name(right, needle)
            }
            HirExpr::UnaryOp { operand, .. }
            | HirExpr::Await { value: operand, .. }
            | HirExpr::QuestionMark { expr: operand, .. }
            | HirExpr::OkWrap { value: operand, .. }
            | HirExpr::ErrWrap { value: operand, .. } => Self::expr_mentions_name(operand, needle),
            HirExpr::Compare {
                left, comparators, ..
            } => {
                Self::expr_mentions_name(left, needle)
                    || comparators
                        .iter()
                        .any(|expr| Self::expr_mentions_name(expr, needle))
            }
            HirExpr::BoolOp { values, .. }
            | HirExpr::Call { args: values, .. }
            | HirExpr::PythonCall { args: values, .. }
            | HirExpr::IntrinsicCall { args: values, .. }
            | HirExpr::IteratorCall { args: values, .. }
            | HirExpr::ListLiteral {
                elements: values, ..
            }
            | HirExpr::SetLiteral {
                elements: values, ..
            }
            | HirExpr::TupleLiteral {
                elements: values, ..
            }
            | HirExpr::ConstructorCall { args: values, .. }
            | HirExpr::SuperCall { args: values, .. } => values
                .iter()
                .any(|expr| Self::expr_mentions_name(expr, needle)),
            HirExpr::IfExpr {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                Self::expr_mentions_name(condition, needle)
                    || Self::expr_mentions_name(then_expr, needle)
                    || Self::expr_mentions_name(else_expr, needle)
            }
            HirExpr::RangeLiteral {
                start, end, step, ..
            } => {
                Self::expr_mentions_name(start, needle)
                    || Self::expr_mentions_name(end, needle)
                    || step
                        .as_ref()
                        .is_some_and(|expr| Self::expr_mentions_name(expr, needle))
            }
            HirExpr::DictLiteral { keys, values, .. } => keys
                .iter()
                .chain(values.iter())
                .any(|expr| Self::expr_mentions_name(expr, needle)),
            HirExpr::Index { object, index, .. } => {
                Self::expr_mentions_name(object, needle) || Self::expr_mentions_name(index, needle)
            }
            HirExpr::MethodCall { object, args, .. } => {
                Self::expr_mentions_name(object, needle)
                    || args
                        .iter()
                        .any(|expr| Self::expr_mentions_name(expr, needle))
            }
            HirExpr::ContainsOp {
                element,
                collection,
                ..
            } => {
                Self::expr_mentions_name(element, needle)
                    || Self::expr_mentions_name(collection, needle)
            }
            HirExpr::FString { parts, .. } => parts.iter().any(|part| match part {
                sifr_ir::HirFStringPart::Literal(_) => false,
                sifr_ir::HirFStringPart::Expr(expr) => Self::expr_mentions_name(expr, needle),
            }),
            HirExpr::Slice {
                object,
                start,
                stop,
                step,
                ..
            } => {
                Self::expr_mentions_name(object, needle)
                    || start
                        .as_ref()
                        .is_some_and(|expr| Self::expr_mentions_name(expr, needle))
                    || stop
                        .as_ref()
                        .is_some_and(|expr| Self::expr_mentions_name(expr, needle))
                    || step
                        .as_ref()
                        .is_some_and(|expr| Self::expr_mentions_name(expr, needle))
            }
            HirExpr::WalrusExpr { name, value, .. } => {
                name == needle || Self::expr_mentions_name(value, needle)
            }
            HirExpr::FieldAccess { object, .. } => Self::expr_mentions_name(object, needle),
            HirExpr::Lambda { body, .. } => Self::expr_mentions_name(body, needle),
            HirExpr::ListComp {
                expr, generators, ..
            }
            | HirExpr::SetComp {
                expr, generators, ..
            } => {
                Self::expr_mentions_name(expr, needle)
                    || generators.iter().any(|(_, iter, filter)| {
                        Self::expr_mentions_name(iter, needle)
                            || filter
                                .as_ref()
                                .is_some_and(|expr| Self::expr_mentions_name(expr, needle))
                    })
            }
            HirExpr::DictComp {
                key_expr,
                val_expr,
                generators,
                ..
            } => {
                Self::expr_mentions_name(key_expr, needle)
                    || Self::expr_mentions_name(val_expr, needle)
                    || generators.iter().any(|(_, iter, filter)| {
                        Self::expr_mentions_name(iter, needle)
                            || filter
                                .as_ref()
                                .is_some_and(|expr| Self::expr_mentions_name(expr, needle))
                    })
            }
            HirExpr::GeneratorExpr {
                expr, iter, filter, ..
            } => {
                Self::expr_mentions_name(expr, needle)
                    || Self::expr_mentions_name(iter, needle)
                    || filter
                        .as_ref()
                        .is_some_and(|expr| Self::expr_mentions_name(expr, needle))
            }
            HirExpr::IntLiteral(_)
            | HirExpr::LargeIntLiteral(_)
            | HirExpr::FloatLiteral(_)
            | HirExpr::StringLiteral(_)
            | HirExpr::BoolLiteral(_)
            | HirExpr::NoneLiteral
            | HirExpr::EnumVariant { .. } => false,
        }
    }

    pub(crate) fn lower_string_push_str_arg_for_ir(
        &mut self,
        value: &HirExpr,
    ) -> Result<crate::RustExpr, crate::CodegenError> {
        if let HirExpr::StringLiteral(val) = value {
            return Ok(crate::RustExpr::Verbatim(format!("{val:?}")));
        }

        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
            return Err(crate::CodegenError::new(
                "could not lower string concat assignment part",
            ));
        };
        Ok(crate::RustExpr::MethodCall {
            receiver: Box::new(crate::RustExpr::Paren(Box::new(value_expr))),
            method: "as_str".to_string(),
            args: vec![],
        })
    }

    pub(crate) fn lower_string_push_method_and_arg_for_ir(
        &mut self,
        value: &HirExpr,
    ) -> Result<(String, crate::RustExpr), crate::CodegenError> {
        if let HirExpr::StringLiteral(val) = value {
            if let Some(ch) = Self::single_char_string_literal(val) {
                return Ok((
                    "push".to_string(),
                    crate::RustExpr::Literal(crate::RustLiteral::Char(ch)),
                ));
            }
            return Ok((
                "push_str".to_string(),
                crate::RustExpr::Verbatim(format!("{val:?}")),
            ));
        }

        Ok((
            "push_str".to_string(),
            self.lower_string_push_str_arg_for_ir(value)?,
        ))
    }

    fn single_char_string_literal(value: &str) -> Option<char> {
        let mut chars = value.chars();
        let ch = chars.next()?;
        if chars.next().is_none() {
            Some(ch)
        } else {
            None
        }
    }
}
