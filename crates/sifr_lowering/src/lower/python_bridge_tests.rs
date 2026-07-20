use crate::{ExternalDefs, LoweringOptions, PythonBridgeTargetAuthority, PythonTrustPolicy};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;
use std::collections::BTreeMap;

const SOURCE: &str = r"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str

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
        .any(|error| error.code == Some(DiagnosticCode::PYIMP_INVALID_TARGET)));
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

#[test]
fn reserved_bridge_target_cannot_be_reinterpreted_as_an_external_distribution() {
    let parsed = parse_module(SOURCE).expect("source should parse");
    let result = crate::lower_module_with_externals_name_and_options(
        "main",
        parsed.suite(),
        &ExternalDefs::default(),
        LoweringOptions {
            python_trust_policy: Some(PythonTrustPolicy {
                required_import_roots: vec!["bridge".to_string()],
                trusted_import_roots: vec!["bridge".to_string()],
            }),
            ..LoweringOptions::default()
        },
    );
    let Err(errors) = result else {
        panic!("reserved bridge target must not fall through to an external distribution");
    };
    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYIMP_INVALID_TARGET)));

    let non_reserved = SOURCE.replace("bridge.pkg.compute", "external_bridge.pkg.compute");
    let parsed = parse_module(&non_reserved).expect("non-reserved source should parse");
    let lowered = crate::lower_module_with_externals_name_and_options(
        "main",
        parsed.suite(),
        &ExternalDefs::default(),
        LoweringOptions {
            python_trust_policy: Some(PythonTrustPolicy {
                required_import_roots: vec!["external_bridge".to_string()],
                trusted_import_roots: vec!["external_bridge".to_string()],
            }),
            ..LoweringOptions::default()
        },
    )
    .expect("external distribution remains reachable through a non-reserved target");
    let declaration = &lowered.module.functions[0].python_interop[0];
    assert_eq!(
        declaration.required_import_root.as_deref(),
        Some("external_bridge")
    );
}

#[test]
fn bridge_authority_is_scoped_to_the_declaring_module() {
    let parsed = parse_module(SOURCE).expect("source should parse");
    let result = crate::lower_module_with_externals_name_and_options(
        "main",
        parsed.suite(),
        &ExternalDefs::default(),
        LoweringOptions {
            python_bridge_authorities: BTreeMap::from([(
                "other".to_string(),
                PythonBridgeTargetAuthority {
                    runtime_package: "__sifr_bridge__.p_abc123".to_string(),
                    modules: ["pkg".to_string()].into_iter().collect(),
                },
            )]),
            ..LoweringOptions::default()
        },
    );
    let Err(errors) = result else {
        panic!("authority owned by another module must not activate this target");
    };
    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYIMP_INVALID_TARGET)));
}

#[test]
fn nested_inventoried_bridge_module_rewrites_to_the_resolved_package() {
    let source = SOURCE.replace("bridge.pkg.compute", "bridge.pkg.sub.compute");
    let parsed = parse_module(&source).expect("source should parse");
    let lowered = crate::lower_module_with_externals_name_and_options(
        "main",
        parsed.suite(),
        &ExternalDefs::default(),
        LoweringOptions {
            python_bridge_authorities: BTreeMap::from([(
                "main".to_string(),
                PythonBridgeTargetAuthority {
                    runtime_package: "__sifr_bridge__.p_abc123".to_string(),
                    modules: ["pkg.sub".to_string()].into_iter().collect(),
                },
            )]),
            ..LoweringOptions::default()
        },
    )
    .expect("nested inventoried module should lower");
    assert_eq!(
        lowered.module.functions[0].python_interop[0]
            .target
            .as_ref()
            .expect("target")
            .dotted(),
        "__sifr_bridge__.p_abc123.pkg.sub.compute"
    );
}
