use super::support::parse_suite;
use crate::{collect_project_hir_modules, compile_stdlib, type_check_source};
use sifr_diagnostics::DiagnosticCode;
use sifr_lowering::{LoweringOptions, PythonBridgeTargetAuthority};
use sifr_type_system::Type;
use std::collections::{BTreeMap, HashMap};

const HTTP: &str = include_str!(
    "../../../../verification/areas/python_interop/fixtures/async_declaration/httpx2_client.sifr"
);
const CONTEXT: &str = include_str!(
    "../../../../verification/areas/python_interop/fixtures/async_context/aiosqlite_session.sifr"
);

fn check_example(source: &str, bridge_module: &str) -> Vec<sifr_ir::HirDiagnostic> {
    let stdlib = compile_stdlib().expect("stdlib must compile");
    sifr_lowering::lower_module_with_externals_name_and_options(
        "main",
        &parse_suite(source),
        &stdlib.defs,
        LoweringOptions {
            python_bridge_authorities: BTreeMap::from([(
                "main".to_string(),
                PythonBridgeTargetAuthority {
                    runtime_package: "__sifr_bridge__.p_error_channel_test".to_string(),
                    modules: [bridge_module.to_string()].into_iter().collect(),
                },
            )]),
            ..LoweringOptions::default()
        },
    )
    .err()
    .unwrap_or_default()
}

#[test]
fn async_python_error_channel_preserves_both_original_examples() {
    for (source, bridge) in [(HTTP, "client"), (CONTEXT, "session")] {
        let errors = check_example(source, bridge);
        assert!(errors.is_empty(), "original async contract: {errors:?}");
    }
}

#[test]
fn async_python_error_channel_rejects_unrelated_return_errors() {
    for (source, bridge, count) in [(HTTP, "client", 1), (CONTEXT, "session", 3)] {
        let source = source.replace(
            "async def main() -> Result[None, Error]:",
            "async def main() -> Result[None, ValueError]:",
        );
        let errors = check_example(&source, bridge);
        assert_eq!(errors.len(), count, "{errors:?}");
        assert!(
            errors.iter().all(|error| {
                error.code == Some(DiagnosticCode::RESULT_INVALID_RAISE)
                    && error.message.contains("PythonError")
            }),
            "{errors:?}"
        );
    }
}

#[test]
fn async_python_error_channel_retains_stdlib_ancestry_without_data_parent() {
    let stdlib = compile_stdlib().expect("stdlib must compile");
    let error = &stdlib.defs.classes["sifr.python"]["PythonError"];
    let Type::Class {
        identity,
        parent_class,
        fields,
        ..
    } = error
    else {
        panic!("PythonError must be a nominal class");
    };
    assert_eq!(identity.as_deref(), Some("_sifr.python.PythonError"));
    assert_eq!(parent_class.as_deref(), Some("Error"));
    assert_eq!(fields.len(), 5);
    assert!(fields.iter().all(|(_, ty)| *ty == Type::Str));
}

#[test]
fn async_python_error_channel_preserves_local_and_imported_error_ancestry() {
    let modules = HashMap::from([
        (
            "errors".to_string(),
            parse_suite(
                "class DomainError(Error):\n    message: str\n\nclass DetailedError(DomainError):\n    detail: str\n\n    def __init__(self, message: str, detail: str):\n        super().__init__(message)\n        self.detail = detail\n",
            ),
        ),
        (
            "main".to_string(),
            parse_suite(
                "from errors import DomainError, DetailedError\n\ndef direct(own error: DomainError) -> Result[None, Error]:\n    raise error\n\ndef inherited(own error: DetailedError) -> Result[None, Error]:\n    raise error\n\ndef main():\n    pass\n",
            ),
        ),
    ]);
    let stdlib = compile_stdlib().expect("stdlib must compile");
    let lowered = collect_project_hir_modules(&modules, stdlib.defs)
        .expect("exported errors must retain their declared ancestry");
    let root = &lowered.hir_modules["errors"].classes[0];
    assert!(root.parent_class.is_none());
    assert!(root.parent_type.is_none());
    assert_eq!(root.semantic_parent_chain().as_deref(), Some("Error"));
    assert_eq!(root.fields.len(), 1);
}

#[test]
fn async_python_error_channel_rejects_same_named_nominal_target() {
    let source = "from sifr.python import PythonError\n\nclass Error(ValueError):\n    message: str\n\ndef fail(own error: PythonError) -> Result[None, Error]:\n    raise error\n\ndef main():\n    pass\n";
    let errors = type_check_source(source);
    assert!(
        errors
            .iter()
            .any(|error| { error.code == DiagnosticCode::RESULT_INVALID_RAISE.code() }),
        "same-name target must remain nominal: {errors:?}"
    );
}

#[test]
fn async_python_error_channel_preserves_same_named_stdlib_root_ancestry() {
    for module in ["sifr.csv", "sifr.configparser"] {
        let source = format!(
            "from {module} import Error as ImportedError\n\ndef propagate(own error: ImportedError) -> Result[None, Error]:\n    raise error\n\ndef main():\n    pass\n"
        );
        let errors = type_check_source(&source);
        assert!(errors.is_empty(), "{module}: {errors:?}");
    }
}
