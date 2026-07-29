use super::integer_literals::canonical_large_int_literal_text;
use super::{HirExpr, Type};
use sifr_python_ast::{ExprBytesLiteral, ExprNumberLiteral, Number};

pub(in crate::lower) fn lower_number_literal(num: &ExprNumberLiteral) -> Option<HirExpr> {
    match &num.value {
        Number::Int(i) => {
            if let Some(val) = i.as_i64() {
                Some(HirExpr::IntLiteral(val))
            } else {
                Some(HirExpr::LargeIntLiteral(canonical_large_int_literal_text(
                    i,
                )))
            }
        }
        Number::Float(f) => Some(HirExpr::FloatLiteral(*f)),
        Number::Complex { .. } => None,
    }
}

pub(in crate::lower) fn lower_bytes_literal(bytes: &ExprBytesLiteral) -> HirExpr {
    let mut elements = Vec::new();
    for part in &bytes.value {
        for value in part.as_slice() {
            elements.push(HirExpr::IntLiteral(i64::from(*value)));
        }
    }
    HirExpr::ListLiteral {
        elements,
        ty: Type::Bytes,
    }
}
