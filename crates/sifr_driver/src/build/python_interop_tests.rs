use super::*;
use crate::build::project_codegen::GeneratedBinaryProject;
use sifr_codegen::{
    InteropBuildPlan, PythonInteropPlan, PythonTargetProbe, PythonTargetProbeStatus,
};
use std::collections::{BTreeMap, HashSet};

#[test]
fn inspectable_target_is_verified() {
    let runtime = PackagePythonRuntime::for_tests(python(), "probe");
    let generated = apply_python_interop_metadata(project("math.sqrt"), Some(&runtime))
        .expect("math.sqrt should probe");
    assert_eq!(
        generated.interop.python.target_probes[0].status,
        PythonTargetProbeStatus::Verified
    );
}

#[test]
fn non_callable_target_is_rejected() {
    let runtime = PackagePythonRuntime::for_tests(python(), "probe");
    let Err(diagnostics) = apply_python_interop_metadata(project("math.pi"), Some(&runtime)) else {
        panic!("math.pi is not callable");
    };
    assert_eq!(diagnostics[0].code, "SIFR-PYCALL-0001");
}

#[test]
fn opaque_probe_requires_a_python_type() {
    let runtime = PackagePythonRuntime::for_tests(python(), "probe");
    let mut generated = project("math.sqrt");
    generated.interop.python.target_probes[0].expects_type = true;
    generated.interop.python.declarations[0].declaration.kind =
        sifr_ir::PythonInteropDecoratorKind::Opaque;
    let Err(diagnostics) = apply_python_interop_metadata(generated, Some(&runtime)) else {
        panic!("opaque target must be a Python type");
    };
    assert_eq!(diagnostics[0].code, "SIFR-PYCALL-0001");

    let mut generated = project("builtins.str");
    generated.interop.python.target_probes[0].expects_type = true;
    generated.interop.python.declarations[0].declaration.kind =
        sifr_ir::PythonInteropDecoratorKind::Opaque;
    apply_python_interop_metadata(generated, Some(&runtime))
        .expect("Python class target should verify");
}

#[test]
fn opaque_target_does_not_validate_constructor_signature() {
    let mut generated = project("pkg.Connection");
    generated.interop.python.declarations[0].declaration.kind =
        sifr_ir::PythonInteropDecoratorKind::Opaque;
    generated.interop.python.target_probes[0].expects_type = true;
    let inspection = PythonTargetInspection {
        ok: true,
        callable: true,
        is_type: true,
        inspectable: true,
        parameters: vec![probe_parameter("connector", "POSITIONAL_OR_KEYWORD", false)],
        error: None,
    };

    assert!(
        apply_python_target_inspection(
            &mut generated.interop.python,
            "pkg.Connection",
            Ok(&inspection),
        )
        .is_empty()
    );
    assert_eq!(
        generated.interop.python.target_probes[0].status,
        PythonTargetProbeStatus::Verified
    );
}

#[test]
fn uninspectable_callable_is_runtime_checked() {
    let runtime = PackagePythonRuntime::for_tests(python(), "probe");
    let generated = apply_python_interop_metadata(project("builtins.dir"), Some(&runtime))
        .expect("uninspectable callable should remain runtime checked");
    assert_eq!(
        generated.interop.python.target_probes[0].status,
        PythonTargetProbeStatus::RuntimeChecked
    );
}

#[test]
fn embedded_bridge_target_skips_external_interpreter_probe() {
    let runtime = PackagePythonRuntime::for_tests("/definitely/missing/python", "probe");
    let generated = apply_python_interop_metadata(
        project("__sifr_bridge__.p_abc123.adapter.value"),
        Some(&runtime),
    )
    .expect("embedded bridge target should be runtime checked by the reserved loader");

    assert_eq!(
        generated.interop.python.target_probes[0].status,
        PythonTargetProbeStatus::RuntimeChecked
    );
}

#[test]
fn read_only_check_defers_library_target_without_an_environment() {
    let generated = apply_python_interop_metadata_for_check(project("math.sqrt"), None)
        .expect("library checks defer final-application probing");
    assert_eq!(
        generated.interop.python.target_probes[0].status,
        PythonTargetProbeStatus::Planned
    );
    let Err(error) = apply_python_interop_metadata(project("math.sqrt"), None) else {
        panic!("builds still require a selected environment");
    };
    assert_eq!(error[0].code, "SIFR-PYENV-0003");
}

#[test]
fn read_only_check_marks_embedded_bridge_target_runtime_checked_when_deferred() {
    let generated = apply_python_interop_metadata_for_check(
        project("__sifr_bridge__.p_abc123.adapter.value"),
        None,
    )
    .expect("embedded bridge structure is checked without an external interpreter");
    assert_eq!(
        generated.interop.python.target_probes[0].status,
        PythonTargetProbeStatus::RuntimeChecked
    );
}

#[test]
fn inspectable_incompatible_arity_is_rejected() {
    let runtime = PackagePythonRuntime::for_tests(python(), "probe");
    let Err(diagnostics) = apply_python_interop_metadata(project("math.pow"), Some(&runtime))
    else {
        panic!("math.pow requires two positional parameters");
    };
    assert_eq!(diagnostics[0].code, "SIFR-PYCALL-0001");
}

#[test]
fn final_application_requires_selected_environment() {
    let Err(diagnostics) = apply_python_interop_metadata(project("math.sqrt"), None) else {
        panic!("final application declaration must select an environment");
    };
    assert_eq!(diagnostics[0].code, "SIFR-PYENV-0003");
}

#[test]
fn arrow_target_requires_exact_environment_certification() {
    let runtime = PackagePythonRuntime::for_tests(python(), "probe");
    let Err(diagnostics) =
        apply_python_interop_metadata(arrow_project("builtins.bytearray"), Some(&runtime))
    else {
        panic!("uncertified Arrow target must fail");
    };
    assert_eq!(diagnostics[0].code, "SIFR-PYZC-0001");

    let mut runtime = PackagePythonRuntime::for_tests(python(), "probe");
    let certification = arrow_certification("builtins.bytearray");
    assert!(
        runtime
            .set_arrow_certifications(vec![certification])
            .is_ok()
    );
    apply_python_interop_metadata(arrow_project("builtins.bytearray"), Some(&runtime))
        .expect("exactly certified Arrow target should pass");
}

#[test]
fn arrow_target_rejects_schema_mode_mismatch() {
    let mut runtime = PackagePythonRuntime::for_tests(python(), "probe");
    let mut certification = arrow_certification("builtins.bytearray");
    certification.schema_mode = sifr_package::ArrowCertifiedSchemaMode::Parameter;
    assert!(
        runtime
            .set_arrow_certifications(vec![certification])
            .is_ok()
    );

    let Err(diagnostics) =
        apply_python_interop_metadata(arrow_project("builtins.bytearray"), Some(&runtime))
    else {
        panic!("schema-mode mismatch must fail");
    };
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("schema-request mode"))
    );
}

#[test]
fn arrow_target_rejects_kind_mismatch() {
    let mut runtime = PackagePythonRuntime::for_tests(python(), "probe");
    let mut certification = arrow_certification("builtins.bytearray");
    certification.kind = sifr_package::ArrowCertifiedKind::Stream;
    assert!(
        runtime
            .set_arrow_certifications(vec![certification])
            .is_ok()
    );

    let Err(diagnostics) =
        apply_python_interop_metadata(arrow_project("builtins.bytearray"), Some(&runtime))
    else {
        panic!("kind mismatch must fail");
    };
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("return kind"))
    );
}

#[test]
fn dlpack_target_requires_exact_certification_and_call_shape() {
    let runtime = PackagePythonRuntime::for_tests(python(), "probe");
    let Err(diagnostics) =
        apply_python_interop_metadata(dlpack_project("math.sqrt"), Some(&runtime))
    else {
        panic!("DLPack target must require certification");
    };
    assert_eq!(diagnostics[0].code, "SIFR-PYZC-0001");

    let mut runtime = runtime;
    assert!(
        runtime
            .set_dlpack_certifications(vec![dlpack_certification("math.sqrt")])
            .is_ok()
    );
    apply_python_interop_metadata(dlpack_project("math.sqrt"), Some(&runtime))
        .expect("matching DLPack certification should pass");

    assert!(
        runtime
            .set_dlpack_certifications(vec![dlpack_certification("math.pow")])
            .is_ok()
    );
    let Err(diagnostics) =
        apply_python_interop_metadata(dlpack_project("math.pow"), Some(&runtime))
    else {
        panic!("DLPack target with incompatible arity must fail");
    };
    assert_eq!(diagnostics[0].code, "SIFR-PYCALL-0001");
}

fn dlpack_project(target: &str) -> GeneratedBinaryProject {
    let mut generated = project(target);
    let declaration = &mut generated.interop.python.declarations[0].declaration;
    declaration.kind = sifr_ir::PythonInteropDecoratorKind::Dlpack;
    declaration.dlpack = Some(sifr_ir::PythonDlpackDeclaration {
        device: sifr_ir::PythonDlpackDevice::Cpu,
        stream: sifr_ir::PythonDlpackStreamMode::None,
        element_type: Some(sifr_type_system::Type::Float),
    });
    generated
}

#[test]
fn record_expansion_rejects_uninspectable_target() {
    let runtime = PackagePythonRuntime::for_tests(python(), "probe");
    let mut generated = project("builtins.dir");
    generated.interop.python.target_probes[0].requires_inspectable_signature = true;
    let Err(diagnostics) = apply_python_interop_metadata(generated, Some(&runtime)) else {
        panic!("record expansion must require inspectability");
    };
    assert_eq!(diagnostics[0].code, "SIFR-PYCALL-0001");
}

#[test]
fn omittable_positional_parameters_require_keyword_capable_target_parameters() {
    let mut generated = project("pkg.collect");
    generated.interop.python.declarations[0]
        .declaration
        .parameters = vec![
        parameter("a", false),
        parameter("b", true),
        parameter("c", false),
    ];
    let compatible = vec![
        probe_parameter("a", "POSITIONAL_OR_KEYWORD", false),
        probe_parameter("b", "POSITIONAL_OR_KEYWORD", true),
        probe_parameter("c", "POSITIONAL_OR_KEYWORD", true),
    ];
    assert!(validate_signature(&generated.interop.python.declarations[0], &compatible).is_none());

    let positional_only = vec![
        probe_parameter("a", "POSITIONAL_OR_KEYWORD", false),
        probe_parameter("b", "POSITIONAL_ONLY", true),
        probe_parameter("c", "POSITIONAL_OR_KEYWORD", true),
    ];
    let reason = validate_signature(&generated.interop.python.declarations[0], &positional_only)
        .expect("positional-only omission target must be rejected");
    assert!(reason.contains("positional-only"));
}

fn parameter(name: &str, omit_when_absent: bool) -> sifr_ir::PythonInteropParameter {
    sifr_ir::PythonInteropParameter {
        name: name.to_string(),
        kind: sifr_ir::PythonParameterKind::Positional,
        has_default: omit_when_absent,
        omit_when_absent,
        span: ruff_text_size::TextRange::default(),
    }
}

fn probe_parameter(name: &str, kind: &str, has_default: bool) -> PythonTargetParameter {
    PythonTargetParameter {
        name: name.to_string(),
        kind: kind.to_string(),
        has_default,
    }
}

fn python() -> &'static str {
    if cfg!(windows) { "python" } else { "python3" }
}

fn project(target: &str) -> GeneratedBinaryProject {
    let root = target.split('.').next().unwrap_or_default().to_string();
    let declaration = sifr_ir::PythonInteropDeclaration {
        kind: sifr_ir::PythonInteropDecoratorKind::Function,
        target: Some(sifr_ir::PythonTargetPath {
            segments: target.split('.').map(str::to_string).collect(),
            span: ruff_text_size::TextRange::default(),
        }),
        span: ruff_text_size::TextRange::default(),
        effect: sifr_ir::PythonInteropEffect::BlockingIo,
        cleanup: None,
        consumes_receiver: false,
        parameters: if target == "math.sqrt" {
            vec![sifr_ir::PythonInteropParameter {
                name: "value".to_string(),
                kind: sifr_ir::PythonParameterKind::Positional,
                has_default: false,
                omit_when_absent: false,
                span: ruff_text_size::TextRange::default(),
            }]
        } else {
            Vec::new()
        },
        required_import_root: Some(root.clone()),
        callbacks: Vec::new(),
        buffer: None,
        arrow: None,
        dlpack: None,
    };
    let mut python = PythonInteropPlan::default();
    python.target_probes.push(PythonTargetProbe {
        import_root: Some(root),
        target_path: target.to_string(),
        requires_inspectable_signature: false,
        expects_type: false,
        status: PythonTargetProbeStatus::Planned,
    });
    python
        .declarations
        .push(sifr_codegen::PythonInteropPlanDeclaration {
            module_name: Some("main".to_string()),
            function_name: "target".to_string(),
            declaration,
            certification_target: Some(target.to_string()),
            parameter_types: if target == "math.sqrt" {
                vec![sifr_type_system::Type::Float]
            } else {
                Vec::new()
            },
            return_type: sifr_type_system::Type::None,
            callback_attachments: Vec::new(),
        });
    GeneratedBinaryProject {
        main_rs: "fn main() {}".to_string(),
        support_modules: BTreeMap::new(),
        used_stdlib_modules: HashSet::new(),
        required_features: HashSet::new(),
        interop: InteropBuildPlan {
            python,
            ..InteropBuildPlan::default()
        },
        cache_key_fragment: None,
        bridge_modules: Default::default(),
        python_runtime: None,
    }
}

fn arrow_project(target: &str) -> GeneratedBinaryProject {
    let mut generated = project(target);
    let declaration = &mut generated.interop.python.declarations[0].declaration;
    declaration.kind = sifr_ir::PythonInteropDecoratorKind::Arrow;
    declaration.arrow = Some(sifr_ir::PythonArrowDeclaration {
        kind: sifr_type_system::PythonArrowKind::Array,
        schema: sifr_ir::PythonArrowSchemaMode::Omitted,
    });
    declaration.dlpack = None;
    generated
}

fn arrow_certification(target: &str) -> sifr_package::ArrowCertification {
    sifr_package::ArrowCertification {
        target: target.to_string(),
        kind: sifr_package::ArrowCertifiedKind::Array,
        fixture: "fixtures/arrow.py".to_string(),
        fixture_digest: "digest".to_string(),
        producer_module: "pyarrow.lib".to_string(),
        producer_type: "Array".to_string(),
        distributions: vec![sifr_package::ArrowCertifiedDistribution {
            name: "pyarrow".to_string(),
            version: "1".to_string(),
        }],
        schema_mode: sifr_package::ArrowCertifiedSchemaMode::Omitted,
        identity_method: sifr_package::ArrowCertifiedIdentityMethod::BufferAddress,
        pointer_identity_verified: true,
        exact_release_count: 1,
        copy_performed: false,
    }
}

fn dlpack_certification(target: &str) -> sifr_package::DlpackCertification {
    sifr_package::DlpackCertification {
        target: target.to_string(),
        fixture: "fixtures/dlpack.py".to_string(),
        fixture_digest: "digest".to_string(),
        producer_module: "torch".to_string(),
        producer_type: "Tensor".to_string(),
        distributions: vec![sifr_package::ArrowCertifiedDistribution {
            name: "torch".to_string(),
            version: "1".to_string(),
        }],
        device: sifr_package::DlpackCertifiedDevice::Cpu,
        stream_policy: sifr_package::DlpackCertifiedStreamPolicy::None,
        pointer_identity_verified: true,
        exact_deleter_count: 1,
        copy_performed: false,
        within_run_assertions: true,
    }
}
