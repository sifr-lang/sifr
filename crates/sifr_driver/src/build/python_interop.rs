use super::project_codegen::GeneratedBinaryProject;
use super::python_runtime::PackagePythonRuntime;
use crate::diagnostics::diagnostic_with_code;
use serde::Deserialize;
use sifr_codegen::PythonTargetProbeStatus;
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
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
print(json.dumps({"ok": True, "callable": is_callable, "inspectable": inspectable, "parameters": parameters, "error": None}))
"#;

#[derive(Deserialize)]
struct ProbeOutput {
    ok: bool,
    callable: bool,
    inspectable: bool,
    #[serde(default)]
    parameters: Vec<ProbeParameter>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct ProbeParameter {
    name: String,
    kind: String,
    has_default: bool,
}

pub(super) fn apply_python_interop_metadata(
    mut generated: GeneratedBinaryProject,
    runtime: Option<&PackagePythonRuntime>,
) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
    if generated.interop.python.declarations.is_empty() {
        return Ok(generated);
    }
    let Some(runtime) = runtime else {
        return Err(vec![diagnostic_with_code(
            "Python declarations require a root-selected Python environment",
            DiagnosticCode::PYENV_MISSING_SELECTION,
        )]);
    };

    let mut diagnostics = Vec::new();
    for probe in &mut generated.interop.python.target_probes {
        match execute_probe(runtime, &probe.target_path) {
            Ok(output) if !output.ok => diagnostics.push(diagnostic_with_code(
                format!(
                    "invalid Python declaration target '{}': {}",
                    probe.target_path,
                    output
                        .error
                        .unwrap_or_else(|| "target is unresolved".to_string())
                ),
                DiagnosticCode::PYIMP_INVALID_TARGET,
            )),
            Ok(output) if !output.callable => diagnostics.push(diagnostic_with_code(
                format!(
                    "invalid Python declaration call shape: target '{}' is not callable",
                    probe.target_path
                ),
                DiagnosticCode::PYCALL_INVALID_SHAPE,
            )),
            Ok(output) if !output.inspectable && probe.requires_inspectable_signature => {
                diagnostics.push(diagnostic_with_code(
                    format!(
                        "invalid Python declaration call shape: target '{}' must be inspectable for `**record` expansion",
                        probe.target_path
                    ),
                    DiagnosticCode::PYCALL_INVALID_SHAPE,
                ));
            }
            Ok(output) => {
                if output.inspectable {
                    if let Some(reason) = generated
                        .interop
                        .python
                        .declarations
                        .iter()
                        .filter(|declaration| {
                            declaration
                                .declaration
                                .target
                                .as_ref()
                                .is_some_and(|target| target.dotted() == probe.target_path)
                        })
                        .find_map(|declaration| validate_signature(declaration, &output.parameters))
                    {
                        diagnostics.push(diagnostic_with_code(
                            format!(
                                "invalid Python declaration call shape for '{}': {reason}",
                                probe.target_path
                            ),
                            DiagnosticCode::PYCALL_INVALID_SHAPE,
                        ));
                        continue;
                    }
                }
                probe.status = if output.inspectable {
                    PythonTargetProbeStatus::Verified
                } else {
                    PythonTargetProbeStatus::RuntimeChecked
                };
            }
            Err(reason) => diagnostics.push(diagnostic_with_code(
                format!(
                    "invalid Python declaration target '{}': probe failed: {reason}",
                    probe.target_path
                ),
                DiagnosticCode::PYIMP_INVALID_TARGET,
            )),
        }
    }
    if diagnostics.is_empty() {
        Ok(generated)
    } else {
        Err(diagnostics)
    }
}

fn validate_signature(
    declaration: &sifr_codegen::PythonInteropPlanDeclaration,
    target: &[ProbeParameter],
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

fn execute_probe(runtime: &PackagePythonRuntime, target: &str) -> Result<ProbeOutput, String> {
    let output = Command::new(runtime.interpreter())
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

    fn probe_parameter(name: &str, kind: &str, has_default: bool) -> ProbeParameter {
        ProbeParameter {
            name: name.to_string(),
            kind: kind.to_string(),
            has_default,
        }
    }

    fn python() -> &'static str {
        if cfg!(windows) {
            "python"
        } else {
            "python3"
        }
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
        };
        let mut python = PythonInteropPlan::default();
        python.target_probes.push(PythonTargetProbe {
            import_root: Some(root),
            target_path: target.to_string(),
            requires_inspectable_signature: false,
            status: PythonTargetProbeStatus::Planned,
        });
        python
            .declarations
            .push(sifr_codegen::PythonInteropPlanDeclaration {
                module_name: Some("main".to_string()),
                function_name: "target".to_string(),
                declaration,
                parameter_types: if target == "math.sqrt" {
                    vec![sifr_type_system::Type::Float]
                } else {
                    Vec::new()
                },
                return_type: sifr_type_system::Type::None,
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
            python_runtime: None,
        }
    }
}
