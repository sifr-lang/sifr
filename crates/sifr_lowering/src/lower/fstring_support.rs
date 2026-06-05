use super::expressions::lower_expr;
use super::LowerCtx;
use crate::hir_nodes::{HirExpr, HirFStringPart};
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
