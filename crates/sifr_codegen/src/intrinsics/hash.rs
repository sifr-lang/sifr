//! Hash intrinsic lowerers for registry migration.

use crate::RustExpr;

pub(super) fn lower_sha256(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ use sha2::Digest; format!(\"{{:x}}\", sha2::Sha256::digest(({}).as_bytes())) }}",
        args[0],
    )))
}

pub(super) fn lower_md5(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "format!(\"{{:x}}\", md5::compute(({}).as_bytes()))",
        args[0],
    )))
}
