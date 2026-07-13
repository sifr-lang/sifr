use crate::{interop_build_plan_for_named_modules, PythonTargetProbeStatus};
use ruff_text_size::TextRange;
use sifr_ir::{
    HirClass, HirClassKind, HirExpr, HirFunction, HirImport, HirModule, HirStmt, MethodKind,
    PythonInteropDeclaration, PythonInteropDecoratorKind, PythonInteropEffect,
    PythonRecordExpansion, PythonTargetPath,
};
use sifr_type_system::Type;

#[test]
fn plan_retains_deferred_probe_requirements_record_constraint_and_cache_identity() {
    let declaration = PythonInteropDeclaration {
        kind: PythonInteropDecoratorKind::Function,
        target: Some(PythonTargetPath {
            segments: vec!["json".to_string(), "dumps".to_string()],
            span: TextRange::default(),
        }),
        span: TextRange::default(),
        effect: PythonInteropEffect::BlockingIo,
        cleanup: None,
        consumes_receiver: false,
        parameters: Vec::new(),
        required_import_root: Some("json".to_string()),
    };
    let module = HirModule {
        functions: vec![
            function("dumps", Vec::new(), vec![declaration]),
            function(
                "main",
                vec![HirStmt::Expr {
                    expr: HirExpr::PythonCall {
                        func: "dumps".to_string(),
                        args: Vec::new(),
                        provided_arguments: Vec::new(),
                        record_expansions: vec![PythonRecordExpansion {
                            span: TextRange::default(),
                            fields: vec!["indent".to_string()],
                        }],
                        ty: Type::Str,
                    },
                }],
                Vec::new(),
            ),
        ],
        classes: Vec::new(),
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let plan = interop_build_plan_for_named_modules([(Some("main"), &module)]);

    assert_eq!(plan.python.required_import_roots, ["json"]);
    assert_eq!(plan.python.target_probes.len(), 1);
    assert_eq!(
        plan.python.target_probes[0].status,
        PythonTargetProbeStatus::Planned
    );
    assert!(plan.python.target_probes[0].requires_inspectable_signature);
    let cache_key = plan.cache_key_fragment();
    assert!(cache_key.contains("python.target=json.dumps"));
    assert!(cache_key.contains("python.binding_contract=sifr-python-binding-v1"));
    assert!(cache_key.contains("python.return_type=None"));
    assert!(cache_key.contains("python.required_import=json"));
    assert!(cache_key.contains("python.probe=json.dumps:inspectable:callable:planned"));
}

#[test]
fn python_cache_identity_changes_with_authoritative_sifr_types() {
    let declaration = PythonInteropDeclaration {
        kind: PythonInteropDecoratorKind::Function,
        target: Some(PythonTargetPath {
            segments: vec!["json".to_string(), "loads".to_string()],
            span: TextRange::default(),
        }),
        span: TextRange::default(),
        effect: PythonInteropEffect::BlockingIo,
        cleanup: None,
        consumes_receiver: false,
        parameters: Vec::new(),
        required_import_root: Some("json".to_string()),
    };
    let mut string_result = function("loads", Vec::new(), vec![declaration.clone()]);
    string_result.return_type = Type::Str;
    let mut integer_result = function("loads", Vec::new(), vec![declaration]);
    integer_result.return_type = Type::Int;
    let string_module = module_with_functions(vec![string_result]);
    let integer_module = module_with_functions(vec![integer_result]);

    let string_plan = interop_build_plan_for_named_modules([(Some("main"), &string_module)]);
    let integer_plan = interop_build_plan_for_named_modules([(Some("main"), &integer_module)]);

    assert_ne!(
        string_plan.cache_key_fragment(),
        integer_plan.cache_key_fragment()
    );
}

#[test]
fn raw_coroutine_call_requires_the_owned_async_loop() {
    let mut module = module_with_functions(vec![function(
        "main",
        vec![HirStmt::Expr {
            expr: HirExpr::Call {
                func: "run_coroutine_blocking".to_string(),
                args: Vec::new(),
                ty: Type::None,
            },
        }],
        Vec::new(),
    )]);
    module.imports.push(HirImport {
        module: "sifr.python".to_string(),
        names: vec!["run_coroutine_blocking".to_string()],
        aliases: Vec::new(),
    });

    let plan = interop_build_plan_for_named_modules([(Some("main"), &module)]);

    assert!(plan.python.requires_async_loop);
    assert!(plan
        .cache_key_fragment()
        .contains("python.requires_async_loop=yes"));
}

#[test]
fn aliased_raw_coroutine_call_requires_the_owned_async_loop() {
    let mut module = module_with_functions(vec![function(
        "main",
        vec![HirStmt::Expr {
            expr: HirExpr::Call {
                func: "run_owned".to_string(),
                args: Vec::new(),
                ty: Type::None,
            },
        }],
        Vec::new(),
    )]);
    module.imports.push(HirImport {
        module: "sifr.python".to_string(),
        names: vec!["run_coroutine_blocking".to_string()],
        aliases: vec![(
            "run_coroutine_blocking".to_string(),
            "run_owned".to_string(),
        )],
    });

    let plan = interop_build_plan_for_named_modules([(Some("main"), &module)]);

    assert!(plan.python.requires_async_loop);
}

#[test]
fn sync_python_declaration_does_not_require_the_owned_async_loop() {
    let module = module_with_functions(vec![function("main", Vec::new(), Vec::new())]);
    let plan = interop_build_plan_for_named_modules([(Some("main"), &module)]);

    assert!(!plan.python.requires_async_loop);
}

#[test]
fn method_only_async_python_declaration_requires_owned_loop() {
    let mut method = function("work", Vec::new(), Vec::new());
    method.is_async = true;
    method.python_interop.push(PythonInteropDeclaration {
        kind: PythonInteropDecoratorKind::Coroutine,
        target: Some(PythonTargetPath {
            segments: vec!["Self".to_string(), "work".to_string()],
            span: TextRange::default(),
        }),
        span: TextRange::default(),
        effect: PythonInteropEffect::Async,
        cleanup: None,
        consumes_receiver: false,
        parameters: Vec::new(),
        required_import_root: None,
    });
    let mut module = module_with_functions(Vec::new());
    module.classes.push(HirClass {
        name: "Client".to_string(),
        fields: Vec::new(),
        methods: vec![method],
        is_hashable: false,
        is_error_type: false,
        kind: HirClassKind::Regular,
        operator_impls: Vec::new(),
        newtype_inner: None,
        implements_protocols: Vec::new(),
        parent_class: Some("NonSend".to_string()),
        type_params: Vec::new(),
        enum_variants: Vec::new(),
        rust_interop: Vec::new(),
    });

    let plan = interop_build_plan_for_named_modules([(Some("main"), &module)]);

    assert!(plan.python.requires_async_loop);
}

fn module_with_functions(functions: Vec<HirFunction>) -> HirModule {
    HirModule {
        functions,
        classes: Vec::new(),
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    }
}

fn function(
    name: &str,
    body: Vec<HirStmt>,
    python_interop: Vec<PythonInteropDeclaration>,
) -> HirFunction {
    HirFunction {
        name: name.to_string(),
        params: Vec::new(),
        return_type: Type::None,
        body,
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop,
        compiler_intrinsic: None,
        type_params: Vec::new(),
    }
}
