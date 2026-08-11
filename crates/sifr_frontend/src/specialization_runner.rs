use crate::{
    decode_const_specialization_outcome, describe_type, verify_json_integer_boundary,
    ConstEvalError, ConstIssueTemplate, DeterministicConstEvaluator, JsonIntegerBoundaryDescriptor,
    JsonIntegerKind, JsonIntegerProfile, JsonIntegerRepresentation,
};
use ruff_text_size::TextRange;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};
use sifr_lowering::{
    ExternalDefs, HirDiagnostic, HirModule, LoweringResult, LoweringWarningDiagnostic,
    StaticSpecializationOutput,
};
use sifr_type_system::Type;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub(crate) fn run_specializations(
    module_name: &str,
    result: &mut LoweringResult,
    external_defs: &ExternalDefs,
) -> Result<(), Vec<HirDiagnostic>> {
    let mut errors = verify_integer_boundaries(module_name, result);
    for request in result.specialization_requests.clone() {
        let Some(class) = result
            .module
            .classes
            .iter()
            .find(|class| class.name == request.owner)
        else {
            errors.push(malformed(
                &request.package_module,
                "specialization_target",
                "specialization target is not a structural class",
                request.range,
            ));
            continue;
        };
        let target_type = Type::Class {
            identity: class.identity.clone(),
            type_args: class
                .type_params
                .iter()
                .cloned()
                .map(Type::TypeVar)
                .collect(),
            name: class.name.clone(),
            fields: class.fields.clone(),
            methods: Vec::new(),
            parent_class: class.parent_class.clone(),
        };
        if !class.type_params.is_empty() {
            errors.push(malformed(
                &request.package_module,
                "specialization_target",
                "@const_specialize requires a concrete class, not an unspecialized generic",
                request.range,
            ));
            continue;
        }
        let shape = describe_type(module_name, &target_type, result).to_const_value();
        let canonical_shape = crate::structural_shape::canonical_value(&shape);
        let Some(functions) = external_defs.const_functions.get(&request.package_module) else {
            errors.push(malformed(
                &request.package_module,
                "specialization_function",
                "specialization package exports no @const_eval functions",
                request.range,
            ));
            continue;
        };
        if !functions.contains_key(&request.function) {
            errors.push(malformed(
                &request.package_module,
                "specialization_function",
                "requested specialization function is not exported as @const_eval",
                request.range,
            ));
            continue;
        }
        let mut function_names = functions.keys().collect::<Vec<_>>();
        function_names.sort();
        let package_module = HirModule {
            functions: function_names
                .into_iter()
                .filter_map(|name| functions.get(name).cloned())
                .collect(),
            classes: Vec::new(),
            imports: Vec::new(),
            constants: Vec::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        };
        let evaluated = DeterministicConstEvaluator::new(&package_module)
            .evaluate_function(&request.function, vec![shape]);
        let evaluated = match evaluated {
            Ok(value) => value,
            Err(error) => {
                errors.push(evaluation_error(
                    &request.package_module,
                    &error,
                    request.range,
                ));
                continue;
            }
        };
        let outcome = match decode_const_specialization_outcome(evaluated, request.range) {
            Ok(outcome) => outcome,
            Err(mut diagnostics) => {
                for diagnostic in &mut diagnostics {
                    replace_unknown_package(diagnostic, &request.package_module);
                }
                errors.extend(diagnostics);
                continue;
            }
        };
        let templates = issue_templates(external_defs, &request.package_module, &request.function);
        match outcome.validate(&templates) {
            Err(diagnostics) => errors.extend(diagnostics),
            Ok(validated) => {
                if let Some(value) = &validated.value {
                    let canonical_value = crate::structural_shape::canonical_value(value);
                    let structural_contract_version = sifr_structural_identity::ALGORITHM_VERSION;
                    let program_identity = sifr_structural_identity::static_program_identity(
                        structural_contract_version,
                        [
                            ("module", module_name.as_bytes()),
                            ("owner", request.owner.as_bytes()),
                            ("package", request.package_module.as_bytes()),
                            ("function", request.function.as_bytes()),
                            ("shape", canonical_shape.as_bytes()),
                            ("value", canonical_value.as_bytes()),
                        ],
                    );
                    result
                        .specialization_outputs
                        .push(StaticSpecializationOutput {
                            owner: request.owner.clone(),
                            package_module: request.package_module.clone(),
                            function: request.function.clone(),
                            canonical_value,
                            program_identity: *program_identity.as_bytes(),
                            structural_contract_version,
                        });
                }
                for diagnostic in validated.diagnostics {
                    match diagnostic.code {
                        Some(DiagnosticCode::META_SPECIALIZATION_WARNING) => {
                            result
                                .warnings
                                .push(LoweringWarningDiagnostic::MetaPackageIssue {
                                    package: diagnostic_arg(&diagnostic, "package"),
                                    reason_code: diagnostic_arg(&diagnostic, "reason_code"),
                                    help: diagnostic.help,
                                    primary_range: diagnostic.primary_range,
                                });
                        }
                        Some(DiagnosticCode::META_SPECIALIZATION_FATAL) => errors.push(diagnostic),
                        _ => errors.push(malformed(
                            &request.package_module,
                            "diagnostic_mapping",
                            "specialization produced a non-metaprogramming diagnostic",
                            request.range,
                        )),
                    }
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn verify_integer_boundaries(module_name: &str, result: &LoweringResult) -> Vec<HirDiagnostic> {
    let mut errors = Vec::new();
    for request in &result.json_integer_boundary_requests {
        let Some(class) = result
            .module
            .classes
            .iter()
            .find(|class| class.name == request.owner)
        else {
            errors.push(malformed(
                "sifr.meta",
                "integer_boundary",
                "integer boundary owner is not a class",
                request.range,
            ));
            continue;
        };
        let Some((_, field_type)) = class.fields.iter().find(|(name, _)| name == &request.field)
        else {
            errors.push(malformed(
                "sifr.meta",
                "integer_boundary",
                "integer boundary field does not exist",
                request.range,
            ));
            continue;
        };
        let integer_kind = match field_type.resolve_alias() {
            Type::Int => JsonIntegerKind::Exact,
            Type::FixedInt(kind) => JsonIntegerKind::Fixed(*kind),
            _ => {
                errors.push(malformed(
                    "sifr.meta",
                    "integer_boundary",
                    "integer boundary field is not an integer type",
                    request.range,
                ));
                continue;
            }
        };
        let profile = match request.profile.as_deref() {
            None => None,
            Some("exact") => Some(JsonIntegerProfile::Exact),
            Some("web") => Some(JsonIntegerProfile::Web),
            Some("string_ints") => Some(JsonIntegerProfile::StringInts),
            Some(_) => {
                errors.push(malformed(
                    "sifr.meta",
                    "integer_boundary",
                    "integer boundary profile must be exact, web, string_ints, or None",
                    request.range,
                ));
                continue;
            }
        };
        let representation = match request.representation.as_str() {
            "default" => JsonIntegerRepresentation::ProfileDefault,
            "number" => JsonIntegerRepresentation::Number,
            "decimal_string" => JsonIntegerRepresentation::DecimalString,
            _ => {
                errors.push(malformed(
                    "sifr.meta",
                    "integer_boundary",
                    "integer boundary representation must be default, number, or decimal_string",
                    request.range,
                ));
                continue;
            }
        };
        let static_range = match (&request.static_minimum, &request.static_maximum) {
            (Some(minimum), Some(maximum)) => Some((minimum.clone(), maximum.clone())),
            (None, None) => None,
            _ => {
                errors.push(malformed(
                    "sifr.meta",
                    "integer_boundary",
                    "integer boundary static range must provide both minimum and maximum",
                    request.range,
                ));
                continue;
            }
        };
        let descriptor = JsonIntegerBoundaryDescriptor {
            profile,
            integer_kind,
            static_range,
            representation,
            source_path: format!("{module_name}.{}.{}", request.owner, request.field),
            source_span: Some(request.range),
        };
        if let Err(diagnostic) = verify_json_integer_boundary(&descriptor) {
            errors.push(*diagnostic);
        }
    }
    errors
}

fn issue_templates(
    external_defs: &ExternalDefs,
    package: &str,
    function: &str,
) -> Vec<ConstIssueTemplate> {
    external_defs
        .declaration_metadata
        .get(package)
        .into_iter()
        .flatten()
        .filter(|metadata| metadata.owner == function && metadata.key == "sifr.meta.issue_template")
        .filter_map(|metadata| {
            let value = crate::structural_shape::const_value_from_hir(&metadata.value)?;
            let crate::ConstValue::Tuple(mut parts) = value else {
                return None;
            };
            if parts.len() != 2 {
                return None;
            }
            let crate::ConstValue::List(arguments) = parts.pop()? else {
                return None;
            };
            let crate::ConstValue::String(reason_code) = parts.pop()? else {
                return None;
            };
            let argument_names = arguments
                .into_iter()
                .map(|argument| match argument {
                    crate::ConstValue::String(argument) => Some(argument),
                    _ => None,
                })
                .collect::<Option<BTreeSet<_>>>()?;
            Some(ConstIssueTemplate {
                package: package.to_string(),
                reason_code,
                argument_names,
            })
        })
        .collect()
}

fn evaluation_error(package: &str, error: &ConstEvalError, range: TextRange) -> HirDiagnostic {
    malformed(
        package,
        "const_evaluation",
        format!("{:?}: {}", error.kind, error.detail),
        range,
    )
}

fn malformed(
    package: &str,
    reason_code: &str,
    problem: impl Into<String>,
    range: TextRange,
) -> HirDiagnostic {
    let problem = problem.into();
    HirDiagnostic {
        code: Some(DiagnosticCode::META_MALFORMED_DECLARATION),
        message: format!(
            "package {package} declared malformed specialization issue {reason_code}: {problem}"
        ),
        args: BTreeMap::from([
            (
                "package".to_string(),
                DiagnosticArg::String(package.to_string()),
            ),
            (
                "reason_code".to_string(),
                DiagnosticArg::String(reason_code.to_string()),
            ),
            (
                "declaration_problem".to_string(),
                DiagnosticArg::String(problem),
            ),
        ]),
        help: None,
        primary_range: Some(range),
        line: None,
        col: None,
    }
}

fn diagnostic_arg(diagnostic: &HirDiagnostic, name: &str) -> String {
    match diagnostic.args.get(name) {
        Some(DiagnosticArg::String(value)) => value.clone(),
        _ => "unknown".to_string(),
    }
}

fn replace_unknown_package(diagnostic: &mut HirDiagnostic, package: &str) {
    if matches!(diagnostic.args.get("package"), Some(DiagnosticArg::String(value)) if value == "unknown")
    {
        diagnostic.args.insert(
            "package".to_string(),
            DiagnosticArg::String(package.to_string()),
        );
        diagnostic.message =
            diagnostic
                .message
                .replacen("package unknown", &format!("package {package}"), 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        collect_module_exports, compile_module_hir, warning_diagnostics, FrontendDiagnosticStyle,
        FrontendSourceContext,
    };
    use sifr_syntax::parse_module_suite;

    fn compile(
        module: &str,
        source: &str,
        external_defs: &ExternalDefs,
    ) -> Result<LoweringResult, Vec<sifr_diagnostics::RenderedDiagnostic>> {
        let parsed = parse_module_suite(source, None).expect("fixture parses");
        compile_module_hir(
            module,
            &parsed,
            external_defs,
            FrontendDiagnosticStyle::Bare,
        )
    }

    fn errors(
        result: Result<LoweringResult, Vec<sifr_diagnostics::RenderedDiagnostic>>,
    ) -> Vec<sifr_diagnostics::RenderedDiagnostic> {
        match result {
            Ok(_) => panic!("compilation unexpectedly succeeded"),
            Err(errors) => errors,
        }
    }

    fn package_source(severity: &str, reason: &str, declare_template: bool) -> String {
        let template = if declare_template {
            format!("@metadata(\"sifr.meta.issue_template\", (\"{reason}\", [\"field\"]))\n")
        } else {
            String::new()
        };
        format!(
            r#"
class IssueArgs:
    field: str

class Issue:
    package: str
    reason_code: str
    severity: str
    arguments: IssueArgs
    notes: list[str]

class Outcome:
    status: str
    value: str | None
    issues: list[Issue]

@const_eval
{template}def describe(shape: dict[str, str]) -> Outcome:
    issue: Issue = Issue("fixture.meta", "{reason}", "{severity}", IssueArgs("value"), ["package note"])
    issues: list[Issue] = [issue]
    if "{severity}" == "warning":
        identity: str | None = shape["canonical_identity"]
        if identity is not None:
            return Outcome("produced", identity, issues)
        return Outcome("failed", None, issues)
    return Outcome("failed", None, issues)
"#
        )
    }

    const TARGET: &str = r#"
from fixture.meta import describe

@const_specialize("fixture.meta", "describe")
class Model:
    value: int
"#;

    #[test]
    fn package_warning_flows_through_frontend_warning_channel() {
        let mut external_defs = ExternalDefs::default();
        let package = compile(
            "fixture.meta",
            &package_source("warning", "shape_notice", true),
            &external_defs,
        )
        .expect("package compiles");
        collect_module_exports("fixture.meta", &package, &mut external_defs);

        let target = compile("target", TARGET, &external_defs).expect("target specializes");
        assert!(matches!(
            target.warnings.last(),
            Some(LoweringWarningDiagnostic::MetaPackageIssue {
                package,
                reason_code,
                ..
            }) if package == "fixture.meta" && reason_code == "shape_notice"
        ));
        assert_eq!(target.specialization_outputs.len(), 1);
        assert_eq!(target.specialization_outputs[0].owner, "Model");
        assert_eq!(
            target.specialization_outputs[0].package_module,
            "fixture.meta"
        );
        assert!(target.specialization_outputs[0]
            .canonical_value
            .contains("target.Model"));
        assert_ne!(target.specialization_outputs[0].program_identity, [0; 32]);
        assert_eq!(
            target.specialization_outputs[0].structural_contract_version,
            sifr_structural_identity::ALGORITHM_VERSION
        );
        let unrelated = compile(
            "target",
            &format!("{TARGET}\n\ndef unrelated() -> int:\n    return 9\n"),
            &external_defs,
        )
        .expect("unrelated declaration compiles");
        assert_eq!(
            target.specialization_outputs[0].program_identity,
            unrelated.specialization_outputs[0].program_identity
        );
        let changed = compile(
            "target",
            &TARGET.replace("value: int", "value: str"),
            &external_defs,
        )
        .expect("changed shape specializes");
        assert_ne!(
            target.specialization_outputs[0].program_identity,
            changed.specialization_outputs[0].program_identity
        );
        let cli = warning_diagnostics(None, &target.warnings);
        let editor = warning_diagnostics(
            Some(FrontendSourceContext {
                display_path: "target.sifr",
                source: TARGET,
            }),
            &target.warnings,
        );
        assert_eq!(cli[0].code, editor[0].code);
        assert_eq!(cli[0].severity, editor[0].severity);
        assert_eq!(cli[0].args, editor[0].args);
        assert_eq!(cli[0].url, editor[0].url);
        assert_eq!(cli[0].message_template, editor[0].message_template);
    }

    #[test]
    fn package_fatal_flows_through_registry_owned_frontend_error() {
        let mut external_defs = ExternalDefs::default();
        let package = compile(
            "fixture.meta",
            &package_source("fatal", "shape_rejected", true),
            &external_defs,
        )
        .expect("package compiles");
        collect_module_exports("fixture.meta", &package, &mut external_defs);

        let diagnostics = errors(compile("target", TARGET, &external_defs));
        assert_eq!(diagnostics[0].code, "SIFR-META-0001");
        assert_eq!(
            diagnostics[0].args["package"],
            DiagnosticArg::String("fixture.meta".to_string())
        );
    }

    #[test]
    fn undeclared_package_issue_fails_closed_as_malformed() {
        let mut external_defs = ExternalDefs::default();
        let package = compile(
            "fixture.meta",
            &package_source("fatal", "undeclared", false),
            &external_defs,
        )
        .expect("package compiles");
        collect_module_exports("fixture.meta", &package, &mut external_defs);

        let diagnostics = errors(compile("target", TARGET, &external_defs));
        assert_eq!(diagnostics[0].code, "SIFR-META-0003");
    }

    #[test]
    fn non_package_integer_boundary_fixture_fails_closed_for_missing_or_unsafe_policy() {
        let missing = errors(compile(
            "fixture.boundaries",
            r#"
@json_integer_boundary("count", None, "default", None, None)
class Counter:
    count: int
"#,
            &ExternalDefs::default(),
        ));
        assert_eq!(missing[0].code, "SIFR-INT-0009");
        assert_eq!(
            missing[0].args["path"],
            DiagnosticArg::String("fixture.boundaries.Counter.count".to_string())
        );

        let unsafe_web = errors(compile(
            "fixture.boundaries",
            r#"
@json_integer_boundary("count", "web", "number", None, None)
class Counter:
    count: int
"#,
            &ExternalDefs::default(),
        ));
        assert_eq!(unsafe_web[0].code, "SIFR-INT-0009");

        let safe = compile(
            "fixture.boundaries",
            r#"
@json_integer_boundary("count", "web", "number", -2147483648, 2147483647)
class Counter:
    count: int32
"#,
            &ExternalDefs::default(),
        );
        assert!(safe.is_ok());
    }
}
