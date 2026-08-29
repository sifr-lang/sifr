use super::render_borrowed_arg_expr;
use crate::{RustExpr, RustParam, RustType};

pub(super) fn lower_find_like(
    object: &RustExpr,
    args: &[RustExpr],
    rust_method: &str,
) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: rust_method.to_string(),
            args: vec![render_borrowed_arg_expr(&args[0])],
        }),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "i".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "SifrInt".to_string(),
                    "from".to_string(),
                ])),
                args: vec![RustExpr::Ident("i".to_string())],
            }),
            is_move: false,
        }],
    })
}
