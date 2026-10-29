use super::{
    declaration_only_binding_needs_mutability,
    stmts_require_var_value_at_entry_including_nested_functions,
};
use sifr_ir::{HirExpr, HirFunction, HirParam, HirStmt, MethodKind};
use sifr_type_system::{ParamConvention, Type};

fn name(value: &str) -> HirExpr {
    HirExpr::Name {
        name: value.to_string(),
        binding_id: None,
        ty: Type::Str,
    }
}

#[test]
fn unconditional_replacement_kills_the_incoming_value() {
    let stmts = vec![
        HirStmt::Assign {
            name: "value".to_string(),
            value: HirExpr::StringLiteral("new".to_string()),
        },
        HirStmt::Return {
            value: Some(name("value")),
        },
    ];

    assert!(!stmts_require_var_value_at_entry_including_nested_functions(&stmts, "value"));
}

#[test]
fn replacement_rhs_can_keep_the_incoming_value_live() {
    let stmts = vec![HirStmt::Assign {
        name: "value".to_string(),
        value: name("value"),
    }];

    assert!(stmts_require_var_value_at_entry_including_nested_functions(
        &stmts, "value"
    ));
}

#[test]
fn every_continuing_branch_must_replace_the_incoming_value() {
    let complete = vec![
        HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::Assign {
                name: "value".to_string(),
                value: HirExpr::StringLiteral("then".to_string()),
            }],
            elif_clauses: vec![],
            else_body: Some(vec![HirStmt::Assign {
                name: "value".to_string(),
                value: HirExpr::StringLiteral("else".to_string()),
            }]),
        },
        HirStmt::Return {
            value: Some(name("value")),
        },
    ];
    let partial = vec![
        HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::Assign {
                name: "value".to_string(),
                value: HirExpr::StringLiteral("then".to_string()),
            }],
            elif_clauses: vec![],
            else_body: None,
        },
        HirStmt::Return {
            value: Some(name("value")),
        },
    ];

    assert!(!stmts_require_var_value_at_entry_including_nested_functions(&complete, "value"));
    assert!(stmts_require_var_value_at_entry_including_nested_functions(
        &partial, "value"
    ));
}

#[test]
fn one_unconditional_replacement_does_not_need_mutability() {
    let stmts = vec![HirStmt::Assign {
        name: "value".to_string(),
        value: HirExpr::IntLiteral(1),
    }];

    assert!(!declaration_only_binding_needs_mutability(&stmts, "value"));
}

#[test]
fn repeated_replacement_on_one_path_needs_mutability() {
    let stmts = vec![
        HirStmt::Assign {
            name: "value".to_string(),
            value: HirExpr::IntLiteral(1),
        },
        HirStmt::Assign {
            name: "value".to_string(),
            value: HirExpr::IntLiteral(2),
        },
    ];

    assert!(declaration_only_binding_needs_mutability(&stmts, "value"));
}

#[test]
fn alternative_branch_replacements_do_not_need_mutability() {
    let stmts = vec![HirStmt::If {
        condition: HirExpr::BoolLiteral(true),
        then_body: vec![HirStmt::Assign {
            name: "value".to_string(),
            value: HirExpr::IntLiteral(1),
        }],
        elif_clauses: vec![],
        else_body: Some(vec![HirStmt::Assign {
            name: "value".to_string(),
            value: HirExpr::IntLiteral(2),
        }]),
    }];

    assert!(!declaration_only_binding_needs_mutability(&stmts, "value"));
}

#[test]
fn nested_function_default_reads_the_value_in_the_defining_scope() {
    let stmts = vec![HirStmt::NestedFunction {
        func: HirFunction {
            name: "read_value".to_string(),
            params: vec![HirParam {
                name: "candidate".to_string(),
                ty: Type::Str,
                default: Some(name("value")),
                keyword_only: false,
                convention: ParamConvention::own(),
            }],
            return_type: Type::Str,
            body: vec![HirStmt::Return {
                value: Some(name("candidate")),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        },
        move_captures: false,
        capture_clones: Vec::new(),
    }];

    assert!(stmts_require_var_value_at_entry_including_nested_functions(
        &stmts, "value"
    ));
}
