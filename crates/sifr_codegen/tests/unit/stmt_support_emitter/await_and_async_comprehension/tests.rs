use super::*;

fn fallible_buffer_value(target_ty: Type) -> HirExpr {
    HirExpr::QuestionMark {
        expr: Box::new(HirExpr::Call {
            mutable_arg_places: Vec::new(),
            func: "make_buffer".to_string(),
            args: Vec::new(),
            ty: Type::Result(
                Box::new(Type::PythonBuffer(Box::new(Type::FixedInt(
                    sifr_type_system::FixedIntType::U8,
                )))),
                Box::new(Type::Str),
            ),
        }),
        ty: target_ty,
    }
}

#[test]
fn fallible_local_binding_coercion_uses_unwrapped_source_type() {
    let buffer_ty =
        Type::PythonBuffer(Box::new(Type::FixedInt(sifr_type_system::FixedIntType::U8)));
    let option_ty = Type::Union(vec![buffer_ty.clone(), Type::None]);
    let mut emitter = RustEmitter::new();
    let option_value = fallible_buffer_value(option_ty.clone());
    let lowered = emitter
        .coerce_local_value_for_target_type_for_ir(
            &option_ty,
            &option_value,
            RustExpr::Ident("value".to_string()),
        )
        .expect("option coercion");
    assert!(matches!(lowered, RustExpr::FnCall { func, .. }
        if matches!(func.as_ref(), RustExpr::Path(path) if path == &["Some".to_string()])));

    let buffer_variant = buffer_ty.union_variant_name();
    let union_ty = Type::Union(vec![buffer_ty, Type::Int, Type::None]);
    let union_value = fallible_buffer_value(union_ty.clone());
    let lowered = emitter
        .coerce_local_value_for_target_type_for_ir(
            &union_ty,
            &union_value,
            RustExpr::Ident("value".to_string()),
        )
        .expect("union coercion");
    assert!(matches!(lowered, RustExpr::FnCall { func, .. }
        if matches!(func.as_ref(), RustExpr::Path(path) if path.last() == Some(&buffer_variant))));
}
