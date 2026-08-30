use super::*;

#[test]
fn test_generate_rust_multi_with_metadata_infers_fs_feature_from_private_stdlib_source() {
    let main_module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    mutable_arg_places: Vec::new(),
                    func: "read_text".to_string(),
                    args: vec![HirExpr::StringLiteral("fixture.txt".to_string())],
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
            module: "sifr.io".to_string(),
            names: vec!["read_text".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let mut stdlib_code = StdlibCode::default();
    stdlib_code.transitive_deps.insert(
        "sifr.io".to_string(),
        HashSet::from(["_sifr.fs".to_string()]),
    );
    stdlib_code.module_rust_code.insert(
        "_sifr.fs".to_string(),
        StdlibRustSource {
            module: "_sifr.fs".to_string(),
            source_path: "stdlib/_sifr/fs.sifr".to_string(),
            source_sha256: "test".to_string(),
            nominal_types: HashSet::new(),
            rust: "fn read_text(path: &String) -> Result<String, IOError> {\n    ::sifr_stdlib::fs::read_text(path).map_err(|err| IOError { message: err.to_string(), kind: err.to_string() })\n}\n".to_string(),
        },
    );

    let result = generate_rust_multi_with_metadata(&[("main", &main_module)], &stdlib_code);

    assert!(
        result
            .rust_files
            .get("main")
            .expect("main module should be generated")
            .contains("::sifr_stdlib::fs::read_text")
    );
    assert!(
        result
            .required_features
            .contains(&sifr_stdlib_manifest::StdlibFeature::Fs)
    );
}

#[test]
fn test_generate_rust_multi_requires_runtime_for_absolute_private_stdlib_bridge_paths() {
    let main_module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![],
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
            module: "sifr.math".to_string(),
            names: vec!["isqrt".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };
    let mut stdlib_code = StdlibCode::default();
    stdlib_code.transitive_deps.insert(
        "sifr.math".to_string(),
        HashSet::from(["_sifr.math".to_string()]),
    );
    stdlib_code.module_rust_code.insert(
        "_sifr.math".to_string(),
        StdlibRustSource {
            module: "_sifr.math".to_string(),
            source_path: "stdlib/_sifr/math.sifr".to_string(),
            source_sha256: "test".to_string(),
            nominal_types: HashSet::new(),
            rust: "fn isqrt(n: SifrInt) -> SifrInt { ::sifr_runtime::interop::SifrIntBridge::from(n).to_i64_saturating() }\n".to_string(),
        },
    );

    let result = generate_rust_multi_with_metadata(&[("main", &main_module)], &stdlib_code);

    assert!(
        result
            .required_features
            .contains(&sifr_stdlib_manifest::StdlibFeature::SifrRuntime)
    );
}

#[test]
fn public_stdlib_reexport_uses_transitive_private_signature_for_call_borrowing() {
    let main_module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "path".to_string(),
                    ty: Type::Str,
                    value: HirExpr::StringLiteral("/tmp/sifr-codegen-remove-file".to_string()),
                    is_mutable: false,
                },
                HirStmt::Expr {
                    expr: HirExpr::Call {
                        mutable_arg_places: Vec::new(),
                        func: "remove_file".to_string(),
                        args: vec![HirExpr::Name {
                            name: "path".to_string(),
                            binding_id: None,
                            ty: Type::Str,
                        }],
                        ty: Type::Result(Box::new(Type::None), Box::new(Type::Any)),
                    },
                },
            ],
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
            module: "sifr.os".to_string(),
            names: vec!["remove_file".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let mut stdlib_code = StdlibCode::default();
    stdlib_code.transitive_deps.insert(
        "sifr.os".to_string(),
        HashSet::from(["_sifr.fs".to_string()]),
    );
    stdlib_code.func_signatures.insert(
        "_sifr.fs".to_string(),
        std::collections::HashMap::from([(
            "remove_file".to_string(),
            (
                vec![(Type::Str, ParamConvention::borrow())],
                Type::Result(Box::new(Type::None), Box::new(Type::Any)),
            ),
        )]),
    );

    let generated = generate_rust_with_stdlib_for_module(&main_module, &stdlib_code, None);

    assert!(
        generated.rust_source.contains("remove_file(&path);"),
        "public stdlib reexports should borrow according to transitive private signatures:\n{}",
        generated.rust_source
    );
}
