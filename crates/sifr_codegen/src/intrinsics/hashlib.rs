//! Hashlib intrinsic lowerers for registry lowering.

use crate::RustExpr;

fn parenthesized(expr: &RustExpr) -> RustExpr {
    RustExpr::Paren(Box::new(expr.clone()))
}

fn digest_hex(digest_path: &str, arg: &RustExpr) -> RustExpr {
    RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "{:x}".to_string(),
        args: vec![RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(digest_path.to_string())),
            args: vec![RustExpr::MethodCall {
                receiver: Box::new(parenthesized(arg)),
                method: "as_bytes".to_string(),
                args: vec![],
            }],
        }],
    }
}

pub(super) fn lower_sha1(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex("<sha1::Sha1 as sha1::Digest>::digest", &args[0]))
}

pub(super) fn lower_sha512(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex(
        "<sha2::Sha512 as sha2::Digest>::digest",
        &args[0],
    ))
}

pub(super) fn lower_sha224(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex(
        "<sha2::Sha224 as sha2::Digest>::digest",
        &args[0],
    ))
}

pub(super) fn lower_sha384(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex(
        "<sha2::Sha384 as sha2::Digest>::digest",
        &args[0],
    ))
}

pub(super) fn lower_blake2b(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex(
        "<blake2::Blake2b512 as blake2::Digest>::digest",
        &args[0],
    ))
}

pub(super) fn lower_blake2s(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex(
        "<blake2::Blake2s256 as blake2::Digest>::digest",
        &args[0],
    ))
}
