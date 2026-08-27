use super::*;

#[test]
fn test_generate_rust_multi_with_metadata_preserves_trait_impl_visibility() {
    let main_module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    mutable_arg_places: Vec::new(),
                    func: "helper".to_string(),
                    args: vec![],
                    ty: Type::None,
                },
            }],
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
        imports: vec![HirImport {
            module: "helper".to_string(),
            names: vec!["helper".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let helper_module = HirModule {
        functions: vec![HirFunction {
            name: "helper".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    mutable_arg_places: Vec::new(),
                    func: "loads".to_string(),
                    args: vec![HirExpr::StringLiteral(
                        "name = \"fixture-five\"\nvalue = 5".to_string(),
                    )],
                    ty: Type::Result(Box::new(Type::Str), Box::new(Type::Any)),
                },
            }],
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
        imports: vec![HirImport {
            module: "sifr.tomllib".to_string(),
            names: vec!["loads".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let stdlib_code = trait_impl_fixture_stdlib_code();
    let result = generate_rust_multi_with_metadata(
        &[("main", &main_module), ("helper", &helper_module)],
        &stdlib_code,
    )
    .expect("code generation should succeed");

    let helper_rs = result
        .rust_files
        .get("helper")
        .expect("helper module should be generated");
    assert!(
        helper_rs.contains("pub fn helper()"),
        "support-module functions should be exported"
    );
    assert!(
        helper_rs.contains("impl ::std::fmt::Display for TOMLDecodeError"),
        "stdlib trait impls should be preserved in publicized helper modules"
    );
    assert!(
        !helper_rs.contains("pub fn fmt("),
        "trait impl methods must not receive pub visibility during support-module publicization"
    );
}
