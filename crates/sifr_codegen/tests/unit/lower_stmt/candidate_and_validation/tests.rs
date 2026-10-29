use super::*;

#[test]
fn local_binding_setdefault_materializes_owned_key_and_default() {
    let expr = HirExpr::MethodCall {
        authority: crate::method_authority_tests::builtin_authority("setdefault"),
        object: Box::new(HirExpr::Name {
            name: "defaults".to_string(),
            binding_id: None,
            ty: Type::Any,
        }),
        method: "setdefault".to_string(),
        args: vec![
            HirExpr::Name {
                name: "key".to_string(),
                binding_id: None,
                ty: Type::Str,
            },
            HirExpr::Name {
                name: "value".to_string(),
                binding_id: None,
                ty: Type::Str,
            },
        ],
        receiver_convention: Some(sifr_type_system::ReceiverConvention::MutableBorrow),
        receiver_target: None,
        mutable_arg_places: Vec::new(),
        source: None,
        ty: Type::Str,
    };
    let bindings = HashMap::from([(
        "defaults".to_string(),
        Type::Dict(Box::new(Type::Str), Box::new(Type::Str)),
    )]);

    let lowered = try_lower_expr_stmt_with_bindings(&expr, &bindings)
        .expect("local binding method call lowers");
    let rendered = crate::render_stmts(&lowered);

    assert_eq!(
        rendered.trim(),
        "defaults.entry(key.to_owned()).or_insert(value.to_owned()).to_owned();"
    );
    assert!(!rendered.contains(".clone()"), "{rendered}");
}
