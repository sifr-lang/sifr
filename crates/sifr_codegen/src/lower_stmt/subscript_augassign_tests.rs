use super::*;
#[test]
fn lowers_simple_list_subscript_augassign_floor_div_equal_stmt() {
    let stmt = HirStmt::SubscriptAugAssign {
        object: "items".to_string(),
        index: HirExpr::IntLiteral(0),
        op: "//=".to_string(),
        value: HirExpr::Name {
            name: "d".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        object_ty: Type::List(Box::new(Type::Int)),
        failure: None,
    };
    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
    let mut emitter = crate::RustEmitter::new();
    let lowered = emitter
        .lower_subscript_augassign_stmt_for_ir(
            "items",
            match &stmt {
                HirStmt::SubscriptAugAssign { index, .. } => index,
                _ => unreachable!(),
            },
            "//=",
            match &stmt {
                HirStmt::SubscriptAugAssign { value, .. } => value,
                _ => unreachable!(),
            },
            &Type::List(Box::new(Type::Int)),
            None,
        )
        .expect("typed floor-div lowering should succeed")
        .expect("typed floor-div augassign should lower");
    let rendered = crate::render_stmts(std::slice::from_ref(&lowered));
    assert!(
        rendered.contains("*__elem = __elem.floor_div_known_nonzero(&__assign_value)"),
        "{rendered}"
    );
}

#[test]
fn lowers_simple_list_subscript_augassign_power_equal_stmt() {
    let stmt = HirStmt::SubscriptAugAssign {
        object: "items".to_string(),
        index: HirExpr::IntLiteral(0),
        op: "**=".to_string(),
        value: HirExpr::IntLiteral(2),
        object_ty: Type::List(Box::new(Type::Int)),
        failure: None,
    };
    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
    let mut emitter = crate::RustEmitter::new();
    let lowered = emitter
        .lower_subscript_augassign_stmt_for_ir(
            "items",
            match &stmt {
                HirStmt::SubscriptAugAssign { index, .. } => index,
                _ => unreachable!(),
            },
            "**=",
            match &stmt {
                HirStmt::SubscriptAugAssign { value, .. } => value,
                _ => unreachable!(),
            },
            &Type::List(Box::new(Type::Int)),
            None,
        )
        .expect("typed power lowering should succeed")
        .expect("typed power augassign should lower");
    let rendered = crate::render_stmts(std::slice::from_ref(&lowered));
    assert!(
        rendered.contains("*__elem = __elem.pow_known_valid(2_u32)"),
        "{rendered}"
    );
}

#[test]
fn lowers_simple_alias_list_subscript_augassign_stmt() {
    let stmt = HirStmt::SubscriptAugAssign {
        object: "items".to_string(),
        index: HirExpr::IntLiteral(0),
        op: "+=".to_string(),
        value: HirExpr::IntLiteral(1),
        object_ty: Type::alias("IntList", Type::List(Box::new(Type::Int))),
        failure: None,
    };
    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}

#[test]
fn does_not_lower_subscript_augassign_with_non_leaf_value() {
    let stmt = HirStmt::SubscriptAugAssign {
        object: "items".to_string(),
        index: HirExpr::IntLiteral(0),
        op: "+=".to_string(),
        value: HirExpr::Call {
            mutable_arg_places: Vec::new(),
            func: "next_value".to_string(),
            args: vec![],
            ty: Type::Int,
        },
        object_ty: Type::List(Box::new(Type::Int)),
        failure: None,
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}
