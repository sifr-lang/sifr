use crate::{RustExpr, RustLiteral};

pub(crate) fn lower_checked_add(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    let rhs = args.first()?;
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Paren(Box::new(object.clone()))),
            method: "checked_add".to_string(),
            args: vec![rhs.clone()],
        }),
        method: "ok_or_else".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![],
            body: Box::new(RustExpr::StructInit {
                name: "OverflowError".to_string(),
                fields: vec![(
                    "message".to_string(),
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Literal(RustLiteral::Str(
                            "fixed-width integer addition overflow".to_string(),
                        ))),
                        method: "to_string".to_string(),
                        args: vec![],
                    },
                )],
            }),
            is_move: false,
        }],
    })
}

pub(crate) fn lower_wrapping_add(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_primitive_method(object, "wrapping_add", args)
}

pub(crate) fn lower_saturating_add(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_primitive_method(object, "saturating_add", args)
}

pub(crate) fn lower_overflowing_add(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_primitive_method(object, "overflowing_add", args)
}

fn lower_primitive_method(object: &RustExpr, method: &str, args: &[RustExpr]) -> Option<RustExpr> {
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Paren(Box::new(object.clone()))),
        method: method.to_string(),
        args: vec![args.first()?.clone()],
    })
}
