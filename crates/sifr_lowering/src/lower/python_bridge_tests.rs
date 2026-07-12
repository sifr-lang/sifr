use crate::{ExternalDefs, LoweringOptions, PythonBridgeTargetAuthority};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;
use std::collections::BTreeMap;

const SOURCE: &str = r"
class PythonError(Error):
    message: str

@python(bridge.pkg.compute)
def compute(value: int) -> Result[int, PythonError]: ...
";

#[test]
fn bridge_target_is_a_hard_error_without_package_authority() {
    let parsed = parse_module(SOURCE).expect("source should parse");
    let result = crate::lower_module_with_externals_name_and_options(
        "main",
        parsed.suite(),
        &ExternalDefs::default(),
        LoweringOptions::default(),
    );
    let Err(errors) = result else {
        panic!("bridge target without an owning inventory should fail");
    };

    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION)));
}

#[test]
fn bridge_target_rewrites_to_resolved_package_when_activated() {
    let parsed = parse_module(SOURCE).expect("source should parse");
    let result = crate::lower_module_with_externals_name_and_options(
        "main",
        parsed.suite(),
        &ExternalDefs::default(),
        LoweringOptions {
            python_bridge_authorities: BTreeMap::from([(
                "main".to_string(),
                PythonBridgeTargetAuthority {
                    runtime_package: "__sifr_bridge__.p_abc123".to_string(),
                    modules: ["pkg".to_string()].into_iter().collect(),
                },
            )]),
            ..LoweringOptions::default()
        },
    )
    .expect("resolved package bridge should lower");
    let declaration = &result.module.functions[0].python_interop[0];

    assert_eq!(
        declaration.target.as_ref().expect("target").dotted(),
        "__sifr_bridge__.p_abc123.pkg.compute"
    );
    assert_eq!(declaration.required_import_root, None);
}

#[test]
fn bridge_target_requires_an_inventoried_module() {
    let parsed = parse_module(SOURCE).expect("source should parse");
    let result = crate::lower_module_with_externals_name_and_options(
        "main",
        parsed.suite(),
        &ExternalDefs::default(),
        LoweringOptions {
            python_bridge_authorities: BTreeMap::from([(
                "main".to_string(),
                PythonBridgeTargetAuthority {
                    runtime_package: "__sifr_bridge__.p_abc123".to_string(),
                    modules: ["other".to_string()].into_iter().collect(),
                },
            )]),
            ..LoweringOptions::default()
        },
    );
    let Err(errors) = result else {
        panic!("missing bridge module should fail");
    };

    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYIMP_INVALID_TARGET)));
}
