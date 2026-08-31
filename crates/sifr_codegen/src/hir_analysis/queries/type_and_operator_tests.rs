use super::*;
use sifr_ir::{HirExpr, HirStmt};
use sifr_type_system::Type;

#[test]
fn collect_typevar_operator_requirements_detects_add_and_sub() {
    let stmts = vec![
        HirStmt::Expr {
            expr: HirExpr::BinOp {
                left: Box::new(HirExpr::Name {
                    name: "a".to_string(),
                    binding_id: None,
                    ty: Type::TypeVar("T".to_string()),
                }),
                op: "+".to_string(),
                right: Box::new(HirExpr::Name {
                    name: "b".to_string(),
                    binding_id: None,
                    ty: Type::TypeVar("T".to_string()),
                }),
                ty: Type::TypeVar("T".to_string()),
            },
        },
        HirStmt::Expr {
            expr: HirExpr::BinOp {
                left: Box::new(HirExpr::Name {
                    name: "a".to_string(),
                    binding_id: None,
                    ty: Type::TypeVar("T".to_string()),
                }),
                op: "-".to_string(),
                right: Box::new(HirExpr::Name {
                    name: "b".to_string(),
                    binding_id: None,
                    ty: Type::TypeVar("T".to_string()),
                }),
                ty: Type::TypeVar("T".to_string()),
            },
        },
    ];

    let req = collect_typevar_operator_requirements(&stmts, "T");
    assert!(req.needs_add);
    assert!(req.needs_sub);
}

#[test]
fn collect_typevar_operator_requirements_detects_equality() {
    let stmts = vec![HirStmt::Expr {
        expr: HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "a".to_string(),
                binding_id: None,
                ty: Type::TypeVar("T".to_string()),
            }),
            ops: vec!["==".to_string()],
            comparators: vec![HirExpr::Name {
                name: "b".to_string(),
                binding_id: None,
                ty: Type::TypeVar("T".to_string()),
            }],
            ty: Type::Bool,
        },
    }];

    let req = collect_typevar_operator_requirements(&stmts, "T");
    assert!(req.needs_partial_eq);
}

#[test]
fn collect_typevar_operator_requirements_detects_display_calls() {
    let stmts = vec![HirStmt::Expr {
        expr: HirExpr::Call {
            mutable_arg_places: Vec::new(),
            func: "str".to_string(),
            args: vec![HirExpr::Name {
                name: "value".to_string(),
                binding_id: None,
                ty: Type::TypeVar("T".to_string()),
            }],
            ty: Type::Str,
        },
    }];

    let req = collect_typevar_operator_requirements(&stmts, "T");
    assert!(req.needs_display);
}

#[test]
fn collect_let_declared_types_covers_nested_blocks() {
    let stmts = vec![HirStmt::If {
        condition: HirExpr::BoolLiteral(true),
        then_body: vec![HirStmt::Let {
            name: "x".to_string(),
            ty: Type::Union(vec![Type::Int, Type::Str]),
            value: HirExpr::IntLiteral(1),
            is_mutable: true,
        }],
        elif_clauses: vec![],
        else_body: None,
    }];

    let declared = collect_let_declared_types(&stmts);
    assert_eq!(declared.len(), 1);
    assert!(matches!(declared[0], Type::Union(_)));
}
