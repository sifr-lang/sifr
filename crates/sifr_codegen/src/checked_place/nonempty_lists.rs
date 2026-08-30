use super::{RustEmitter, RustExpr, RustLiteral, RustStmt, Type, expr_mentions_name};

impl RustEmitter {
    pub(crate) fn stmt_defines_nonempty_list(stmt: &crate::HirStmt) -> bool {
        matches!(
            stmt,
            crate::HirStmt::Let { value, .. }
                if nonempty_list_comprehension_depth(value).is_some_and(|depth| depth > 1)
        )
    }

    pub(crate) fn lower_nonempty_list_binding_value_for_ir(
        &mut self,
        name: &str,
        value: &crate::HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let Some(depth) = nonempty_list_comprehension_depth(value).filter(|depth| *depth > 1)
        else {
            return Ok(None);
        };
        let Some(lowered) = self.lower_nonempty_list_comprehension_for_ir(value)? else {
            return Ok(None);
        };
        self.nonempty_list_bindings.insert(name.to_string(), depth);
        Ok(Some(lowered))
    }

    fn lower_nonempty_list_comprehension_for_ir(
        &mut self,
        value: &crate::HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let crate::HirExpr::ListComp {
            expr,
            generators,
            ty,
        } = value
        else {
            return Ok(None);
        };
        let [(var, iter, None)] = generators.as_slice() else {
            return Ok(None);
        };
        if !range_executes_at_least_once(iter) {
            return Ok(None);
        }
        let rendered_var = if expr_mentions_name(expr, var) {
            var.clone()
        } else {
            format!("_{var}")
        };

        let lowered_element = if nonempty_list_comprehension_depth(expr).is_some() {
            let Some(lowered) = self.lower_nonempty_list_comprehension_for_ir(expr)? else {
                return Ok(None);
            };
            lowered
        } else {
            let Some(lowered) = self.lower_stmt_expr_for_ir(expr)? else {
                return Ok(None);
            };
            if let Type::List(element_ty) = Self::resolve_alias_type_for_loop_iter(ty) {
                crate::helpers::adapt_collection_value_for_target(
                    element_ty.as_ref(),
                    expr,
                    lowered,
                )
            } else {
                lowered
            }
        };
        let Some(lowered_iter) = self.lower_comprehension_iter_for_ir(iter)? else {
            return Ok(None);
        };
        let tail_name = format!("__sifr_nonempty_tail_{}", self.nonempty_list_bindings.len());
        let tail_iter = RustExpr::MethodCall {
            receiver: Box::new(lowered_iter),
            method: "skip".to_string(),
            args: vec![RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                ty: crate::RustType::Named("usize".to_string()),
            }],
        };
        Ok(Some(RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: false,
                    name: rendered_var.clone(),
                    ty: None,
                    value: RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "SifrInt".to_string(),
                            "from_i64".to_string(),
                        ])),
                        args: vec![RustExpr::Literal(RustLiteral::Int(0))],
                    },
                },
                RustStmt::Let {
                    mutable: true,
                    name: tail_name.clone(),
                    ty: None,
                    value: RustExpr::Vec(Vec::new()),
                },
                RustStmt::For {
                    var: rendered_var,
                    iter: tail_iter,
                    body: vec![RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(tail_name.clone())),
                        method: "push".to_string(),
                        args: vec![lowered_element.clone()],
                    })],
                },
            ],
            expr: Some(Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "sifr_runtime".to_string(),
                    "SifrNonEmptyVec".to_string(),
                    "new".to_string(),
                ])),
                args: vec![lowered_element, RustExpr::Ident(tail_name)],
            })),
        }))
    }

    pub(crate) fn lower_proven_nonempty_head_read_for_ir(
        &self,
        object: &crate::HirExpr,
        index: &crate::HirExpr,
        result_ty: &Type,
    ) -> Option<RustExpr> {
        if !is_zero_index(index) {
            return None;
        }
        let (mut lowered, remaining_depth) = self.nonempty_object_expr(object)?;
        if remaining_depth == 0 {
            return None;
        }
        lowered = RustExpr::Field {
            expr: Box::new(lowered),
            field: "head".to_string(),
        };
        if crate::helpers::is_copy_type_for_codegen(result_ty) {
            Some(lowered)
        } else {
            Some(RustExpr::MethodCall {
                receiver: Box::new(lowered),
                method: "clone".to_string(),
                args: Vec::new(),
            })
        }
    }

    fn nonempty_object_expr(&self, object: &crate::HirExpr) -> Option<(RustExpr, usize)> {
        match object {
            crate::HirExpr::Name { name, .. } => Some((
                RustExpr::Ident(name.clone()),
                *self.nonempty_list_bindings.get(name)?,
            )),
            crate::HirExpr::Index { object, index, .. } if is_zero_index(index) => {
                let (lowered_parent, depth) = self.nonempty_object_expr(object)?;
                if depth == 0 {
                    return None;
                }
                let lowered = RustExpr::Field {
                    expr: Box::new(lowered_parent),
                    field: "head".to_string(),
                };
                Some((lowered, depth - 1))
            }
            _ => None,
        }
    }
}

fn nonempty_list_comprehension_depth(value: &crate::HirExpr) -> Option<usize> {
    let crate::HirExpr::ListComp {
        expr, generators, ..
    } = value
    else {
        return None;
    };
    let [(_, iter, None)] = generators.as_slice() else {
        return None;
    };
    if !range_executes_at_least_once(iter) {
        return None;
    }
    Some(nonempty_list_comprehension_depth(expr).map_or(1, |inner_depth| inner_depth + 1))
}

fn range_executes_at_least_once(iter: &crate::HirExpr) -> bool {
    let iter = match iter {
        crate::HirExpr::IteratorCall { op, args, .. }
            if matches!(op, sifr_ir::HirIteratorOp::Iter) && args.len() == 1 =>
        {
            &args[0]
        }
        other => other,
    };
    let crate::HirExpr::RangeLiteral {
        start, end, step, ..
    } = iter
    else {
        return false;
    };
    if step.is_some() || !is_zero_index(start) {
        return false;
    }
    match end.as_ref() {
        crate::HirExpr::IntLiteral(value) => *value > 0,
        crate::HirExpr::LargeIntLiteral(value) => {
            value.parse::<i128>().is_ok_and(|value| value > 0)
        }
        crate::HirExpr::BinOp { op, right, .. } if op == "+" => match right.as_ref() {
            crate::HirExpr::IntLiteral(value) => *value > 0,
            crate::HirExpr::LargeIntLiteral(value) => {
                value.parse::<i128>().is_ok_and(|value| value > 0)
            }
            _ => false,
        },
        _ => false,
    }
}

fn is_zero_index(index: &crate::HirExpr) -> bool {
    matches!(index, crate::HirExpr::IntLiteral(0))
        || matches!(index, crate::HirExpr::LargeIntLiteral(value) if value == "0")
}
