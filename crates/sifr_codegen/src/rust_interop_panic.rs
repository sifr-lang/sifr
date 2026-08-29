use sifr_ir::{RustInteropDeclaration, RustInteropValue, RustTargetPath};
use sifr_type_system::Type;

use crate::helpers::wrap_union_member_expr;
use crate::rust_interop_error_mapping::bridge_error_contract_expr;
use crate::{RustExpr, RustMatchArm, RustParam, RustStmt};

const BRIDGE_RESULT: &str = "__sifr_bridge_result";
const ORIGINAL_PANIC: &str = "__sifr_original_panic";
const MAPPED_ERROR: &str = "__sifr_mapped_error";

pub(crate) fn recoverable_sync_panic_result_expr(
    call: RustExpr,
    return_type: &Type,
    declaration: &RustInteropDeclaration,
    successful_call: RustExpr,
) -> Option<RustExpr> {
    let Type::Result(_, error_type) = return_type.resolve_alias() else {
        return None;
    };
    let panic_type = rust_panic_error_member(error_type)?;
    let caught_call = catch_panic_expr(call);
    let panic_result = map_error_target(declaration).map_or_else(
        || rust_panic_result(error_type, panic_type),
        |mapper| mapped_panic_result(mapper, error_type, panic_type),
    );
    Some(RustExpr::Match {
        expr: Box::new(caught_call),
        arms: vec![
            match_arm(
                &format!("Ok({BRIDGE_RESULT})"),
                RustExpr::Ident(BRIDGE_RESULT.to_string()),
                successful_call,
            ),
            RustMatchArm {
                pattern: format!("Err({ORIGINAL_PANIC})").into(),
                bindings: vec![ORIGINAL_PANIC.to_string()],
                guard: None,
                body: vec![RustStmt::TailExpr(panic_result)],
            },
        ],
    })
}

pub(crate) fn bridge_declared_error_expr(value: RustExpr, error_type: &Type) -> RustExpr {
    let Some(member) = ordinary_error_member(error_type) else {
        return bridge_error_contract_expr(value, error_type);
    };
    let mapped = bridge_error_contract_expr(value.clone(), member);
    wrap_union_member_expr(error_type, member, mapped)
        .unwrap_or_else(|| bridge_error_contract_expr(value, error_type))
}

pub(crate) fn rust_panic_error_member(error_type: &Type) -> Option<&Type> {
    match error_type.resolve_alias() {
        Type::Union(members) => members.iter().find(|member| is_rust_panic_error(member)),
        member if is_rust_panic_error(member) => Some(member),
        _ => None,
    }
}

pub(crate) fn stored_rust_panic_error_value(
    panic: RustExpr,
    error_type: &Type,
) -> Option<RustExpr> {
    let panic_type = rust_panic_error_member(error_type)?;
    let message = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(panic),
            method: "message".to_string(),
            args: Vec::new(),
        }),
        method: "to_string".to_string(),
        args: Vec::new(),
    };
    let panic_error = RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            crate::render_type(&crate::sifr_type_to_rust_type(panic_type)),
            "new".to_string(),
        ])),
        args: vec![message],
    };
    Some(
        wrap_union_member_expr(error_type, panic_type, panic_error).unwrap_or_else(|| {
            RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    crate::render_type(&crate::sifr_type_to_rust_type(panic_type)),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::compiler_fragment(
                    "\"Rust bridge panicked\".to_string()".to_string(),
                )],
            }
        }),
    )
}

pub(crate) fn ordinary_error_member(error_type: &Type) -> Option<&Type> {
    let Type::Union(members) = error_type.resolve_alias() else {
        return None;
    };
    let ordinary = members
        .iter()
        .filter(|member| !is_rust_panic_error(member))
        .collect::<Vec<_>>();
    matches!(ordinary.as_slice(), [_member] if members.len() == 2).then(|| ordinary[0])
}

pub(crate) fn map_error_target(declaration: &RustInteropDeclaration) -> Option<&RustTargetPath> {
    declaration
        .arguments
        .iter()
        .find(|argument| argument.name.as_deref() == Some("panic"))
        .and_then(|argument| match &argument.value {
            RustInteropValue::PolicyCall { name, argument, .. } if name == "map_error" => {
                match argument.as_ref() {
                    RustInteropValue::TargetPath(target) => Some(target),
                    _ => None,
                }
            }
            _ => None,
        })
}

fn mapped_panic_result(mapper: &RustTargetPath, error_type: &Type, panic_type: &Type) -> RustExpr {
    let mapper_call = RustExpr::FnCall {
        func: Box::new(RustExpr::Path(mapper.segments.clone())),
        args: vec![RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(ORIGINAL_PANIC.to_string())),
            method: "clone".to_string(),
            args: Vec::new(),
        }],
    };
    let mapped = ordinary_error_member(error_type).map_or_else(
        || rust_panic_result(error_type, panic_type),
        |ordinary| {
            let converted =
                bridge_error_contract_expr(RustExpr::Ident(MAPPED_ERROR.to_string()), ordinary);
            result_error(
                wrap_union_member_expr(error_type, ordinary, converted)
                    .unwrap_or_else(|| RustExpr::Ident(MAPPED_ERROR.to_string())),
            )
        },
    );
    RustExpr::Match {
        expr: Box::new(catch_panic_expr(mapper_call)),
        arms: vec![
            match_arm(
                &format!("Ok({MAPPED_ERROR})"),
                RustExpr::Ident(MAPPED_ERROR.to_string()),
                mapped,
            ),
            RustMatchArm {
                pattern: "Err(_)".into(),
                bindings: Vec::new(),
                guard: None,
                body: vec![RustStmt::TailExpr(rust_panic_result(
                    error_type, panic_type,
                ))],
            },
        ],
    }
}

fn rust_panic_result(error_type: &Type, panic_type: &Type) -> RustExpr {
    let message = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(ORIGINAL_PANIC.to_string())),
            method: "message".to_string(),
            args: Vec::new(),
        }),
        method: "to_string".to_string(),
        args: Vec::new(),
    };
    let panic_error = RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            crate::render_type(&crate::sifr_type_to_rust_type(panic_type)),
            "new".to_string(),
        ])),
        args: vec![message],
    };
    result_error(
        wrap_union_member_expr(error_type, panic_type, panic_error).unwrap_or_else(|| {
            RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    crate::render_type(&crate::sifr_type_to_rust_type(panic_type)),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::compiler_fragment(
                    "\"Rust bridge panicked\".to_string()".to_string(),
                )],
            }
        }),
    )
}

fn catch_panic_expr(body: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "sifr_runtime".to_string(),
            "interop".to_string(),
            "catch_rust_panic".to_string(),
        ])),
        args: vec![RustExpr::Closure {
            params: Vec::<RustParam>::new(),
            body: Box::new(body),
            is_move: false,
        }],
    }
}

fn result_error(error: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
        args: vec![error],
    }
}

fn match_arm(pattern: &str, binding: RustExpr, body: RustExpr) -> RustMatchArm {
    RustMatchArm {
        pattern: pattern.into(),
        bindings: match binding {
            RustExpr::Ident(name) => vec![name],
            _ => Vec::new(),
        },
        guard: None,
        body: vec![RustStmt::TailExpr(body)],
    }
}

fn is_rust_panic_error(ty: &Type) -> bool {
    matches!(
        ty.resolve_alias(),
        Type::Class { name, .. } if name == "RustPanicError"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper_target_requires_the_named_map_error_policy_shape() {
        let target = RustTargetPath {
            segments: vec![
                "bridge".to_string(),
                "wrapper".to_string(),
                "map".to_string(),
            ],
            span: ruff_text_size::TextRange::default(),
        };
        let declaration = RustInteropDeclaration {
            kind: sifr_ir::RustInteropDecoratorKind::Function,
            target: None,
            arguments: vec![sifr_ir::RustInteropArgument {
                name: Some("panic".to_string()),
                value: RustInteropValue::PolicyCall {
                    name: "map_error".to_string(),
                    argument: Box::new(RustInteropValue::TargetPath(target.clone())),
                    span: ruff_text_size::TextRange::default(),
                },
                span: ruff_text_size::TextRange::default(),
            }],
            span: ruff_text_size::TextRange::default(),
            effect: sifr_ir::RustInteropEffect::Sync,
            abi_requirements: sifr_ir::RustInteropAbiRequirements::default(),
            consumes_receiver: false,
        };

        assert_eq!(map_error_target(&declaration), Some(&target));
    }
}
