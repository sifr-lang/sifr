use super::{HirExpr, RustExpr};
use std::str::FromStr as _;

pub(super) fn decimal_literal_expr(argument: &HirExpr) -> Option<RustExpr> {
    let HirExpr::StringLiteral(source) = argument else {
        return None;
    };
    let value = rust_decimal::Decimal::from_str_exact(source).ok()?;
    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "Decimal".to_string(),
            "from_i128_with_scale".to_string(),
        ])),
        args: vec![
            RustExpr::Verbatim(format!("{}_i128", value.mantissa())),
            RustExpr::Literal(crate::RustLiteral::Int(i64::from(value.scale()))),
        ],
    })
}

pub(super) fn bigdecimal_literal_expr(argument: &HirExpr) -> Option<RustExpr> {
    let HirExpr::StringLiteral(source) = argument else {
        return None;
    };
    let value = bigdecimal::BigDecimal::from_str(source).ok()?;
    let (coefficient, scale) = value.as_bigint_and_exponent();
    let bytes = coefficient
        .to_signed_bytes_be()
        .into_iter()
        .map(|byte| RustExpr::Literal(crate::RustLiteral::Int(i64::from(byte))))
        .collect();
    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "BigDecimal".to_string(),
            "new".to_string(),
        ])),
        args: vec![
            RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "bigdecimal".to_string(),
                    "num_bigint".to_string(),
                    "BigInt".to_string(),
                    "from_signed_bytes_be".to_string(),
                ])),
                args: vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Vec(bytes)),
                }],
            },
            RustExpr::Literal(crate::RustLiteral::Int(scale)),
        ],
    })
}
