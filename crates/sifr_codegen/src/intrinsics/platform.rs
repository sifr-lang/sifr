//! Platform intrinsic lowerers for registry lowering.

use crate::RustExpr;

fn lower_const_to_string(args: &[RustExpr], constant: &str) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "env".to_string(),
            "consts".to_string(),
            constant.to_string(),
        ])),
        method: "to_string".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_platform_system(args: &[RustExpr]) -> Option<RustExpr> {
    lower_const_to_string(args, "OS")
}

pub(super) fn lower_platform_arch(args: &[RustExpr]) -> Option<RustExpr> {
    lower_const_to_string(args, "ARCH")
}

pub(super) fn lower_platform_node(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Ident(
        "std::process::Command::new(\"hostname\").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()".to_string(),
    ))
}

pub(super) fn lower_platform_release(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Ident(
        "{ std::process::Command::new(\"uname\").arg(\"-r\").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default() }".to_string(),
    ))
}

pub(super) fn lower_platform_version(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Ident(
        "{ std::process::Command::new(\"uname\").arg(\"-v\").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default() }".to_string(),
    ))
}

pub(super) fn lower_platform_processor(args: &[RustExpr]) -> Option<RustExpr> {
    lower_const_to_string(args, "ARCH")
}
