//! One allocation policy for displaying a typed optional source value.
use crate::{RustExpr, RustLiteral, RustParam, RustType};

pub(crate) fn lower(value: RustExpr, format_str: impl Into<String>, binding: &str) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Paren(Box::new(value))),
        method: "map_or_else".to_string(),
        args: vec![
            RustExpr::Closure {
                params: vec![],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Literal(RustLiteral::Str("None".to_string()))),
                    method: "to_string".to_string(),
                    args: vec![],
                }),
                is_move: false,
            },
            RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: binding.to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str: format_str.into(),
                    args: vec![RustExpr::Ident(binding.to_string())],
                }),
                is_move: false,
            },
        ],
    }
}
