//! Hashlib intrinsic lowerers for registry migration.

use crate::RustExpr;

fn borrowed_str(expr: &str) -> String {
    format!("&({expr})")
}

pub(super) fn lower_sha1(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "{{ use sha1::Digest; format!(\"{{:x}}\", sha1::Sha1::digest({}.as_bytes())) }}",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_sha512(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "{{ use sha2::Digest; format!(\"{{:x}}\", sha2::Sha512::digest({}.as_bytes())) }}",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_sha224(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "{{ use sha2::Digest; let mut __h = sha2::Sha224::new(); __h.update({}.as_bytes()); format!(\"{{:x}}\", __h.finalize()) }}",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_sha384(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "{{ use sha2::Digest; let mut __h = sha2::Sha384::new(); __h.update({}.as_bytes()); format!(\"{{:x}}\", __h.finalize()) }}",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_blake2b(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "{{ use blake2::{{Blake2b512, Digest}}; let mut __h = Blake2b512::new(); __h.update({}.as_bytes()); format!(\"{{:x}}\", __h.finalize()) }}",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_blake2s(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "{{ use blake2::{{Blake2s256, Digest}}; let mut __h = Blake2s256::new(); __h.update({}.as_bytes()); format!(\"{{:x}}\", __h.finalize()) }}",
        borrowed_str(&args[0])
    )))
}
