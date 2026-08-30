use super::*;

fn key_error_type() -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "KeyError".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    }
}

#[test]
fn checked_dict_subscript_augassign_returns_key_error_when_missing() {
    let error_ty = key_error_type();
    let stmt = HirStmt::SubscriptAugAssign {
        object: "mapping".to_string(),
        index: HirExpr::StringLiteral("missing".to_string()),
        op: "+=".to_string(),
        value: HirExpr::IntLiteral(1),
        object_ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        failure: Some(error_ty.clone()),
    };
    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());

    let mut emitter = crate::RustEmitter::new();
    let lowered = emitter
        .lower_checked_place_mutation_stmt_for_ir(&stmt)
        .expect("checked dict augassign lowering should succeed")
        .expect("checked dict augassign should lower");
    let rendered = crate::render_stmts(&lowered);
    assert!(
        rendered.contains("mapping.get_mut(&__assign_key)"),
        "{rendered}"
    );
    assert!(rendered.contains("return Err(KeyError::new"), "{rendered}");
}

#[test]
fn annotated_defaultdict_alias_keeps_entry_insertion_codegen() {
    let stmt = HirStmt::SubscriptAugAssign {
        object: "mapping".to_string(),
        index: HirExpr::StringLiteral("missing".to_string()),
        op: "+=".to_string(),
        value: HirExpr::IntLiteral(3),
        object_ty: Type::alias(
            "__sifr_defaultdict_int",
            Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        ),
        failure: None,
    };
    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());

    let mut emitter = crate::RustEmitter::new();
    let lowered = emitter
        .lower_checked_place_mutation_stmt_for_ir(&stmt)
        .expect("typed defaultdict augassign lowering should succeed")
        .expect("typed defaultdict augassign should lower");
    assert!(matches!(
        lowered.as_slice(),
        [RustStmt::Block(stmts)]
            if matches!(
                stmts.first(),
                Some(RustStmt::Let {
                    value: RustExpr::MethodCall { method, .. },
                    ..
                }) if method == "or_insert"
            )
    ));
}
