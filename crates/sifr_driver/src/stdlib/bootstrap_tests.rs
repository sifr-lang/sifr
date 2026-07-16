use super::*;
use sifr_lowering::CompilerIntrinsicId;
use std::path::PathBuf;

fn sample_param(name: &str, ty: Type, convention: ParamConvention) -> HirParam {
    HirParam {
        name: name.to_string(),
        ty,
        default: None,
        keyword_only: false,
        convention,
    }
}

#[test]
fn function_type_from_params_preserves_named_conventions() {
    let params = vec![
        sample_param("value", Type::Int, ParamConvention::borrow()),
        sample_param("count", Type::Int, ParamConvention::own()),
    ];

    let function_type = function_type_from_params(&params, &Type::Bool);

    assert_eq!(
        function_type,
        FunctionType {
            params: vec![
                ("value".to_string(), Type::Int, ParamConvention::borrow()),
                ("count".to_string(), Type::Int, ParamConvention::own()),
            ],
            return_type: Box::new(Type::Bool),
        }
    );
}

#[test]
fn function_type_from_hir_exports_async_functions_as_coroutines() {
    let function = HirFunction {
        name: "async_status".to_string(),
        params: vec![sample_param("value", Type::Int, ParamConvention::borrow())],
        return_type: Type::Result(Box::new(Type::Bool), Box::new(Type::Str)),
        body: Vec::new(),
        is_async: true,
        method_kind: sifr_ir::MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    assert_eq!(
        function_type_from_hir(&function),
        FunctionType {
            params: vec![("value".to_string(), Type::Int, ParamConvention::borrow())],
            return_type: Box::new(Type::Coroutine(Box::new(Type::Bool), Box::new(Type::Str))),
        }
    );
}

#[test]
fn signature_params_can_override_constructor_conventions() {
    let params = vec![sample_param("self", Type::Str, ParamConvention::borrow())];

    assert_eq!(
        signature_params(&params, Some(ParamConvention::own())),
        vec![(Type::Str, ParamConvention::own())]
    );
}

#[test]
fn public_constant_integer_value_exports_filter_to_public_recorded_values() {
    let mut values = HashMap::new();
    values.insert("ANSWER".to_string(), 42);
    values.insert("_PRIVATE".to_string(), 99);
    values.insert("STALE".to_string(), 100);

    let exports =
        collect_public_constant_integer_value_exports(["ANSWER", "MISSING"].into_iter(), &values);

    assert_eq!(exports.len(), 1);
    assert_eq!(exports.get("ANSWER"), Some(&42));
    assert!(!exports.contains_key("_PRIVATE"));
    assert!(!exports.contains_key("STALE"));
}

#[test]
fn stdlib_class_exports_preserve_parent_markers_and_generic_templates() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let object_ty = compiled
        .defs
        .classes
        .get("sifr.python")
        .and_then(|classes| classes.get("Object"))
        .expect("sifr.python.Object should be exported");

    assert!(matches!(
        object_ty,
        Type::Class {
            identity: Some(identity),
            parent_class: Some(parent),
            ..
        } if identity == "_sifr.python.Object"
            && parent.split('|').any(|name| name == "NonSend")
    ));

    let shared_ty = compiled
        .defs
        .classes
        .get("sifr.sync")
        .and_then(|classes| classes.get("Shared"))
        .expect("sifr.sync.Shared should be exported");
    assert!(matches!(
        shared_ty,
        Type::Class { type_args, .. }
            if type_args == &vec![Type::TypeVar("T".to_string())]
    ));
}

#[test]
fn python_core_re_exports_preserve_callable_metadata() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let workloads = compiled
        .defs
        .function_workloads
        .get("sifr.python")
        .expect("sifr.python should export workload metadata");

    assert_eq!(
        workloads.get("threadsafe_callback_echo"),
        Some(&"blocking_io".to_string())
    );
    assert_eq!(
        workloads.get("close_local_callback"),
        Some(&"blocking_io".to_string())
    );

    let defaults = compiled
        .defs
        .function_defaults
        .get("sifr.python")
        .and_then(|module_defaults| module_defaults.get("PythonError"))
        .expect("PythonError constructor defaults should be re-exported");
    assert_eq!(defaults.len(), 4);
}

#[test]
fn python_call_helpers_borrow_argument_collections() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let functions = compiled
        .defs
        .functions
        .get("sifr.python")
        .expect("sifr.python functions should be exported");

    for function_name in ["call", "call_attr"] {
        let function = functions
            .get(function_name)
            .unwrap_or_else(|| panic!("sifr.python.{function_name} should be exported"));
        for parameter_name in ["args", "kwargs"] {
            let convention = function
                .params
                .iter()
                .find_map(|(name, _, convention)| (name == parameter_name).then_some(*convention))
                .unwrap_or_else(|| {
                    panic!("sifr.python.{function_name} should export {parameter_name}")
                });
            assert!(
                convention.is_shared_borrow(),
                "sifr.python.{function_name} must borrow {parameter_name} so temporary argument lists clone their handles"
            );
        }
    }
}

#[test]
fn retained_public_declarations_export_typed_compiler_identity() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    assert_eq!(
        compiled
            .defs
            .compiler_intrinsics
            .get("sifr.test")
            .and_then(|ids| ids.get("assert_eq")),
        Some(&CompilerIntrinsicId::TestAssertEqual)
    );
    assert_eq!(
        compiled
            .defs
            .compiler_intrinsics
            .get("sifr.task")
            .and_then(|ids| ids.get("current_context")),
        Some(&CompilerIntrinsicId::TaskCurrentContext)
    );
}

fn fixture_source(module: &str, source: &str, kind: LoadedStdlibSourceKind) -> LoadedStdlibSource {
    let stdlib_root =
        std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"))
            .expect("development stdlib root should resolve");
    LoadedStdlibSource {
        module: module.to_string(),
        source: source.to_string(),
        path: stdlib_root.join(format!("{}.sifr", module.replace('.', "/"))),
        kind,
    }
}

fn compile_fixture_sources(
    sources: &[LoadedStdlibSource],
) -> Result<StdlibCompiled, Vec<RenderedDiagnostic>> {
    let sysroot = sifr_sysroot::resolve_sysroot(None).expect("development sysroot should resolve");
    compile_stdlib_sources_with_sysroot(sources, sysroot)
}

fn fixture_diagnostics(sources: &[LoadedStdlibSource]) -> Vec<RenderedDiagnostic> {
    match compile_fixture_sources(sources) {
        Ok(_) => panic!("fixture should fail stdlib bootstrap"),
        Err(diagnostics) => diagnostics,
    }
}

#[test]
fn private_stdlib_imports_resolve_only_from_compiled_source_exports() {
    let sources = [
        fixture_source(
            "_sifr.fixture",
            "def existing(value: int) -> int:\n    return value\n",
            LoadedStdlibSourceKind::PrivateDeclaration,
        ),
        fixture_source(
            "sifr.fixture",
            "from _sifr.fixture import existing\n\ndef forwarded(value: int) -> int:\n    return existing(value)\n",
            LoadedStdlibSourceKind::Public,
        ),
    ];

    let compiled = compile_fixture_sources(&sources).expect("source-backed import should compile");
    assert!(compiled
        .defs
        .functions
        .get("sifr.fixture")
        .is_some_and(|functions| functions.contains_key("forwarded")));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.fixture")
        .is_some_and(|deps| deps.contains("_sifr.fixture")));
}

#[test]
fn missing_private_stdlib_member_is_a_structured_bootstrap_failure() {
    let sources = [
        fixture_source(
            "_sifr.fixture",
            "def existing(value: int) -> int:\n    return value\n",
            LoadedStdlibSourceKind::PrivateDeclaration,
        ),
        fixture_source(
            "sifr.fixture",
            "from _sifr.fixture import absent\n",
            LoadedStdlibSourceKind::Public,
        ),
    ];

    let diagnostics = fixture_diagnostics(&sources);
    assert!(!diagnostics.is_empty());
    assert!(diagnostics
        .iter()
        .all(|diagnostic| diagnostic.code == DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE.code()));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains("_sifr.fixture") && diagnostic.message.contains("absent")
    }));
}

#[test]
fn missing_private_stdlib_module_is_a_structured_bootstrap_failure() {
    let sources = [fixture_source(
        "sifr.fixture",
        "from _sifr.missing import absent\n",
        LoadedStdlibSourceKind::Public,
    )];

    let diagnostics = fixture_diagnostics(&sources);
    assert!(!diagnostics.is_empty());
    assert!(diagnostics
        .iter()
        .all(|diagnostic| diagnostic.code == DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE.code()));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("_sifr.missing")));
}
