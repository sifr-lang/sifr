use super::python_interop::{PythonInteropPlanDiagnostic, scoped_diagnostic};
use super::python_runtime::PackagePythonRuntime;
use crate::diagnostics::diagnostic_with_code;
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use std::process::Command;

pub fn validate_certification_distributions(
    runtime: &PackagePythonRuntime,
    artifact: &sifr_package::PythonCertificationArtifact,
) -> Result<(), String> {
    let expected = artifact
        .arrow
        .iter()
        .flat_map(|certification| certification.distributions.iter())
        .chain(
            artifact
                .dlpack
                .iter()
                .flat_map(|certification| certification.distributions.iter()),
        )
        .collect::<std::collections::BTreeSet<_>>();
    validate_distribution_versions(expected, |distribution| {
        let output = Command::new(runtime.interpreter())
            .args([
                "-I",
                "-B",
                "-c",
                "import importlib.metadata,sys; print(importlib.metadata.version(sys.argv[1]))",
                &distribution.name,
            ])
            .output()
            .map_err(|error| {
                format!(
                    "could not inspect certified Python distribution '{}': {error}",
                    distribution.name
                )
            })?;
        if !output.status.success() {
            return Err(format!(
                "could not inspect certified Python distribution '{}'",
                distribution.name
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    })
}

pub fn validate_binding_distributions(
    runtime: &PackagePythonRuntime,
    artifact: &sifr_package::PythonBindingArtifact,
) -> Result<(), String> {
    for distribution in artifact
        .bindings
        .iter()
        .filter_map(|binding| binding.distribution.as_ref())
    {
        let output = Command::new(runtime.interpreter())
            .args([
                "-I",
                "-B",
                "-c",
                "import importlib.metadata,sys; print(importlib.metadata.version(sys.argv[1]))",
                &distribution.name,
            ])
            .output()
            .map_err(|error| {
                format!(
                    "could not inspect bound Python distribution '{}': {error}",
                    distribution.name
                )
            })?;
        if !output.status.success() {
            return Err(format!(
                "could not inspect bound Python distribution '{}'",
                distribution.name
            ));
        }
        let installed = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if installed != distribution.version {
            return Err(format!(
                "bound Python distribution '{}=={}' does not match the selected environment",
                distribution.name, distribution.version
            ));
        }
    }
    Ok(())
}

fn validate_distribution_versions<'a>(
    expected: impl IntoIterator<Item = &'a sifr_package::ArrowCertifiedDistribution>,
    mut installed_version: impl FnMut(
        &sifr_package::ArrowCertifiedDistribution,
    ) -> Result<String, String>,
) -> Result<(), String> {
    for distribution in expected {
        if installed_version(distribution)? != distribution.version {
            return Err(format!(
                "certified Python distribution '{}=={}' does not match the selected environment",
                distribution.name, distribution.version
            ));
        }
    }
    Ok(())
}

pub fn validate_protocol_certifications_for_plan(
    plan: &sifr_codegen::PythonInteropPlan,
    runtime: &PackagePythonRuntime,
) -> Vec<PythonInteropPlanDiagnostic> {
    let mut diagnostics = Vec::new();
    validate_arrow(plan, runtime, &mut diagnostics);
    validate_dlpack(plan, runtime, &mut diagnostics);
    diagnostics
}

fn validate_arrow(
    plan: &sifr_codegen::PythonInteropPlan,
    runtime: &PackagePythonRuntime,
    diagnostics: &mut Vec<PythonInteropPlanDiagnostic>,
) {
    for declaration in plan
        .declarations
        .iter()
        .filter(|item| item.declaration.kind == sifr_ir::PythonInteropDecoratorKind::Arrow)
    {
        let Some(runtime_target) = declaration.declaration.target.as_ref() else {
            continue;
        };
        let target = declaration
            .certification_target
            .clone()
            .unwrap_or_else(|| runtime_target.dotted());
        let target = logical_target(&target, &plan.bridge_packages);
        let Some(certification) = runtime.arrow_certification(&target) else {
            diagnostics.push(scoped_diagnostic(declaration, missing("Arrow", &target)));
            continue;
        };
        let requested = declaration.declaration.arrow.as_ref().is_some_and(|arrow| {
            matches!(
                arrow.schema,
                sifr_ir::PythonArrowSchemaMode::Parameter { .. }
            )
        });
        let certified_requested =
            certification.schema_mode == sifr_package::ArrowCertifiedSchemaMode::Parameter;
        if requested != certified_requested {
            diagnostics.push(scoped_diagnostic(
                declaration,
                invalid(format!(
                    "Arrow declaration target '{target}' schema-request mode does not match its executable certification"
                )),
            ));
        }
        if declaration
            .declaration
            .arrow
            .as_ref()
            .is_some_and(|arrow| !arrow_kind_matches(certification.kind, arrow.kind))
        {
            diagnostics.push(scoped_diagnostic(
                declaration,
                invalid(format!(
                    "Arrow declaration target '{target}' return kind does not match its executable certification"
                )),
            ));
        }
    }
}

fn validate_dlpack(
    plan: &sifr_codegen::PythonInteropPlan,
    runtime: &PackagePythonRuntime,
    diagnostics: &mut Vec<PythonInteropPlanDiagnostic>,
) {
    for declaration in plan.declarations.iter().filter(|item| {
        matches!(
            item.declaration.kind,
            sifr_ir::PythonInteropDecoratorKind::Dlpack
                | sifr_ir::PythonInteropDecoratorKind::DlpackStream
        )
    }) {
        let Some(runtime_target) = declaration.certification_target.as_deref() else {
            continue;
        };
        let target = logical_target(runtime_target, &plan.bridge_packages);
        let Some(certification) = runtime.dlpack_certification(&target) else {
            diagnostics.push(scoped_diagnostic(declaration, missing("DLPack", &target)));
            continue;
        };
        let Some(contract) = declaration.declaration.dlpack.as_ref() else {
            continue;
        };
        if !dlpack_device_matches(certification.device, contract.device) {
            diagnostics.push(scoped_diagnostic(
                declaration,
                invalid(format!(
                    "DLPack declaration target '{target}' device policy does not match its executable certification"
                )),
            ));
        }
        let parameter_stream = matches!(
            contract.stream,
            sifr_ir::PythonDlpackStreamMode::Parameter { .. }
        );
        let certified_parameter_stream =
            certification.stream_policy == sifr_package::DlpackCertifiedStreamPolicy::Parameter;
        if parameter_stream != certified_parameter_stream {
            diagnostics.push(scoped_diagnostic(
                declaration,
                invalid(format!(
                    "DLPack declaration target '{target}' stream policy does not match its executable certification"
                )),
            ));
        }
    }
}

fn missing(protocol: &str, target: &str) -> RenderedDiagnostic {
    invalid(format!(
        "{protocol} declaration target '{target}' has no exact executable no-copy certification for the selected Python environment"
    ))
}

fn invalid(message: String) -> RenderedDiagnostic {
    diagnostic_with_code(message, DiagnosticCode::PYZC_INVALID_DECLARATION)
}

fn logical_target(
    runtime_target: &str,
    bridge_packages: &[sifr_codegen::PythonBridgePackagePlan],
) -> String {
    for package in bridge_packages {
        if let Some(suffix) = runtime_target.strip_prefix(&package.runtime_package) {
            if suffix.starts_with('.') {
                return format!("bridge{suffix}");
            }
        }
    }
    runtime_target.to_string()
}

const fn arrow_kind_matches(
    certified: sifr_package::ArrowCertifiedKind,
    declared: sifr_type_system::PythonArrowKind,
) -> bool {
    matches!(
        (certified, declared),
        (
            sifr_package::ArrowCertifiedKind::Array,
            sifr_type_system::PythonArrowKind::Array
        ) | (
            sifr_package::ArrowCertifiedKind::Schema,
            sifr_type_system::PythonArrowKind::Schema
        ) | (
            sifr_package::ArrowCertifiedKind::Stream,
            sifr_type_system::PythonArrowKind::Stream
        ) | (
            sifr_package::ArrowCertifiedKind::DeviceArray,
            sifr_type_system::PythonArrowKind::DeviceArray
        ) | (
            sifr_package::ArrowCertifiedKind::DeviceStream,
            sifr_type_system::PythonArrowKind::DeviceStream
        )
    )
}

const fn dlpack_device_matches(
    certified: sifr_package::DlpackCertifiedDevice,
    declared: sifr_ir::PythonDlpackDevice,
) -> bool {
    matches!(
        (certified, declared),
        (
            sifr_package::DlpackCertifiedDevice::Cpu,
            sifr_ir::PythonDlpackDevice::Cpu
        ) | (
            sifr_package::DlpackCertifiedDevice::Cuda,
            sifr_ir::PythonDlpackDevice::Cuda
        ) | (
            sifr_package::DlpackCertifiedDevice::Any,
            sifr_ir::PythonDlpackDevice::Any
        )
    )
}

#[cfg(test)]
mod tests {
    use super::{logical_target, validate_distribution_versions};
    use sifr_package::ArrowCertifiedDistribution;

    #[test]
    fn bridge_certification_uses_stable_logical_target() {
        let packages = vec![sifr_codegen::PythonBridgePackagePlan {
            package_id: "demo".to_string(),
            resolved_package_key: "key".to_string(),
            runtime_package: "__sifr_bridge__.p_abc123".to_string(),
            inventory_digest: "digest".to_string(),
            modules: Vec::new(),
        }];
        assert_eq!(
            logical_target("__sifr_bridge__.p_abc123.tensor.make", &packages),
            "bridge.tensor.make"
        );
        assert_eq!(logical_target("torch.Tensor", &packages), "torch.Tensor");
    }

    #[test]
    fn certified_distribution_versions_fail_closed_on_drift_and_probe_failure() {
        let expected = [ArrowCertifiedDistribution {
            name: "pyarrow".to_string(),
            version: "25.0.1".to_string(),
        }];
        validate_distribution_versions(&expected, |_| Ok("25.0.1".to_string()))
            .expect("matching installed version should pass");
        assert!(
            validate_distribution_versions(&expected, |_| Ok("23.0.0".to_string()))
                .expect_err("version drift must fail")
                .contains("does not match")
        );
        assert_eq!(
            validate_distribution_versions(&expected, |_| Err("probe failed".to_string()))
                .expect_err("probe failure must fail"),
            "probe failed"
        );
    }
}
