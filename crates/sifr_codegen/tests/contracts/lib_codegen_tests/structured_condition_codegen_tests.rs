use super::*;

#[test]
fn test_structured_stmt_path_handles_chained_compare_condition_inside_loop_if() {
    let stmt = HirStmt::While {
        condition: HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "left".to_string(),
                binding_id: None,
                ty: Type::Int,
            }),
            ops: vec!["<=".to_string()],
            comparators: vec![HirExpr::Name {
                name: "right".to_string(),
                binding_id: None,
                ty: Type::Int,
            }],
            ty: Type::Bool,
        },
        body: vec![HirStmt::If {
            condition: HirExpr::Compare {
                left: Box::new(HirExpr::Name {
                    name: "left".to_string(),
                    binding_id: None,
                    ty: Type::Int,
                }),
                ops: vec!["<=".to_string(), "<".to_string()],
                comparators: vec![
                    HirExpr::Name {
                        name: "target".to_string(),
                        binding_id: None,
                        ty: Type::Int,
                    },
                    HirExpr::Name {
                        name: "right".to_string(),
                        binding_id: None,
                        ty: Type::Int,
                    },
                ],
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Assign {
                name: "left".to_string(),
                value: HirExpr::IntLiteral(1),
            }],
            elif_clauses: vec![],
            else_body: Some(vec![HirStmt::AugAssign {
                name: "left".to_string(),
                op: "+=".to_string(),
                value: HirExpr::IntLiteral(1),
            }]),
        }],
        else_body: None,
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert!(matches!(captured.first(), Some(RustStmt::While { .. })));
}
