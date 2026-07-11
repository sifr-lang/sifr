use super::*;
use sifr_ir::CompilerIntrinsicId;

fn counter_total_expr() -> HirExpr {
    HirExpr::IntrinsicCall {
        intrinsic: CompilerIntrinsicId::CounterTotal,
        args: vec![HirExpr::IntrinsicCall {
            intrinsic: CompilerIntrinsicId::CounterFromList,
            args: vec![HirExpr::ListLiteral {
                elements: vec![HirExpr::StringLiteral("a".to_string())],
                ty: Type::List(Box::new(Type::Str)),
            }],
            ty: Type::Str,
            call_range: Default::default(),
            arg_ranges: vec![Default::default()],
        }],
        ty: Type::Int,
        call_range: Default::default(),
        arg_ranges: vec![Default::default()],
    }
}

#[test]
fn test_structured_expr_path_handles_intrinsic_call_expression() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: counter_total_expr(),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.collections".to_string(),
            names: vec!["counter_total".to_string(), "counter_from_list".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };
    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains(".values().sum::<i64>()"));
    assert!(
        generated.lowering_stats.expr_structured > 0,
        "intrinsic call should be emitted through structured expr path"
    );
    assert!(
        generated.lowering_stats.stmt_structured > 0,
        "expression statement should be emitted through structured stmt path"
    );
}

#[test]
fn test_structured_expr_path_handles_nested_intrinsic_call_argument() {
    let list_ty = Type::List(Box::new(Type::Str));
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::IntrinsicCall {
                    intrinsic: CompilerIntrinsicId::CounterGet,
                    args: vec![
                        HirExpr::IntrinsicCall {
                            intrinsic: CompilerIntrinsicId::CounterFromList,
                            args: vec![HirExpr::ListLiteral {
                                elements: vec![HirExpr::StringLiteral("a".to_string())],
                                ty: list_ty.clone(),
                            }],
                            ty: Type::Str,
                            call_range: Default::default(),
                            arg_ranges: vec![Default::default()],
                        },
                        HirExpr::StringLiteral("a".to_string()),
                    ],
                    ty: Type::Int,
                    call_range: Default::default(),
                    arg_ranges: vec![Default::default(), Default::default()],
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.collections".to_string(),
            names: vec!["counter_get".to_string(), "counter_from_list".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };
    let generated = generate_rust_with_metadata(&module);
    assert!(
        generated.rust_source.contains("serde_json::from_str"),
        "nested intrinsic call argument should lower through registry"
    );
    assert!(
        generated.rust_source.contains("HashMap::<String, i64>"),
        "nested counter construction should lower through registry"
    );
    assert!(
        !generated.rust_source.contains("counter_get("),
        "counter_get should not be emitted as unresolved function call"
    );
}

#[test]
fn test_structured_expr_path_handles_intrinsic_arg_with_typed_method_call() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::IntrinsicCall {
                    intrinsic: CompilerIntrinsicId::CounterGet,
                    args: vec![
                        HirExpr::IntrinsicCall {
                            intrinsic: CompilerIntrinsicId::CounterFromList,
                            args: vec![HirExpr::ListLiteral {
                                elements: vec![HirExpr::StringLiteral("path".to_string())],
                                ty: Type::List(Box::new(Type::Str)),
                            }],
                            ty: Type::Str,
                            call_range: Default::default(),
                            arg_ranges: vec![Default::default()],
                        },
                        HirExpr::MethodCall {
                            object: Box::new(HirExpr::StringLiteral("PATH".to_string())),
                            method: "lower".to_string(),
                            args: vec![],
                            ty: Type::Str,
                        },
                    ],
                    ty: Type::Int,
                    call_range: Default::default(),
                    arg_ranges: vec![Default::default(), Default::default()],
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.collections".to_string(),
            names: vec!["counter_get".to_string(), "counter_from_list".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(
        generated.rust_source.contains("serde_json::from_str"),
        "intrinsic arg with typed method call should lower through registry"
    );
    assert!(
        generated.rust_source.contains(".to_lowercase()"),
        "typed method call argument should lower through registry"
    );
    assert!(
        !generated.rust_source.contains("counter_get("),
        "counter_get should not be emitted as unresolved function call"
    );
}
