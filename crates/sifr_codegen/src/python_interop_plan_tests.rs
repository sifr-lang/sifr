use crate::{interop_build_plan_for_named_modules, PythonTargetProbeStatus};
use ruff_text_size::TextRange;
use sifr_ir::{
    HirExpr, HirFunction, HirModule, HirStmt, MethodKind, PythonInteropDeclaration,
    PythonInteropDecoratorKind, PythonInteropEffect, PythonRecordExpansion, PythonTargetPath,
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
    assert!(cache_key.contains("python.required_import=json"));
    assert!(cache_key.contains("python.probe=json.dumps:inspectable:planned"));
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
