use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::visitor::{self, Visitor};
use sifr_python_ast::{Expr, Number, Stmt};

use super::LowerCtx;
use super::integer_literals::canonical_large_int_literal_text;

pub(in crate::lower) const INTEGER_EVAL_DECIMAL_DIGIT_BUDGET: usize = 4096;

pub(in crate::lower) fn validate_module_integer_literals(stmts: &[Stmt], ctx: &mut LowerCtx) {
    let mut visitor = IntegerLiteralBudgetVisitor { ctx };
    visitor::walk_body(&mut visitor, stmts);
}

struct IntegerLiteralBudgetVisitor<'ctx> {
    ctx: &'ctx mut LowerCtx,
}

impl<'a> Visitor<'a> for IntegerLiteralBudgetVisitor<'_> {
    fn visit_expr(&mut self, expr: &'a Expr) {
        if let Expr::NumberLiteral(number_literal) = expr {
            if let Number::Int(value) = &number_literal.value {
                validate_integer_literal_budget(value, number_literal.range(), self.ctx);
            }
        }
        visitor::walk_expr(self, expr);
    }
}

fn validate_integer_literal_budget(
    value: &sifr_python_ast::Int,
    range: ruff_text_size::TextRange,
    ctx: &mut LowerCtx,
) {
    if value.as_i64().is_some() {
        return;
    }

    let canonical = canonical_large_int_literal_text(value);
    let digits = canonical.len();
    if digits <= INTEGER_EVAL_DECIMAL_DIGIT_BUDGET {
        return;
    }

    ctx.error_with_code_at(
        DiagnosticCode::INT_EVAL_BUDGET_EXCEEDED,
        format!(
            "integer literal exceeds compile-time evaluation budget: {digits} decimal digits (max {INTEGER_EVAL_DECIMAL_DIGIT_BUDGET})"
        ),
        range,
    );
}
