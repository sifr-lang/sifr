use super::*;
use sifr_ir::CompilerIntrinsicId;

fn module_with_expr(expr: HirExpr) -> HirModule {
    HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr { expr }],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    }
}

fn bytes_with_size(size: i64) -> HirExpr {
    HirExpr::IntrinsicCall {
        intrinsic: CompilerIntrinsicId::BytesWithSize,
        args: vec![HirExpr::IntLiteral(size)],
        ty: Type::Result(Box::new(Type::Bytes), Box::new(Type::Any)),
        call_range: Default::default(),
        arg_ranges: vec![Default::default()],
    }
}

#[test]
fn structured_intrinsic_supports_nested_intrinsic_arguments() {
    let module = module_with_expr(HirExpr::IntrinsicCall {
        intrinsic: CompilerIntrinsicId::TestAssertEqual,
        args: vec![bytes_with_size(2), bytes_with_size(2)],
        ty: Type::None,
        call_range: Default::default(),
        arg_ranges: vec![Default::default(), Default::default()],
    });

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("assert_eq!("));
    assert!(generated
        .rust_source
        .contains("bytes(size) requires a non-negative size"));
    assert!(generated.lowering_stats.expr_structured > 0);
}

#[test]
fn structured_intrinsic_supports_typed_method_call_arguments() {
    let module = module_with_expr(HirExpr::IntrinsicCall {
        intrinsic: CompilerIntrinsicId::BytesFromHex,
        args: vec![HirExpr::MethodCall {
            object: Box::new(HirExpr::StringLiteral("A0".to_string())),
            method: "lower".to_string(),
            args: vec![],
            receiver_convention: Some(sifr_type_system::ReceiverConvention::SharedBorrow),
            receiver_target: None,
            mutable_arg_places: Vec::new(),
            source: None,
            ty: Type::Str,
        }],
        ty: Type::Result(Box::new(Type::Bytes), Box::new(Type::Any)),
        call_range: Default::default(),
        arg_ranges: vec![Default::default()],
    });

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains(".to_lowercase()"));
    assert!(generated
        .rust_source
        .contains("u8::from_str_radix(pair_str, 16)"));
    assert!(generated.lowering_stats.expr_structured > 0);
}
