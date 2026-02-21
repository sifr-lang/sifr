//! Platform intrinsic lowerers for registry migration.

use crate::RustExpr;

pub(super) fn lower_platform_system(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode("std::env::consts::OS.to_string()".to_string()))
}

pub(super) fn lower_platform_arch(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode("std::env::consts::ARCH.to_string()".to_string()))
}

pub(super) fn lower_platform_node(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "std::process::Command::new(\"hostname\").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()".to_string(),
    ))
}

pub(super) fn lower_platform_release(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "{ std::process::Command::new(\"uname\").arg(\"-r\").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default() }".to_string(),
    ))
}

pub(super) fn lower_platform_version(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "{ std::process::Command::new(\"uname\").arg(\"-v\").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default() }".to_string(),
    ))
}

pub(super) fn lower_platform_processor(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode("std::env::consts::ARCH.to_string()".to_string()))
}
