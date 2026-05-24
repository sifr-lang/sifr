use super::*;

#[test]
fn lowers_simple_yield_inside_generator_closure() {
    let stmt = HirStmt::Yield {
        value: HirExpr::IntLiteral(7),
    };
    let lowered = try_lower_simple_stmt_with_ctx(
        &stmt,
        false,
        &HashSet::new(),
        &HashSet::new(),
        SimpleStmtLoweringCtx {
            return_type: None,
            in_display_impl: false,
            in_class_scope: false,
            in_generator_closure: true,
        },
    )
    .expect("yield lowered");

    assert!(matches!(
        lowered[0],
        RustStmt::Return(Some(RustExpr::FnCall { .. }))
    ));
}

#[test]
fn lowers_simple_yield_outside_generator_closure() {
    let stmt = HirStmt::Yield {
        value: HirExpr::IntLiteral(7),
    };
    let lowered = try_lower_simple_stmt_with_ctx(
        &stmt,
        false,
        &HashSet::new(),
        &HashSet::new(),
        SimpleStmtLoweringCtx::default(),
    )
    .expect("yield lowered");

    assert!(matches!(
        lowered[0],
        RustStmt::Expr(RustExpr::MethodCall { .. })
    ));
}

#[test]
fn lowers_simple_star_unpack_from_name() {
    let stmt = HirStmt::StarUnpack {
        before: vec![("head".to_string(), Type::Int)],
        star: ("mid".to_string(), Type::List(Box::new(Type::Int))),
        after: vec![("tail".to_string(), Type::Int)],
        value: HirExpr::Name {
            name: "xs".to_string(),
            ty: Type::List(Box::new(Type::Int)),
        },
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("star unpack lowered");
    assert_eq!(lowered.len(), 4);
    assert!(matches!(
        lowered[0],
        RustStmt::Let { ref name, .. } if name == "_star_tmp"
    ));
}

#[test]
fn lowers_simple_with_without_context_manager_protocol() {
    let stmt = HirStmt::With {
        items: vec![("x".to_string(), HirExpr::IntLiteral(1), false)],
        body: vec![HirStmt::Expr {
            expr: HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            },
        }],
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("with lowered");
    assert!(matches!(lowered[0], RustStmt::Block(_)));
}
