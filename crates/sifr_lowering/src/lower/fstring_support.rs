use super::LowerCtx;
use super::expressions::{is_poisoned_binding_expr, lower_expr};
use super::type_bounds::supports_print_formatting;
use crate::hir_nodes::{HirExpr, HirFStringPart};
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{ExprFString, InterpolatedStringElement};
use sifr_type_system::Type;

pub(in crate::lower) fn lower_fstring_expr(
    fstring: &ExprFString,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let mut parts = Vec::new();

    for part in &fstring.value {
        match part {
            sifr_python_ast::FStringPart::Literal(s) => {
                parts.push(HirFStringPart::Literal(s.to_string()));
            }
            sifr_python_ast::FStringPart::FString(fs) => {
                for element in &fs.elements {
                    match element {
                        InterpolatedStringElement::Literal(lit) => {
                            parts.push(HirFStringPart::Literal(lit.value.to_string()));
                        }
                        InterpolatedStringElement::Interpolation(expr_elem) => {
                            let expr = lower_expr(&expr_elem.expression, ctx)?;
                            if is_poisoned_binding_expr(&expr, ctx) {
                                return None;
                            }
                            if !supports_print_formatting(expr.ty())
                                && super::python_interop::python_context_borrow_reference(
                                    &expr, ctx,
                                )
                                .is_none()
                            {
                                ctx.error_with_code_at(
                                    DiagnosticCode::TYPE_MISMATCH,
                                    format!(
                                        "f-string value type '{}' lacks the generated Rust formatting trait required by codegen",
                                        expr.ty().display_name()
                                    ),
                                    expr_elem.expression.range(),
                                );
                                return None;
                            }
                            parts.push(HirFStringPart::Expr(expr));
                        }
                    }
                }
            }
        }
    }

    Some(HirExpr::FString {
        parts,
        ty: Type::Str,
    })
}
