use crate::{RustEmitter, RustExpr};
use sifr_hir::{HirExpr, HirFStringPart};

impl RustEmitter {
    pub(super) fn render_expr_with_lowered_fallback(&mut self, expr: &HirExpr) -> String {
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
