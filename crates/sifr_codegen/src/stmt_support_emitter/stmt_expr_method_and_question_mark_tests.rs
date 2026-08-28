use super::*;

fn list_method_call(method: &str, args: Vec<HirExpr>, result_ty: Type) -> HirExpr {
    let list_ty = Type::List(Box::new(Type::Int));
    HirExpr::MethodCall {
        object: Box::new(HirExpr::Name {
            name: "items".to_string(),
            binding_id: Some(sifr_ir::BindingId(1)),
            ty: list_ty,
        }),
        method: method.to_string(),
        args,
        receiver_convention: Some(if method == "append" {
            sifr_type_system::ReceiverConvention::MutableBorrow
        } else {
            sifr_type_system::ReceiverConvention::SharedBorrow
        }),
        receiver_target: (method == "append").then(|| {
            sifr_ir::MutableReceiverTarget::Place(sifr_ir::Place {
                root: sifr_ir::BindingId(1),
                projections: Vec::new(),
            })
        }),
        mutable_arg_places: vec![None; usize::from(method == "append")],
        source: None,
        ty: result_ty,
    }
}

#[test]
fn generic_imported_project_calls_use_the_canonical_function_name() {
    let expression = HirExpr::GenericCall {
        func: "load::<i64>".to_string(),
        type_args: vec![Type::Int],
        args: Vec::new(),
        mutable_arg_places: Vec::new(),
        ty: Type::None,
    };
    let imported = std::collections::HashSet::from(["load".to_string()]);

    assert!(is_imported_project_call_for_ir(&expression, &imported));
}

#[test]
fn statement_list_methods_use_registry_authority_after_legacy_fallback_removal() {
    let mut emitter = RustEmitter::new();
    let append = list_method_call("append", vec![HirExpr::IntLiteral(1)], Type::None);
    let cloned = list_method_call("cloned", Vec::new(), Type::List(Box::new(Type::Int)));

    let lowered_append = emitter
        .lower_stmt_expr_for_ir(&append)
        .expect("append lowering must not error")
        .expect("append must be accepted by registry authority");
    let lowered_cloned = emitter
        .lower_stmt_expr_for_ir(&cloned)
        .expect("cloned lowering must not error")
        .expect("cloned must be accepted by registry authority");

    assert!(matches!(
        lowered_append,
        crate::RustExpr::MethodCall { method, .. } if method == "push"
    ));
    assert!(matches!(
        lowered_cloned,
        crate::RustExpr::MethodCall { method, .. } if method == "clone"
    ));
}

#[test]
fn structured_fallback_uses_registry_authority_after_strict_argument_decline() {
    let mut emitter = RustEmitter::new();
    let append = list_method_call(
        "append",
        vec![HirExpr::Call {
            func: "predicate".to_string(),
            args: Vec::new(),
            mutable_arg_places: Vec::new(),
            ty: Type::Int,
        }],
        Type::None,
    );

    let lowered = emitter
        .lower_stmt_expr_for_ir(&append)
        .expect("append lowering must not error")
        .expect("structured fallback must lower the append");

    assert!(matches!(
        lowered,
        crate::RustExpr::MethodCall { method, args, .. }
            if method == "push"
                && matches!(args.first(), Some(crate::RustExpr::FnCall { .. }))
    ));
}
