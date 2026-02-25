use crate::RustEmitter;
use sifr_hir::{HirExpr, HirFStringPart};

impl RustEmitter {
    pub(super) fn render_expr_via_fallback_only(&mut self, expr: &HirExpr) -> String {
        let saved_output = std::mem::take(&mut self.output);
        let saved_indent = self.indent;
        self.indent = 0;
        self.emit_expr(expr);
        let result = std::mem::take(&mut self.output);
        self.output = saved_output;
        self.indent = saved_indent;
        result.trim().to_string()
    }

    pub(super) fn try_lower_registry_expr_result(
        &self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if self.should_force_render_fallback(expr) {
            return Ok(None);
        }
        Ok(
            crate::try_lower_leaf_expr_result(expr)?
                .map(|lowered| self.rewrite_stdlib_constant_idents_in_expr(lowered)),
        )
    }

    pub(super) fn render_expr_with_lowered_fallback(&mut self, expr: &HirExpr) -> String {
        match self.try_lower_registry_expr_result(expr) {
            Ok(Some(lowered_expr)) => crate::render_expr(&lowered_expr),
            Ok(None) => self.render_expr_via_fallback_only(expr),
            Err(_) => {
                self.lowering_stats.expr_lowering_errors += 1;
                self.render_expr_via_fallback_only(expr)
            }
        }
    }

    pub(super) fn should_force_render_fallback(&self, expr: &HirExpr) -> bool {
        if render_expr_contains_force_fallback_name(self, expr) {
            return true;
        }
        matches!(expr, HirExpr::Compare { .. } | HirExpr::BoolOp { .. })
            && render_expr_uses_borrowed_param(
                expr,
                &self.borrowed_params,
                &self.mut_borrowed_params,
            )
    }

    fn rewrite_stdlib_constant_idents_in_expr(&self, expr: crate::RustExpr) -> crate::RustExpr {
        match expr {
            crate::RustExpr::Ident(name) => self.rewrite_stdlib_constant_ident(name),
            crate::RustExpr::MethodCall {
                receiver,
                method,
                args,
            } => crate::RustExpr::MethodCall {
                receiver: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*receiver)),
                method,
                args: args
                    .into_iter()
                    .map(|arg| self.rewrite_stdlib_constant_idents_in_expr(arg))
                    .collect(),
            },
            crate::RustExpr::FnCall { func, args } => crate::RustExpr::FnCall {
                func: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*func)),
                args: args
                    .into_iter()
                    .map(|arg| self.rewrite_stdlib_constant_idents_in_expr(arg))
                    .collect(),
            },
            crate::RustExpr::MacroCall { name, args } => crate::RustExpr::MacroCall {
                name,
                args: args
                    .into_iter()
                    .map(|arg| self.rewrite_stdlib_constant_idents_in_expr(arg))
                    .collect(),
            },
            crate::RustExpr::FormatMacro {
                name,
                format_str,
                args,
            } => crate::RustExpr::FormatMacro {
                name,
                format_str,
                args: args
                    .into_iter()
                    .map(|arg| self.rewrite_stdlib_constant_idents_in_expr(arg))
                    .collect(),
            },
            crate::RustExpr::BinOp { left, op, right } => crate::RustExpr::BinOp {
                left: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*left)),
                op,
                right: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*right)),
            },
            crate::RustExpr::UnaryOp { op, operand } => crate::RustExpr::UnaryOp {
                op,
                operand: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*operand)),
            },
            crate::RustExpr::Field { expr, field } => crate::RustExpr::Field {
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
                field,
            },
            crate::RustExpr::Index { expr, index } => crate::RustExpr::Index {
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
                index: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*index)),
            },
            crate::RustExpr::Slice { expr, start, stop } => crate::RustExpr::Slice {
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
                start: start
                    .map(|part| Box::new(self.rewrite_stdlib_constant_idents_in_expr(*part))),
                stop: stop.map(|part| Box::new(self.rewrite_stdlib_constant_idents_in_expr(*part))),
            },
            crate::RustExpr::Ref { mutable, expr } => crate::RustExpr::Ref {
                mutable,
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
            },
            crate::RustExpr::Deref(expr) => crate::RustExpr::Deref(Box::new(
                self.rewrite_stdlib_constant_idents_in_expr(*expr),
            )),
            crate::RustExpr::Clone(expr) => crate::RustExpr::Clone(Box::new(
                self.rewrite_stdlib_constant_idents_in_expr(*expr),
            )),
            crate::RustExpr::Cast { expr, ty } => crate::RustExpr::Cast {
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
                ty,
            },
            crate::RustExpr::Block { stmts, expr } => crate::RustExpr::Block {
                stmts: stmts
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
                expr: expr.map(|inner| {
                    Box::new(self.rewrite_stdlib_constant_idents_in_expr(*inner))
                }),
            },
            crate::RustExpr::If {
                cond,
                then_expr,
                else_expr,
            } => crate::RustExpr::If {
                cond: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*cond)),
                then_expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*then_expr)),
                else_expr: else_expr.map(|inner| {
                    Box::new(self.rewrite_stdlib_constant_idents_in_expr(*inner))
                }),
            },
            crate::RustExpr::Match { expr, arms } => crate::RustExpr::Match {
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
                arms: arms
                    .into_iter()
                    .map(|arm| crate::RustMatchArm {
                        pattern: arm.pattern,
                        bindings: arm.bindings,
                        guard: arm
                            .guard
                            .map(|guard| self.rewrite_stdlib_constant_idents_in_expr(guard)),
                        body: arm
                            .body
                            .into_iter()
                            .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                            .collect(),
                    })
                    .collect(),
            },
            crate::RustExpr::Closure {
                params,
                body,
                is_move,
            } => crate::RustExpr::Closure {
                params,
                body: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*body)),
                is_move,
            },
            crate::RustExpr::ClosureBlock {
                params,
                body,
                is_move,
            } => crate::RustExpr::ClosureBlock {
                params,
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
                is_move,
            },
            crate::RustExpr::StructInit { name, fields } => crate::RustExpr::StructInit {
                name,
                fields: fields
                    .into_iter()
                    .map(|(field, value)| {
                        (field, self.rewrite_stdlib_constant_idents_in_expr(value))
                    })
                    .collect(),
            },
            crate::RustExpr::Tuple(items) => crate::RustExpr::Tuple(
                items
                    .into_iter()
                    .map(|item| self.rewrite_stdlib_constant_idents_in_expr(item))
                    .collect(),
            ),
            crate::RustExpr::Vec(items) => crate::RustExpr::Vec(
                items
                    .into_iter()
                    .map(|item| self.rewrite_stdlib_constant_idents_in_expr(item))
                    .collect(),
            ),
            crate::RustExpr::Try(expr) => {
                crate::RustExpr::Try(Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)))
            }
            crate::RustExpr::Await(expr) => crate::RustExpr::Await(Box::new(
                self.rewrite_stdlib_constant_idents_in_expr(*expr),
            )),
            crate::RustExpr::Paren(expr) => crate::RustExpr::Paren(Box::new(
                self.rewrite_stdlib_constant_idents_in_expr(*expr),
            )),
            crate::RustExpr::Range { start, end } => crate::RustExpr::Range {
                start: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*start)),
                end: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*end)),
            },
            crate::RustExpr::Literal(lit) => crate::RustExpr::Literal(lit),
            crate::RustExpr::Path(path) => crate::RustExpr::Path(path),
            crate::RustExpr::RawCode(code) => crate::RustExpr::RawCode(code),
        }
    }

    fn rewrite_stdlib_constant_idents_in_stmt(&self, stmt: crate::RustStmt) -> crate::RustStmt {
        match stmt {
            crate::RustStmt::Let {
                mutable,
                name,
                ty,
                value,
            } => crate::RustStmt::Let {
                mutable,
                name,
                ty,
                value: self.rewrite_stdlib_constant_idents_in_expr(value),
            },
            crate::RustStmt::LetPattern { pattern, value } => crate::RustStmt::LetPattern {
                pattern,
                value: self.rewrite_stdlib_constant_idents_in_expr(value),
            },
            crate::RustStmt::Assign { target, value } => crate::RustStmt::Assign {
                target: self.rewrite_stdlib_constant_idents_in_expr(target),
                value: self.rewrite_stdlib_constant_idents_in_expr(value),
            },
            crate::RustStmt::AugAssign { target, op, value } => crate::RustStmt::AugAssign {
                target: self.rewrite_stdlib_constant_idents_in_expr(target),
                op,
                value: self.rewrite_stdlib_constant_idents_in_expr(value),
            },
            crate::RustStmt::Expr(expr) => {
                crate::RustStmt::Expr(self.rewrite_stdlib_constant_idents_in_expr(expr))
            }
            crate::RustStmt::Assert { cond, msg } => crate::RustStmt::Assert {
                cond: self.rewrite_stdlib_constant_idents_in_expr(cond),
                msg: msg.map(|msg| self.rewrite_stdlib_constant_idents_in_expr(msg)),
            },
            crate::RustStmt::Return(expr) => crate::RustStmt::Return(
                expr.map(|ret| self.rewrite_stdlib_constant_idents_in_expr(ret)),
            ),
            crate::RustStmt::If {
                cond,
                then_body,
                else_body,
            } => crate::RustStmt::If {
                cond: self.rewrite_stdlib_constant_idents_in_expr(cond),
                then_body: then_body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
                else_body: else_body.map(|body| {
                    body.into_iter()
                        .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                        .collect()
                }),
            },
            crate::RustStmt::IfLet {
                pattern,
                expr,
                then_body,
                else_body,
            } => crate::RustStmt::IfLet {
                pattern,
                expr: self.rewrite_stdlib_constant_idents_in_expr(expr),
                then_body: then_body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
                else_body: else_body.map(|body| {
                    body.into_iter()
                        .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                        .collect()
                }),
            },
            crate::RustStmt::Match { expr, arms } => crate::RustStmt::Match {
                expr: self.rewrite_stdlib_constant_idents_in_expr(expr),
                arms: arms
                    .into_iter()
                    .map(|arm| crate::RustMatchArm {
                        pattern: arm.pattern,
                        bindings: arm.bindings,
                        guard: arm
                            .guard
                            .map(|guard| self.rewrite_stdlib_constant_idents_in_expr(guard)),
                        body: arm
                            .body
                            .into_iter()
                            .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                            .collect(),
                    })
                    .collect(),
            },
            crate::RustStmt::For { var, iter, body } => crate::RustStmt::For {
                var,
                iter: self.rewrite_stdlib_constant_idents_in_expr(iter),
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            },
            crate::RustStmt::While { cond, body } => crate::RustStmt::While {
                cond: self.rewrite_stdlib_constant_idents_in_expr(cond),
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            },
            crate::RustStmt::Loop { body } => crate::RustStmt::Loop {
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            },
            crate::RustStmt::Block(stmts) => crate::RustStmt::Block(
                stmts
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            ),
            crate::RustStmt::Break
            | crate::RustStmt::Continue
            | crate::RustStmt::RawCode(_) => stmt,
        }
    }

    fn rewrite_stdlib_constant_ident(&self, name: String) -> crate::RustExpr {
        if !self.is_stdlib_constant(&name) {
            return crate::RustExpr::Ident(name);
        }
        match name.as_str() {
            "pi" => crate::RustExpr::Path(vec![
                "std".to_string(),
                "f64".to_string(),
                "consts".to_string(),
                "PI".to_string(),
            ]),
            "e" => crate::RustExpr::Path(vec![
                "std".to_string(),
                "f64".to_string(),
                "consts".to_string(),
                "E".to_string(),
            ]),
            "tau" => crate::RustExpr::Path(vec![
                "std".to_string(),
                "f64".to_string(),
                "consts".to_string(),
                "TAU".to_string(),
            ]),
            "inf" => crate::RustExpr::Path(vec!["f64".to_string(), "INFINITY".to_string()]),
            "nan" => crate::RustExpr::Path(vec!["f64".to_string(), "NAN".to_string()]),
            _ => crate::RustExpr::Ident(name),
        }
    }
}

fn render_expr_contains_force_fallback_name(emitter: &RustEmitter, expr: &HirExpr) -> bool {
    match expr {
        HirExpr::Name { name, .. } => {
            (emitter.intrinsic_functions.contains(name.as_str())
                && !is_stdlib_math_constant(name))
                || emitter.module_constants.contains_key(name)
        }
        HirExpr::BinOp { left, right, .. } => {
            render_expr_contains_force_fallback_name(emitter, left)
                || render_expr_contains_force_fallback_name(emitter, right)
        }
        HirExpr::UnaryOp { operand, .. } => {
            render_expr_contains_force_fallback_name(emitter, operand)
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            render_expr_contains_force_fallback_name(emitter, left)
                || comparators
                    .iter()
                    .any(|expr| render_expr_contains_force_fallback_name(emitter, expr))
        }
        HirExpr::BoolOp { values, .. } => values
            .iter()
            .any(|expr| render_expr_contains_force_fallback_name(emitter, expr)),
        HirExpr::Call { args, .. } => args
            .iter()
            .any(|expr| render_expr_contains_force_fallback_name(emitter, expr)),
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            render_expr_contains_force_fallback_name(emitter, condition)
                || render_expr_contains_force_fallback_name(emitter, then_expr)
                || render_expr_contains_force_fallback_name(emitter, else_expr)
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            render_expr_contains_force_fallback_name(emitter, start)
                || render_expr_contains_force_fallback_name(emitter, end)
                || step
                    .as_ref()
                    .is_some_and(|expr| render_expr_contains_force_fallback_name(emitter, expr))
        }
        HirExpr::ListLiteral { elements, .. }
        | HirExpr::SetLiteral { elements, .. }
        | HirExpr::TupleLiteral { elements, .. } => elements
            .iter()
            .any(|expr| render_expr_contains_force_fallback_name(emitter, expr)),
        HirExpr::DictLiteral { keys, values, .. } => {
            keys.iter()
                .any(|expr| render_expr_contains_force_fallback_name(emitter, expr))
                || values
                    .iter()
                    .any(|expr| render_expr_contains_force_fallback_name(emitter, expr))
        }
        HirExpr::Index { object, index, .. } => {
            render_expr_contains_force_fallback_name(emitter, object)
                || render_expr_contains_force_fallback_name(emitter, index)
        }
        HirExpr::MethodCall { object, args, .. } => {
            render_expr_contains_force_fallback_name(emitter, object)
                || args
                    .iter()
                    .any(|expr| render_expr_contains_force_fallback_name(emitter, expr))
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => {
            render_expr_contains_force_fallback_name(emitter, element)
                || render_expr_contains_force_fallback_name(emitter, collection)
        }
        HirExpr::FString { parts, .. } => parts.iter().any(|part| {
            matches!(
                part,
                HirFStringPart::Expr(expr)
                    if render_expr_contains_force_fallback_name(emitter, expr)
            )
        }),
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            render_expr_contains_force_fallback_name(emitter, object)
                || start
                    .as_ref()
                    .is_some_and(|expr| render_expr_contains_force_fallback_name(emitter, expr))
                || stop
                    .as_ref()
                    .is_some_and(|expr| render_expr_contains_force_fallback_name(emitter, expr))
                || step
                    .as_ref()
                    .is_some_and(|expr| render_expr_contains_force_fallback_name(emitter, expr))
        }
        HirExpr::WalrusExpr { value, .. } => {
            render_expr_contains_force_fallback_name(emitter, value)
        }
        HirExpr::FieldAccess { object, .. } => {
            render_expr_contains_force_fallback_name(emitter, object)
        }
        HirExpr::ConstructorCall { args, .. } => args
            .iter()
            .any(|expr| render_expr_contains_force_fallback_name(emitter, expr)),
        HirExpr::QuestionMark { expr, .. } => {
            render_expr_contains_force_fallback_name(emitter, expr)
        }
        HirExpr::OkWrap { value, .. } | HirExpr::ErrWrap { value, .. } => {
            render_expr_contains_force_fallback_name(emitter, value)
        }
        HirExpr::SuperCall { args, .. } => args
            .iter()
            .any(|expr| render_expr_contains_force_fallback_name(emitter, expr)),
        HirExpr::Lambda { body, .. } => render_expr_contains_force_fallback_name(emitter, body),
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            render_expr_contains_force_fallback_name(emitter, expr)
                || generators.iter().any(|(_, iter_expr, maybe_filter)| {
                    render_expr_contains_force_fallback_name(emitter, iter_expr)
                        || maybe_filter.as_ref().is_some_and(|filter| {
                            render_expr_contains_force_fallback_name(emitter, filter)
                        })
                })
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            render_expr_contains_force_fallback_name(emitter, key_expr)
                || render_expr_contains_force_fallback_name(emitter, val_expr)
                || generators.iter().any(|(_, iter_expr, maybe_filter)| {
                    render_expr_contains_force_fallback_name(emitter, iter_expr)
                        || maybe_filter.as_ref().is_some_and(|filter| {
                            render_expr_contains_force_fallback_name(emitter, filter)
                        })
                })
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            render_expr_contains_force_fallback_name(emitter, expr)
                || render_expr_contains_force_fallback_name(emitter, iter)
                || filter
                    .as_ref()
                    .is_some_and(|cond| render_expr_contains_force_fallback_name(emitter, cond))
        }
        HirExpr::IntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::EnumVariant { .. } => false,
    }
}

fn is_stdlib_math_constant(name: &str) -> bool {
    matches!(name, "pi" | "e" | "tau" | "inf" | "nan")
}

impl RustEmitter {
    pub(super) fn emit_lambda_untyped(&mut self, expr: &HirExpr) {
        if let HirExpr::Lambda { params, body, .. } = expr {
            self.write("|");
            for (i, param) in params.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.write(&param.name);
            }
            self.write("| ");
            self.emit_expr(body);
        } else {
            // Not a lambda, emit as-is
            self.emit_expr(expr);
        }
    }

    pub(super) fn emit_fstring_macro(&mut self, macro_name: &str, parts: &[HirFStringPart]) {
        let mut format_str = String::new();
        let mut exprs: Vec<&HirExpr> = Vec::new();
        for part in parts {
            match part {
                HirFStringPart::Literal(s) => {
                    // Escape braces in the literal for Rust's format!
                    for ch in s.chars() {
                        match ch {
                            '{' => format_str.push_str("{{"),
                            '}' => format_str.push_str("}}"),
                            _ => format_str.push(ch),
                        }
                    }
                }
                HirFStringPart::Expr(expr) => {
                    format_str.push_str("{}");
                    exprs.push(expr);
                }
            }
        }
        self.write(macro_name);
        self.write("(\"");
        self.write(&format_str);
        self.write("\"");
        for expr in &exprs {
            self.write(", ");
            self.emit_display_expr(expr);
        }
        self.write(")");
    }
}

fn render_expr_uses_borrowed_param(
    expr: &HirExpr,
    borrowed_params: &std::collections::HashSet<String>,
    mut_borrowed_params: &std::collections::HashSet<String>,
) -> bool {
    match expr {
        HirExpr::Name { name, .. } => {
            borrowed_params.contains(name) || mut_borrowed_params.contains(name)
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            render_expr_uses_borrowed_param(left, borrowed_params, mut_borrowed_params)
                || comparators.iter().any(|c| {
                    render_expr_uses_borrowed_param(c, borrowed_params, mut_borrowed_params)
                })
        }
        HirExpr::BoolOp { values, .. } => values
            .iter()
            .any(|v| render_expr_uses_borrowed_param(v, borrowed_params, mut_borrowed_params)),
        HirExpr::UnaryOp { operand, .. } => {
            render_expr_uses_borrowed_param(operand, borrowed_params, mut_borrowed_params)
        }
        HirExpr::BinOp { left, right, .. } => {
            render_expr_uses_borrowed_param(left, borrowed_params, mut_borrowed_params)
                || render_expr_uses_borrowed_param(right, borrowed_params, mut_borrowed_params)
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            render_expr_uses_borrowed_param(condition, borrowed_params, mut_borrowed_params)
                || render_expr_uses_borrowed_param(then_expr, borrowed_params, mut_borrowed_params)
                || render_expr_uses_borrowed_param(else_expr, borrowed_params, mut_borrowed_params)
        }
        _ => false,
    }
}
