use super::project_codegen::GeneratedBinaryProject;
use super::python_runtime::PackagePythonRuntime;
use crate::diagnostics::diagnostic_with_code;
use ruff_text_size::TextRange;
use serde::Deserialize;
use sifr_codegen::PythonTargetProbeStatus;
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use std::path::Path;
use std::process::Command;

const TARGET_PROBE: &str = r#"
import importlib, inspect, json, sys

parts = sys.argv[1].split('.')
value = None
error = None
for split in range(len(parts) - 1, 0, -1):
    try:
        value = importlib.import_module('.'.join(parts[:split]))
    except (ImportError, ModuleNotFoundError):
        continue
    try:
        for segment in parts[split:]:
            value = getattr(value, segment)
    except Exception as exc:
        error = f"{type(exc).__name__}: {exc}"
        value = None
    break

if value is None:
    print(json.dumps({"ok": False, "callable": False, "inspectable": False, "error": error or "target module could not be imported"}))
    raise SystemExit(0)

is_callable = callable(value)
inspectable = False
parameters = []
if is_callable:
    try:
        signature = inspect.signature(value)
        inspectable = True
        parameters = [
            {"name": parameter.name, "kind": parameter.kind.name, "has_default": parameter.default is not inspect.Parameter.empty}
            for parameter in signature.parameters.values()
        ]
    except (TypeError, ValueError):
        pass
print(json.dumps({"ok": True, "callable": is_callable, "is_type": isinstance(value, type), "inspectable": inspectable, "parameters": parameters, "error": None}))
"#;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct PythonTargetInspection {
    pub ok: bool,
    pub callable: bool,
    #[serde(default)]
    pub is_type: bool,
    pub inspectable: bool,
    #[serde(default)]
    pub parameters: Vec<PythonTargetParameter>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct PythonTargetParameter {
    pub name: String,
    pub kind: String,
    pub has_default: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PythonInteropPlanDiagnostic {
    pub module_name: Option<String>,
    pub span: TextRange,
    pub diagnostic: RenderedDiagnostic,
}

pub(super) fn apply_python_interop_metadata(
    generated: GeneratedBinaryProject,
    runtime: Option<&PackagePythonRuntime>,
) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
    apply_python_interop_metadata_with_policy(generated, runtime, false)
}

pub(super) fn apply_python_interop_metadata_for_check(
    generated: GeneratedBinaryProject,
    runtime: Option<&PackagePythonRuntime>,
) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
    apply_python_interop_metadata_with_policy(generated, runtime, true)
}

fn apply_python_interop_metadata_with_policy(
    mut generated: GeneratedBinaryProject,
    runtime: Option<&PackagePythonRuntime>,
    allow_deferred: bool,
) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
    if generated.interop.python.declarations.is_empty() {
        return Ok(generated);
    }
    let Some(runtime) = runtime else {
        mark_embedded_bridge_targets(&mut generated.interop.python);
        if allow_deferred {
            return Ok(generated);
        }
        return Err(vec![diagnostic_with_code(
            "Python declarations require a root-selected Python environment",
            DiagnosticCode::PYENV_MISSING_SELECTION,
        )]);
    };

    let mut diagnostics = super::python_certification::validate_protocol_certifications_for_plan(
        &generated.interop.python,
        runtime,
    )
    .into_iter()
    .map(|diagnostic| diagnostic.diagnostic)
    .collect::<Vec<_>>();
    diagnostics.extend(probe_python_interop_plan(
        &mut generated.interop.python,
        runtime.interpreter(),
    ));
    if diagnostics.is_empty() {
        Ok(generated)
    } else {
        Err(diagnostics)
    }
}

/// Apply the compiler's exact declaration target and call-shape probe to an
/// already lowered Python interop plan. Tooling uses this query so editor
/// status never invents a second interpretation of declaration compatibility.
pub fn probe_python_interop_plan(
    plan: &mut sifr_codegen::PythonInteropPlan,
    interpreter: &Path,
) -> Vec<RenderedDiagnostic> {
    mark_embedded_bridge_targets(plan);
    let mut seen_targets = std::collections::BTreeSet::new();
    let targets = plan
        .target_probes
        .iter()
        .filter(|probe| !probe.target_path.starts_with("__sifr_bridge__."))
        .filter(|probe| seen_targets.insert(probe.target_path.clone()))
        .map(|probe| probe.target_path.clone())
        .collect::<Vec<_>>();
    targets
        .into_iter()
        .flat_map(|target| {
            let inspection = inspect_python_target(interpreter, &target);
            apply_python_target_inspection(
                plan,
                &target,
                inspection.as_ref().map_err(String::as_str),
            )
        })
        .map(|diagnostic| diagnostic.diagnostic)
        .collect()
}

pub fn inspect_python_target(
    interpreter: &Path,
    target: &str,
) -> Result<PythonTargetInspection, String> {
    execute_probe(interpreter, target)
}

pub fn apply_python_target_inspection(
    plan: &mut sifr_codegen::PythonInteropPlan,
    target: &str,
    inspection: Result<&PythonTargetInspection, &str>,
) -> Vec<PythonInteropPlanDiagnostic> {
    let probe_indices = plan
        .target_probes
        .iter()
        .enumerate()
        .filter_map(|(index, probe)| (probe.target_path == target).then_some(index))
        .collect::<Vec<_>>();
    if probe_indices.is_empty() {
        return Vec::new();
    }
    if target.starts_with("__sifr_bridge__.") {
        for index in probe_indices {
            plan.target_probes[index].status = PythonTargetProbeStatus::RuntimeChecked;
        }
        return Vec::new();
    }
    let expects_type = probe_indices
        .iter()
        .any(|index| plan.target_probes[*index].expects_type);
    let requires_inspectable_signature = probe_indices
        .iter()
        .any(|index| plan.target_probes[*index].requires_inspectable_signature);
    let diagnostic = match inspection {
        Ok(output) if !output.ok => Some(diagnostic_with_code(
            format!(
                "invalid Python declaration target '{target}': {}",
                output
                    .error
                    .clone()
                    .unwrap_or_else(|| "target is unresolved".to_string())
            ),
            DiagnosticCode::PYIMP_INVALID_TARGET,
        )),
        Ok(output) if !output.callable => Some(diagnostic_with_code(
            format!("invalid Python declaration call shape: target '{target}' is not callable"),
            DiagnosticCode::PYCALL_INVALID_SHAPE,
        )),
        Ok(output) if expects_type && !output.is_type => Some(diagnostic_with_code(
            format!(
                "invalid Python declaration target '{target}': a declaration requires this target to be a Python type"
            ),
            DiagnosticCode::PYCALL_INVALID_SHAPE,
        )),
        Ok(output) if !output.inspectable && requires_inspectable_signature => {
            Some(diagnostic_with_code(
                format!(
                    "invalid Python declaration call shape: a declaration requires target '{target}' to be inspectable for `**record` expansion"
                ),
                DiagnosticCode::PYCALL_INVALID_SHAPE,
            ))
        }
        Err(reason) => Some(diagnostic_with_code(
            format!("invalid Python declaration target '{target}': probe failed: {reason}"),
            DiagnosticCode::PYIMP_INVALID_TARGET,
        )),
        Ok(_) => None,
    };
    if let Some(diagnostic) = diagnostic {
        return scoped_target_diagnostics(plan, target, diagnostic);
    }
    let Ok(output) = inspection else {
        return Vec::new();
    };
    if output.inspectable {
        let diagnostics = plan
            .declarations
            .iter()
            .filter(|declaration| {
                declaration_targets(declaration, target)
                    && declaration_requires_signature_validation(declaration)
            })
            .filter_map(|declaration| {
                validate_signature(declaration, &output.parameters).map(|reason| {
                    scoped_diagnostic(
                        declaration,
                        diagnostic_with_code(
                            format!(
                                "invalid Python declaration call shape for '{target}': {reason}"
                            ),
                            DiagnosticCode::PYCALL_INVALID_SHAPE,
                        ),
                    )
                })
            })
            .collect::<Vec<_>>();
        if !diagnostics.is_empty() {
            return diagnostics;
        }
    }
    let status = if output.inspectable {
        PythonTargetProbeStatus::Verified
    } else {
        PythonTargetProbeStatus::RuntimeChecked
    };
    for index in probe_indices {
        plan.target_probes[index].status = status;
    }
    Vec::new()
}

fn declaration_requires_signature_validation(
    declaration: &sifr_codegen::PythonInteropPlanDeclaration,
) -> bool {
    matches!(
        declaration.declaration.kind,
        sifr_ir::PythonInteropDecoratorKind::Function
            | sifr_ir::PythonInteropDecoratorKind::Buffer
            | sifr_ir::PythonInteropDecoratorKind::Arrow
            | sifr_ir::PythonInteropDecoratorKind::Dlpack
            | sifr_ir::PythonInteropDecoratorKind::DlpackStream
    )
}

fn declaration_targets(
    declaration: &sifr_codegen::PythonInteropPlanDeclaration,
    target: &str,
) -> bool {
    declaration
        .declaration
        .target
        .as_ref()
        .is_some_and(|candidate| candidate.dotted() == target)
}

fn scoped_target_diagnostics(
    plan: &sifr_codegen::PythonInteropPlan,
    target: &str,
    diagnostic: RenderedDiagnostic,
) -> Vec<PythonInteropPlanDiagnostic> {
    let mut diagnostics = plan
        .declarations
        .iter()
        .filter(|declaration| declaration_targets(declaration, target))
        .map(|declaration| scoped_diagnostic(declaration, diagnostic.clone()))
        .collect::<Vec<_>>();
    if diagnostics.is_empty() {
        diagnostics.push(PythonInteropPlanDiagnostic {
            module_name: None,
            span: TextRange::default(),
            diagnostic,
        });
    }
    diagnostics
}

pub(crate) fn scoped_diagnostic(
    declaration: &sifr_codegen::PythonInteropPlanDeclaration,
    diagnostic: RenderedDiagnostic,
) -> PythonInteropPlanDiagnostic {
    PythonInteropPlanDiagnostic {
        module_name: declaration.module_name.clone(),
        span: declaration.declaration.span,
        diagnostic,
    }
}

fn mark_embedded_bridge_targets(plan: &mut sifr_codegen::PythonInteropPlan) {
    for probe in &mut plan.target_probes {
        if probe.target_path.starts_with("__sifr_bridge__.") {
            probe.status = PythonTargetProbeStatus::RuntimeChecked;
        }
    }
}

fn validate_signature(
    declaration: &sifr_codegen::PythonInteropPlanDeclaration,
    target: &[PythonTargetParameter],
) -> Option<String> {
    use sifr_ir::PythonParameterKind;

    let positional = declaration
        .declaration
        .parameters
        .iter()
        .filter(|parameter| parameter.kind == PythonParameterKind::Positional)
        .take_while(|parameter| !parameter.omit_when_absent)
        .count();
    let target_positional = target
        .iter()
        .filter(|parameter| {
            matches!(
                parameter.kind.as_str(),
                "POSITIONAL_ONLY" | "POSITIONAL_OR_KEYWORD"
            )
        })
        .collect::<Vec<_>>();
    let has_varargs = target
        .iter()
        .any(|parameter| parameter.kind == "VAR_POSITIONAL");
    let has_kwargs = target
        .iter()
        .any(|parameter| parameter.kind == "VAR_KEYWORD");
    if positional > target_positional.len() && !has_varargs {
        return Some(format!(
            "declaration passes {positional} positional arguments but target accepts {}",
            target_positional.len()
        ));
    }

    let mut forward_positional_by_name = false;
    for parameter in &declaration.declaration.parameters {
        if parameter.kind == PythonParameterKind::Positional && parameter.omit_when_absent {
            forward_positional_by_name = true;
        }
        match parameter.kind {
            PythonParameterKind::PositionalVariadic => {
                if forward_positional_by_name {
                    return Some(
                        "typed `*args` cannot follow an omittable positional parameter".to_string(),
                    );
                }
                if !has_varargs {
                    return Some("typed `*args` requires a target `*args` parameter".to_string());
                }
            }
            PythonParameterKind::KeywordVariadic if !has_kwargs => {
                return Some("typed `**kwargs` requires a target `**kwargs` parameter".to_string());
            }
            PythonParameterKind::KeywordOnly => {
                let matching = target
                    .iter()
                    .find(|candidate| candidate.name == parameter.name);
                if matching.is_none() && !has_kwargs {
                    return Some(format!(
                        "keyword-only parameter '{}' is not accepted by the target",
                        parameter.name
                    ));
                }
                if matching.is_some_and(|candidate| candidate.kind == "POSITIONAL_ONLY") {
                    return Some(format!(
                        "target parameter '{}' is positional-only",
                        parameter.name
                    ));
                }
                if target_positional
                    .iter()
                    .take(positional)
                    .any(|candidate| candidate.name == parameter.name)
                {
                    return Some(format!(
                        "keyword '{}' duplicates a positionally supplied target parameter",
                        parameter.name
                    ));
                }
            }
            PythonParameterKind::Positional if forward_positional_by_name => {
                let matching = target
                    .iter()
                    .find(|candidate| candidate.name == parameter.name);
                if matching.is_none() && !has_kwargs {
                    return Some(format!(
                        "omittable positional parameter '{}' is not accepted by name by the target",
                        parameter.name
                    ));
                }
                if matching.is_some_and(|candidate| candidate.kind == "POSITIONAL_ONLY") {
                    return Some(format!(
                        "omittable positional parameter '{}' maps to a positional-only target parameter",
                        parameter.name
                    ));
                }
            }
            PythonParameterKind::Positional | PythonParameterKind::KeywordVariadic => {}
        }
    }

    for (index, parameter) in target_positional.iter().enumerate() {
        if index >= positional && !parameter.has_default {
            return Some(format!(
                "target requires positional parameter '{}' not supplied by the declaration",
                parameter.name
            ));
        }
    }
    for parameter in target
        .iter()
        .filter(|parameter| parameter.kind == "KEYWORD_ONLY")
    {
        let supplied = declaration.declaration.parameters.iter().any(|candidate| {
            candidate.kind == PythonParameterKind::KeywordOnly
                && candidate.name == parameter.name
                && !candidate.omit_when_absent
        });
        if !parameter.has_default && !supplied {
            return Some(format!(
                "target requires keyword-only parameter '{}' not always supplied by the declaration",
                parameter.name
            ));
        }
    }
    None
}

fn execute_probe(interpreter: &Path, target: &str) -> Result<PythonTargetInspection, String> {
    let output = Command::new(interpreter)
        .arg("-B")
        .arg("-I")
        .arg("-c")
        .arg(TARGET_PROBE)
        .arg(target)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
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
        let Err(diagnostics) = apply_python_interop_metadata(project("math.pi"), Some(&runtime))
        else {
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
        runtime.set_arrow_certifications(vec![arrow_certification("builtins.bytearray")]);
        apply_python_interop_metadata(arrow_project("builtins.bytearray"), Some(&runtime))
            .expect("exactly certified Arrow target should pass");
    }

    #[test]
    fn arrow_target_rejects_schema_mode_mismatch() {
        let mut runtime = PackagePythonRuntime::for_tests(python(), "probe");
        let mut certification = arrow_certification("builtins.bytearray");
        certification.schema_mode = sifr_package::ArrowCertifiedSchemaMode::Parameter;
        runtime.set_arrow_certifications(vec![certification]);

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
        runtime.set_arrow_certifications(vec![certification]);

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
        runtime.set_dlpack_certifications(vec![dlpack_certification("math.sqrt")]);
        apply_python_interop_metadata(dlpack_project("math.sqrt"), Some(&runtime))
            .expect("matching DLPack certification should pass");

        runtime.set_dlpack_certifications(vec![dlpack_certification("math.pow")]);
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
        assert!(
            validate_signature(&generated.interop.python.declarations[0], &compatible).is_none()
        );

        let positional_only = vec![
            probe_parameter("a", "POSITIONAL_OR_KEYWORD", false),
            probe_parameter("b", "POSITIONAL_ONLY", true),
            probe_parameter("c", "POSITIONAL_OR_KEYWORD", true),
        ];
        let reason =
            validate_signature(&generated.interop.python.declarations[0], &positional_only)
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
}
