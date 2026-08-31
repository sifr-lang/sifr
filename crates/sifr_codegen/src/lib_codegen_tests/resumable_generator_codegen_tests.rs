use super::*;

fn test_error_type(name: &str) -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: name.to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    }
}
#[test]
fn test_generate_rust_generator_try_except_uses_resumable_runtime() {
    let module = HirModule {
        functions: vec![
            HirFunction {
                name: "gen".to_string(),
                params: vec![],
                return_type: Type::Iterator(Box::new(Type::Int)),
                body: vec![HirStmt::TryExcept {
                    body: vec![HirStmt::Yield {
                        value: HirExpr::IntLiteral(1),
                    }],
                    handlers: vec![HirExceptHandler {
                        error_type: Some("Error".to_string()),
                        error_resolved_type: None,
                        name: Some("e".to_string()),
                        body: vec![HirStmt::Yield {
                            value: HirExpr::IntLiteral(2),
                        }],
                    }],
                    body_error_types: vec![test_error_type("Error")],
                }],
                is_async: false,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: vec![],
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: vec![],
            },
            HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::For {
                    target: "v".to_string(),
                    target_ty: Type::Int,
                    iter: HirExpr::Call {
                        mutable_arg_places: Vec::new(),
                        func: "gen".to_string(),
                        args: vec![],
                        ty: Type::Iterator(Box::new(Type::Int)),
                    },
                    body: vec![HirStmt::Expr {
                        expr: HirExpr::Call {
                            mutable_arg_places: Vec::new(),
                            func: "print".to_string(),
                            args: vec![HirExpr::Name {
                                name: "v".to_string(),
                                binding_id: None,
                                ty: Type::Int,
                            }],
                            ty: Type::None,
                        },
                    }],
                    else_body: None,
                }],
                is_async: false,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: vec![],
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: vec![],
            },
        ],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("fn r#gen()"));
    assert!(rust_code.contains("r#gen()"));
    assert!(rust_code.contains("__SifrGenerator::new"));
    assert!(rust_code.contains("__sifr_yielder.suspend(SifrInt::from_i64(1)).await;"));
    assert!(rust_code.contains("__sifr_yielder.suspend(SifrInt::from_i64(2)).await;"));
    assert!(!rust_code.contains("__sifr_generator_initialized"));
    assert!(!rust_code.contains("_yields"));
    syn::parse_file(&rust_code).expect("resumable generator Rust should parse");
}
