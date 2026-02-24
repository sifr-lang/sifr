use crate::{RustEmitter, RustExpr};
use sifr_hir::{HirExpr, HirFStringPart};

impl RustEmitter {
    pub(super) fn render_expr_with_lowered_fallback(&mut self, expr: &HirExpr) -> String {
        if self.should_force_render_fallback(expr) {
            let saved_output = std::mem::take(&mut self.output);
            let saved_indent = self.indent;
            self.indent = 0;
            self.emit_expr(expr);
            let result = std::mem::take(&mut self.output);
            self.output = saved_output;
            self.indent = saved_indent;
            return result.trim().to_string();
        }
        if let Some(lowered_expr) = crate::try_lower_leaf_expr(expr) {
            crate::render_expr(&lowered_expr)
        } else {
            let saved_output = std::mem::take(&mut self.output);
            let saved_indent = self.indent;
            self.indent = 0;
            self.emit_expr(expr);
            let result = std::mem::take(&mut self.output);
            self.output = saved_output;
            self.indent = saved_indent;
            result.trim().to_string()
        }
    }

    fn should_force_render_fallback(&self, expr: &HirExpr) -> bool {
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
}

fn render_expr_contains_force_fallback_name(emitter: &RustEmitter, expr: &HirExpr) -> bool {
    match expr {
        HirExpr::Name { name, .. } => {
            emitter.intrinsic_functions.contains(name.as_str())
                || emitter.is_stdlib_constant(name)
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

impl RustEmitter {
    pub(super) fn try_capture_fallback_expr_as_raw(&mut self, expr: &HirExpr) -> Option<RustExpr> {
        if !matches!(
            expr,
            HirExpr::Call { .. }
                | HirExpr::ConstructorCall { .. }
                | HirExpr::DictComp { .. }
                | HirExpr::DictLiteral { .. }
                | HirExpr::FString { .. }
                | HirExpr::GeneratorExpr { .. }
                | HirExpr::Index { .. }
                | HirExpr::Lambda { .. }
                | HirExpr::ListComp { .. }
                | HirExpr::MethodCall { .. }
                | HirExpr::SetComp { .. }
                | HirExpr::SetLiteral { .. }
                | HirExpr::Slice { .. }
        ) {
            return None;
        }

        let saved_output = std::mem::take(&mut self.output);
        let saved_indent = self.indent;
        let saved_fallback_depth = self.fallback_depth;

        self.output = String::new();
        self.indent = 0;
        self.fallback_depth += 1;
        self.emit_expr(expr);
        let captured = std::mem::take(&mut self.output);

        self.output = saved_output;
        self.indent = saved_indent;
        self.fallback_depth = saved_fallback_depth;

        Some(RustExpr::RawCode(captured.trim().to_string()))
    }

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
