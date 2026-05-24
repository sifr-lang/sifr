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

pub(crate) fn lower_platform_system(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::If {
        cond: Box::new(RustExpr::MacroCall {
            name: "cfg".to_string(),
            args: vec![RustExpr::Ident("target_os = \"windows\"".to_string())],
        }),
        then_expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Literal(crate::RustLiteral::Str(
                "Windows".to_string(),
            ))),
            method: "to_string".to_string(),
            args: vec![],
        }),
        else_expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::MacroCall {
                name: "cfg".to_string(),
                args: vec![RustExpr::Ident("target_os = \"macos\"".to_string())],
            }),
            then_expr: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Literal(crate::RustLiteral::Str(
                    "Darwin".to_string(),
                ))),
                method: "to_string".to_string(),
                args: vec![],
            }),
            else_expr: Some(Box::new(RustExpr::If {
                cond: Box::new(RustExpr::MacroCall {
                    name: "cfg".to_string(),
                    args: vec![RustExpr::Ident("target_os = \"linux\"".to_string())],
                }),
                then_expr: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Literal(crate::RustLiteral::Str(
                        "Linux".to_string(),
                    ))),
                    method: "to_string".to_string(),
                    args: vec![],
                }),
                else_expr: Some(Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "env".to_string(),
                        "consts".to_string(),
                        "OS".to_string(),
                    ])),
                    method: "to_string".to_string(),
                    args: vec![],
                })),
            })),
        })),
    })
}

pub(crate) fn lower_platform_arch(args: &[RustExpr]) -> Option<RustExpr> {
    lower_const_to_string(args, "ARCH")
}

pub(crate) fn lower_platform_node(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Ident(
        "{ std::env::var(\"HOSTNAME\").or_else(|_| std::env::var(\"COMPUTERNAME\")).unwrap_or_else(|_| \"localhost\".to_string()) }".to_string(),
    ))
}

pub(crate) fn lower_platform_release(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Ident(
        "{ std::process::Command::new(\"uname\").arg(\"-r\").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).ok().filter(|s| !s.is_empty()).unwrap_or_else(|| std::env::consts::OS.to_string()) }".to_string(),
    ))
}

pub(crate) fn lower_platform_version(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Ident(
        "{ std::process::Command::new(\"uname\").arg(\"-v\").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).ok().filter(|s| !s.is_empty()).unwrap_or_else(|| std::env::consts::OS.to_string()) }".to_string(),
    ))
}

pub(crate) fn lower_platform_processor(args: &[RustExpr]) -> Option<RustExpr> {
    lower_const_to_string(args, "ARCH")
}
