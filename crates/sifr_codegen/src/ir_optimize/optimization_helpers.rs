use super::{RustExpr, RustLiteral, RustType};

pub(crate) fn not_expr(expr: RustExpr) -> RustExpr {
    match expr {
        RustExpr::Literal(RustLiteral::Bool(value)) => RustExpr::Literal(RustLiteral::Bool(!value)),
        RustExpr::UnaryOp { op, operand } if op == "!" => *operand,
        other => RustExpr::UnaryOp {
            op: "!".to_string(),
            operand: Box::new(other),
        },
    }
}

pub(crate) fn is_copy_type(ty: &RustType) -> bool {
    match ty {
        RustType::I64 | RustType::F64 | RustType::Bool | RustType::Unit => true,
        RustType::Ref { .. } => true,
        RustType::Tuple(items) => items.iter().all(is_copy_type),
        _ => false,
    }
}

#[cfg(test)]
#[path = "../../tests/unit/ir_optimize/optimization_helpers/tests.rs"]
mod tests;
