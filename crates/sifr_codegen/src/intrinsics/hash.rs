//! Hash intrinsic lowerers for registry migration.

use crate::RustExpr;

pub(super) fn lower_sha256(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "{:x}".to_string(),
        args: vec![RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(
                "<sha2::Sha256 as sha2::Digest>::digest".to_string(),
            )),
            args: vec![RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(args[0].clone())),
                method: "as_bytes".to_string(),
                args: vec![],
            }],
        }],
    })
}

pub(super) fn lower_md5(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    // format!("{:x}", md5::compute(arg.as_bytes()))
    Some(RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "{:x}".to_string(),
        args: vec![RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "md5".to_string(),
                "compute".to_string(),
            ])),
            args: vec![RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(args[0].clone())),
                method: "as_bytes".to_string(),
                args: vec![],
            }],
        }],
    })
}
