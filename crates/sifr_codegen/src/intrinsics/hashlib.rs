//! Hashlib intrinsic lowerers for registry migration.

use crate::RustExpr;

fn digest_hex(digest_path: &str, arg: &str) -> RustExpr {
    RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "{:x}".to_string(),
        args: vec![RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(digest_path.to_string())),
            args: vec![RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(arg.to_string())),
                method: "as_bytes".to_string(),
                args: vec![],
            }],
        }],
    }
}

pub(super) fn lower_sha1(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex(
        "<sha1::Sha1 as sha1::Digest>::digest",
        args[0].as_str(),
    ))
}

pub(super) fn lower_sha512(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex(
        "<sha2::Sha512 as sha2::Digest>::digest",
        args[0].as_str(),
    ))
}

pub(super) fn lower_sha224(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex(
        "<sha2::Sha224 as sha2::Digest>::digest",
        args[0].as_str(),
    ))
}

pub(super) fn lower_sha384(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex(
        "<sha2::Sha384 as sha2::Digest>::digest",
        args[0].as_str(),
    ))
}

pub(super) fn lower_blake2b(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex(
        "<blake2::Blake2b512 as blake2::Digest>::digest",
        args[0].as_str(),
    ))
}

pub(super) fn lower_blake2s(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(digest_hex(
        "<blake2::Blake2s256 as blake2::Digest>::digest",
        args[0].as_str(),
    ))
}
