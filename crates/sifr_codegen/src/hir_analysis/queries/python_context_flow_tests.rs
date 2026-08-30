use super::*;
use sifr_ir::{HirAsyncWithKind, HirExpr, HirStmt, HirWithItem, HirWithItemKind};
use sifr_type_system::Type;

fn python_error_type() -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "PythonError".to_string(),
        fields: vec![],
        methods: vec![],
        parent_class: None,
    }
}

#[test]
fn python_async_context_suppression_keeps_following_return_reachable() {
    let error_type = python_error_type();
    let stmts = vec![
        HirStmt::AsyncWith {
            kind: HirAsyncWithKind::Python {
                context: HirExpr::Name {
                    name: "manager".to_string(),
                    binding_id: None,
                    ty: Type::Unknown,
                },
                manager_class: "Manager".to_string(),
                entered_type: Type::Unknown,
                enter_error_type: error_type.clone(),
                exit_error_type: error_type.clone(),
                entered_is_opaque_borrow: false,
                active_error_type: error_type,
                body_may_raise: true,
            },
            target: None,
            body: vec![HirStmt::Raise {
                value: HirExpr::Name {
                    name: "error".to_string(),
                    binding_id: None,
                    ty: Type::Unknown,
                },
            }],
        },
        HirStmt::Return {
            value: Some(HirExpr::BoolLiteral(false)),
        },
    ];

    assert_eq!(
        block_control_flow_effect(&stmts[..1]),
        ControlFlowEffect::FallsThrough
    );
    assert!(body_contains_return(&stmts));
    assert_eq!(
        block_control_flow_effect(&stmts),
        ControlFlowEffect::AlwaysExits
    );
}

#[test]
fn python_context_return_body_still_has_a_suppression_fallthrough() {
    let error_type = python_error_type();
    let statements = vec![python_context_with(error_type, true)];

    assert_eq!(
        block_control_flow_effect(&statements),
        ControlFlowEffect::FallsThrough
    );
    assert!(body_contains_return(&statements));
}

#[test]
fn infallible_python_context_return_body_remains_terminal() {
    let statements = vec![python_context_with(Type::Unknown, false)];

    assert_eq!(
        block_control_flow_effect(&statements),
        ControlFlowEffect::AlwaysReturns
    );
}

fn python_context_with(error_type: Type, body_may_raise: bool) -> HirStmt {
    HirStmt::With {
        items: vec![HirWithItem {
            target: "value".to_string(),
            context: HirExpr::Name {
                name: "manager".to_string(),
                binding_id: None,
                ty: Type::Unknown,
            },
            kind: HirWithItemKind::Python {
                entered_type: Type::Unknown,
                enter_error_type: error_type.clone(),
                exit_error_type: error_type,
                entered_is_opaque_borrow: false,
                body_may_raise,
            },
        }],
        body: vec![HirStmt::Return {
            value: Some(HirExpr::IntLiteral(1)),
        }],
    }
}
