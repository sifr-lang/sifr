//! Hashlib intrinsic lowerers for registry migration.

use crate::RustExpr;

fn arg_expr(args: &[String], idx: usize) -> RustExpr {
    RustExpr::Ident(args[idx].clone())
}

fn ref_expr(expr: RustExpr) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(expr),
    }
}

fn digest_bytes_call(path: Vec<&str>, arg: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(
            path.into_iter().map(std::string::ToString::to_string).collect(),
        )),
        args: vec![RustExpr::MethodCall {
            receiver: Box::new(ref_expr(arg)),
            method: "as_bytes".to_string(),
            args: vec![],
        }],
    }
}

fn hex_format(expr: RustExpr) -> RustExpr {
    RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "{:x}".to_string(),
        args: vec![expr],
    }
}

pub(super) fn lower_sha1(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(hex_format(digest_bytes_call(
        vec!["sha1", "Sha1", "digest"],
        arg_expr(args, 0),
    )))
}

pub(super) fn lower_sha512(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(hex_format(digest_bytes_call(
        vec!["sha2", "Sha512", "digest"],
        arg_expr(args, 0),
    )))
}

pub(super) fn lower_sha224(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(hex_format(digest_bytes_call(
        vec!["sha2", "Sha224", "digest"],
        arg_expr(args, 0),
    )))
}

pub(super) fn lower_sha384(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(hex_format(digest_bytes_call(
        vec!["sha2", "Sha384", "digest"],
        arg_expr(args, 0),
    )))
}

pub(super) fn lower_blake2b(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(hex_format(digest_bytes_call(
        vec!["blake2", "Blake2b512", "digest"],
        arg_expr(args, 0),
    )))
}

pub(super) fn lower_blake2s(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(hex_format(digest_bytes_call(
        vec!["blake2", "Blake2s256", "digest"],
        arg_expr(args, 0),
    )))
}
