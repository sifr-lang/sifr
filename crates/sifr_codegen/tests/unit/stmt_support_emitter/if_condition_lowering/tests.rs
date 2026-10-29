use super::*;

fn isinstance_union_condition(target: &str) -> HirExpr {
    HirExpr::Call {
        func: "isinstance".to_string(),
        args: vec![
            HirExpr::Name {
                name: "value".to_string(),
                binding_id: None,
                ty: Type::Union(vec![Type::Int, Type::Str]),
            },
            HirExpr::StringLiteral(target.to_string()),
        ],
        mutable_arg_places: vec![None, None],
        ty: Type::Bool,
    }
}

#[test]
fn declined_speculative_branch_restores_string_cache_state() {
    let mut emitter = RustEmitter::new();
    emitter
        .string_char_cache_required_names
        .insert("text".to_string());
    emitter
        .string_char_cache_vars
        .insert("existing".to_string(), "__sifr_existing_chars".to_string());
    let before = emitter.string_char_cache_vars.clone();
    let body = vec![
        HirStmt::Let {
            name: "text".to_string(),
            ty: Type::Str,
            value: HirExpr::StringLiteral("value".to_string()),
            is_mutable: false,
        },
        HirStmt::AttributeAugAssign {
            object: "self".to_string(),
            field: "label".to_string(),
            op: "+=".to_string(),
            value: HirExpr::StringLiteral("suffix".to_string()),
        },
    ];

    let lowered = emitter
        .try_lower_if_branch_for_ir(&body)
        .expect("speculative lowering must not error");

    assert!(lowered.is_none());
    assert_eq!(emitter.string_char_cache_vars, before);
}

#[test]
fn successful_speculative_branch_restores_string_cache_state() {
    let mut emitter = RustEmitter::new();
    emitter
        .string_char_cache_required_names
        .insert("text".to_string());
    let before = emitter.string_char_cache_vars.clone();
    let body = vec![HirStmt::Let {
        name: "text".to_string(),
        ty: Type::Str,
        value: HirExpr::StringLiteral("value".to_string()),
        is_mutable: false,
    }];

    let lowered = emitter
        .try_lower_if_branch_for_ir(&body)
        .expect("speculative lowering must not error");

    assert!(lowered.is_some());
    assert_eq!(emitter.string_char_cache_vars, before);
}

#[test]
fn declined_isinstance_union_restores_successful_sibling_cache_state() {
    let mut emitter = RustEmitter::new();
    emitter.register_union_type(&Type::Union(vec![Type::Int, Type::Str]));
    emitter
        .string_char_cache_required_names
        .insert("text".to_string());
    emitter
        .string_char_cache_vars
        .insert("existing".to_string(), "__sifr_existing_chars".to_string());
    let before = emitter.string_char_cache_vars.clone();
    let declined_then_body = vec![HirStmt::AttributeAugAssign {
        object: "self".to_string(),
        field: "label".to_string(),
        op: "+=".to_string(),
        value: HirExpr::StringLiteral("suffix".to_string()),
    }];
    let successful_elif_body = vec![HirStmt::Let {
        name: "text".to_string(),
        ty: Type::Str,
        value: HirExpr::StringLiteral("value".to_string()),
        is_mutable: false,
    }];

    let lowered = emitter
        .try_lower_if_stmt_for_ir(
            &isinstance_union_condition("int"),
            &declined_then_body,
            &[(isinstance_union_condition("str"), successful_elif_body)],
            None,
        )
        .expect("union lowering must not error");

    assert!(lowered.is_none());
    assert_eq!(emitter.string_char_cache_vars, before);
}
