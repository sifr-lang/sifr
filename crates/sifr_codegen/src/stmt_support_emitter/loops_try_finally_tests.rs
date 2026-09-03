use crate::{RustEmitter, RustExpr, RustLiteral, RustStmt, RustType};

fn representative_loop_forms() -> [RustStmt; 3] {
    [
        RustStmt::While {
            cond: RustExpr::Literal(RustLiteral::Bool(true)),
            body: vec![RustStmt::Break],
        },
        RustStmt::For {
            var: "item".to_string(),
            iter: RustExpr::Ident("items".to_string()),
            body: vec![RustStmt::Continue],
        },
        RustStmt::Loop {
            body: vec![RustStmt::Expr(RustExpr::Ident(
                "poll_async_iterator".to_string(),
            ))],
        },
    ]
}

#[test]
fn sync_async_and_statement_block_loops_share_one_else_scaffold() {
    let else_body = vec![RustStmt::Expr(RustExpr::Ident("on_complete".to_string()))];

    for lowered_loop in representative_loop_forms() {
        let scaffold =
            RustEmitter::loop_else_scaffold_for_ir(lowered_loop.clone(), else_body.clone());
        let RustStmt::Block(statements) = scaffold else {
            panic!("loop/else scaffold must be a structured block");
        };

        assert_eq!(statements.len(), 3);
        assert_eq!(
            statements[0],
            RustStmt::Let {
                mutable: true,
                name: "_broke".to_string(),
                ty: Some(RustType::Bool),
                value: RustExpr::Literal(RustLiteral::Bool(false)),
            }
        );
        assert_eq!(statements[1], lowered_loop);
        assert_eq!(
            statements[2],
            RustStmt::If {
                cond: RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(RustExpr::Paren(Box::new(RustExpr::Ident(
                        "_broke".to_string(),
                    )))),
                },
                then_body: else_body.clone(),
                else_body: Some(Vec::new()),
            }
        );
    }
}
