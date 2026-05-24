//! Hashlib intrinsic lowerers for registry lowering.

use super::digest_format::bytes_to_hex_expr;
use crate::RustExpr;

fn parenthesized(expr: &RustExpr) -> RustExpr {
    RustExpr::Paren(Box::new(expr.clone()))
}

fn digest_hex(digest_path: &str, arg: &RustExpr) -> RustExpr {
    bytes_to_hex_expr(RustExpr::FnCall {
        func: Box::new(RustExpr::Ident(digest_path.to_string())),
        args: vec![RustExpr::MethodCall {
            receiver: Box::new(parenthesized(arg)),
            method: "as_bytes".to_string(),
            args: vec![],
        }],
    })
}

fn digest_bytes(digest_path: &str, arg: &RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(digest_path.to_string())),
            args: vec![parenthesized(arg)],
        }),
        method: "to_vec".to_string(),
        args: vec![],
    }
}

fn md5_bytes(arg: &RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Field {
            expr: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "md5".to_string(),
                    "compute".to_string(),
                ])),
                args: vec![parenthesized(arg)],
            }),
            field: "0".to_string(),
        }),
        method: "to_vec".to_string(),
        args: vec![],
    }
}

pub(crate) fn lower_sha1(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex("<sha1::Sha1 as sha1::Digest>::digest", &args[0]))
}

pub(crate) fn lower_sha1_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_bytes(
        "<sha1::Sha1 as sha1::Digest>::digest",
        &args[0],
    ))
}

pub(crate) fn lower_sha512(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex(
        "<sha2::Sha512 as sha2::Digest>::digest",
        &args[0],
    ))
}

pub(crate) fn lower_sha512_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_bytes(
        "<sha2::Sha512 as sha2::Digest>::digest",
        &args[0],
    ))
}

pub(crate) fn lower_sha224(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex(
        "<sha2::Sha224 as sha2::Digest>::digest",
        &args[0],
    ))
}

pub(crate) fn lower_sha224_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_bytes(
        "<sha2::Sha224 as sha2::Digest>::digest",
        &args[0],
    ))
}

pub(crate) fn lower_sha384(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex(
        "<sha2::Sha384 as sha2::Digest>::digest",
        &args[0],
    ))
}

pub(crate) fn lower_sha384_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_bytes(
        "<sha2::Sha384 as sha2::Digest>::digest",
        &args[0],
    ))
}

pub(crate) fn lower_blake2b(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex(
        "<blake2::Blake2b512 as blake2::Digest>::digest",
        &args[0],
    ))
}

pub(crate) fn lower_blake2b_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_bytes(
        "<blake2::Blake2b512 as blake2::Digest>::digest",
        &args[0],
    ))
}

pub(crate) fn lower_blake2s(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex(
        "<blake2::Blake2s256 as blake2::Digest>::digest",
        &args[0],
    ))
}

pub(crate) fn lower_blake2s_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_bytes(
        "<blake2::Blake2s256 as blake2::Digest>::digest",
        &args[0],
    ))
}

pub(crate) fn lower_sha256_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_bytes(
        "<sha2::Sha256 as sha2::Digest>::digest",
        &args[0],
    ))
}

pub(crate) fn lower_md5_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(md5_bytes(&args[0]))
}
