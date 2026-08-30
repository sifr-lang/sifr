use super::*;

#[test]
fn test_structured_stmt_path_handles_delete_with_name_key_inside_loop_if() {
    let stmt = HirStmt::For {
        target: "ch".to_string(),
        target_ty: Type::Str,
        iter: HirExpr::Name {
            name: "order".to_string(),
            binding_id: None,
            ty: Type::Str,
        },
        body: vec![HirStmt::If {
            condition: HirExpr::ContainsOp {
                element: Box::new(HirExpr::Name {
                    name: "ch".to_string(),
                    binding_id: None,
                    ty: Type::Str,
                }),
                collection: Box::new(HirExpr::Name {
                    name: "counts".to_string(),
                    binding_id: None,
                    ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                }),
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Delete {
                object: HirExpr::Name {
                    name: "counts".to_string(),
                    binding_id: None,
                    ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                },
                index: HirExpr::Name {
                    name: "ch".to_string(),
                    binding_id: None,
                    ty: Type::Str,
                },
                failure: None,
            }],
            elif_clauses: vec![],
            else_body: None,
        }],
        else_body: None,
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert!(matches!(captured.first(), Some(RustStmt::For { .. })));
}
