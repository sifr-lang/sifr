use crate::package_issues::{
    SpecializationDiagnostic, SpecializationDiagnostics, evaluation_error, issue_templates,
    replace_unknown_package,
};
use crate::specialization_support::{malformed, method_slot_diagnostic, static_program_value};
use crate::{
    ConstIssueSeverity, DeterministicConstEvaluator, JsonIntegerBoundaryDescriptor,
    JsonIntegerKind, JsonIntegerProfile, JsonIntegerRepresentation,
    decode_const_specialization_outcome, describe_type_with_externals, package_note,
    verify_json_integer_boundary,
};
use sifr_lowering::{
    ExternalDefs, HirDiagnostic, HirModule, LoweringResult, LoweringWarningDiagnostic,
    StaticSpecializationOutput,
};
use sifr_type_system::Type;
use std::collections::HashMap;

pub(crate) fn run_specializations(
    module_name: &str,
    result: &mut LoweringResult,
    external_defs: &ExternalDefs,
    class_declarations: &crate::class_declarations::ClassDeclarationSet,
) -> Result<(), Vec<SpecializationDiagnostic>> {
    let mut errors =
        SpecializationDiagnostics::from_hir(verify_integer_boundaries(module_name, result));
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
            parent_class: class.semantic_parent_chain(),
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
        let described_shape =
            describe_type_with_externals(module_name, &target_type, result, external_defs);
        let mut shape = described_shape.to_const_value();
        let canonical_shape = crate::const_canonical::canonical_value(&shape);
        let Some(declaration) = class_declarations.get(&request.owner) else {
            errors.push(malformed(
                &request.package_module,
                "class_declaration",
                "specialization target has no pre-finalization declaration",
                request.range,
            ));
            continue;
        };
        declaration.attach_to_shape(&mut shape, result);
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
        let outcome = match decode_const_specialization_outcome(
            evaluated,
            request.range,
            declaration.origins(),
        ) {
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
                    let (method_slots, method_slot_context) =
                        match crate::slot_table::resolve_method_slots(
                            value,
                            &described_shape,
                            &target_type,
                            module_name,
                            result,
                            external_defs,
                        ) {
                            Ok(slots) => slots,
                            Err(problem) => {
                                errors.push(method_slot_diagnostic(problem, request.range));
                                continue;
                            }
                        };
                    let static_value = match static_program_value(value) {
                        Ok(value) => value,
                        Err(problem) => {
                            errors.push(malformed(
                                &request.package_module,
                                "static_program_value",
                                problem,
                                request.range,
                            ));
                            continue;
                        }
                    };
                    let canonical_value = crate::const_canonical::canonical_value(value);
                    let structural_contract_version = sifr_structural_identity::ALGORITHM_VERSION;
                    let adapter_identity =
                        crate::adapter_program_identity::post_adapter_hex(result, &request.owner);
                    let program_identity = sifr_structural_identity::static_program_identity(
                        structural_contract_version,
                        [
                            ("module", module_name.as_bytes()),
                            ("owner", request.owner.as_bytes()),
                            ("package", request.package_module.as_bytes()),
                            ("function", request.function.as_bytes()),
                            ("shape", canonical_shape.as_bytes()),
                            ("adapter", adapter_identity.as_bytes()),
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
                            value: static_value,
                            program_identity: *program_identity.as_bytes(),
                            structural_contract_version,
                            method_slots,
                            method_slot_context,
                        });
                }
                for issue in validated.issues {
                    match issue.severity {
                        ConstIssueSeverity::Warning => {
                            let related_ranges = issue
                                .labels
                                .iter()
                                .map(|label| (label.span, label.message.clone()))
                                .collect();
                            result
                                .warnings
                                .push(LoweringWarningDiagnostic::MetaPackageIssue {
                                    package: issue.package.clone(),
                                    reason_code: issue.reason_code.clone(),
                                    help: package_note(&issue),
                                    primary_range: Some(issue.primary_span),
                                    related_ranges,
                                });
                        }
                        ConstIssueSeverity::Fatal => errors.push_package(issue),
                    }
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.into_vec())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FrontendDiagnosticStyle, FrontendSourceContext, collect_module_exports, compile_module_hir,
        warning_diagnostics,
    };
    use sifr_diagnostics::DiagnosticArg;
    use sifr_lowering::StaticMethodSlotContext;
    use sifr_syntax::parse_module_suite;

    mod integer_boundary_tests;

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

    fn field_type<'a>(result: &'a LoweringResult, class_name: &str, field: &str) -> &'a Type {
        result
            .module
            .classes
            .iter()
            .find(|class| class.name == class_name)
            .and_then(|class| class.fields.iter().find(|(name, _)| name == field))
            .map(|(_, ty)| ty)
            .expect("fixture field exists")
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

class SourceOrigin:
    pass

class SourceDeclaration:
    origin: SourceOrigin

class ShapeInput:
    canonical_identity: str
    declaration: SourceDeclaration

class Label:
    origin: SourceOrigin
    message: str

class Issue:
    package: str
    reason_code: str
    severity: str
    arguments: IssueArgs
    primary_origin: SourceOrigin
    labels: list[Label]
    notes: list[str]

class Outcome:
    status: str
    value: str | None
    issues: list[Issue]

@const_eval
{template}def describe(shape: ShapeInput) -> Outcome:
    labels: list[Label] = [Label(shape.declaration.origin, "class declared here")]
    issue: Issue = Issue("fixture.meta", "{reason}", "{severity}", IssueArgs("value"), shape.declaration.origin, labels, ["package note"])
    issues: list[Issue] = [issue]
    if "{severity}" == "warning":
        return Outcome("produced", shape.canonical_identity, issues)
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
        assert!(
            target.specialization_outputs[0]
                .canonical_value
                .contains("target.Model")
        );
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
        let moved = compile("target", &format!("\n\n{TARGET}"), &external_defs)
            .expect("source movement compiles");
        assert_eq!(
            target.specialization_outputs[0].program_identity,
            moved.specialization_outputs[0].program_identity
        );
        let primary_range = |result: &LoweringResult| match result.warnings.last() {
            Some(LoweringWarningDiagnostic::MetaPackageIssue { primary_range, .. }) => {
                *primary_range
            }
            other => panic!("expected package warning, got {other:?}"),
        };
        assert_ne!(primary_range(&target), primary_range(&moved));
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
        let with_callback = compile(
            "target",
            &format!(
                "{TARGET}    @staticmethod\n    @metadata(\"fixture.callback\", \"after\")\n    def normalize(value: int) -> int:\n        return 0\n"
            ),
            &external_defs,
        )
        .expect("annotated method specializes");
        let with_changed_callback = compile(
            "target",
            &format!(
                "{TARGET}    @staticmethod\n    @metadata(\"fixture.callback\", \"after\")\n    def normalize(value: str) -> int:\n        return 0\n"
            ),
            &external_defs,
        )
        .expect("changed annotated method specializes");
        assert_ne!(
            target.specialization_outputs[0].program_identity,
            with_callback.specialization_outputs[0].program_identity
        );
        assert_ne!(
            with_callback.specialization_outputs[0].program_identity,
            with_changed_callback.specialization_outputs[0].program_identity
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
        assert_eq!(editor[0].spans.len(), 2);
        assert!(
            editor[0]
                .spans
                .iter()
                .any(|span| span.label.as_deref() == Some("class declared here"))
        );
    }

    #[test]
    fn imported_annotated_methods_preserve_nested_program_identity() {
        fn compile_consumer(method_return: &str) -> StaticSpecializationOutput {
            let mut external_defs = ExternalDefs::default();
            let package = compile(
                "fixture.meta",
                &package_source("warning", "shape_notice", true),
                &external_defs,
            )
            .expect("package compiles");
            collect_module_exports("fixture.meta", &package, &mut external_defs);

            let return_expression = if method_return == "T" {
                "value"
            } else {
                "\"\""
            };
            let model_source = format!(
                r#"
class Box[T]:
    value: T

    @staticmethod
    @metadata("fixture.callback", "normalize")
    @metadata("parameter", "value", "fixture.role", "input")
    def normalize(value: T) -> {method_return}:
        return {return_expression}
"#
            );
            let models =
                compile("models", &model_source, &external_defs).expect("model module compiles");
            collect_module_exports("models", &models, &mut external_defs);

            let consumer = compile(
                "consumer",
                r#"
from fixture.meta import describe
from models import Box

@const_specialize("fixture.meta", "describe")
class Container:
    item: Box[int]
"#,
                &external_defs,
            )
            .expect("consumer specializes imported model");
            consumer
                .specialization_outputs
                .into_iter()
                .next()
                .expect("consumer emits a static program")
        }

        let integer = compile_consumer("T");
        assert!(integer.canonical_value.contains("normalize:static"));
        assert!(integer.canonical_value.contains("fixture.callback"));
        assert!(integer.canonical_value.contains("5:value:borrow:false:int"));

        let string = compile_consumer("str");
        assert_ne!(integer.program_identity, string.program_identity);
    }

    #[test]
    fn qualified_reserved_slot_list_resolves_checked_method_contract() {
        let mut external_defs = ExternalDefs::default();
        let package = compile(
            "fixture.slots",
            r#"
class Outcome:
    status: str
    value: dict[str, list[str]] | None
    issues: list[str]

@const_eval
def describe(shape: dict[str, str]) -> Outcome:
    return Outcome("produced", {"sifr_method_slots": ["target.Record::normalize"]}, [])
"#,
            &external_defs,
        )
        .expect("slot package compiles");
        collect_module_exports("fixture.slots", &package, &mut external_defs);

        let target = compile(
            "target",
            r#"
from fixture.slots import describe

@const_specialize("fixture.slots", "describe")
class Record:
    value: str

    @staticmethod
    @metadata("fixture.slot", "normalize")
    def normalize(own value: str) -> Result[str, ValueError]:
        return value
"#,
            &external_defs,
        )
        .expect("qualified method slot specializes");
        let output = &target.specialization_outputs[0];
        assert_eq!(output.method_slots.len(), 1);
        assert_eq!(output.method_slots[0].owner_identity, "target.Record");
        assert_eq!(output.method_slots[0].name, "normalize");
        assert_eq!(output.method_slots[0].input_type, Type::Str);
        assert_eq!(output.method_slots[0].output_type, Type::Str);
        assert_eq!(
            output.method_slot_context,
            Some(StaticMethodSlotContext::None)
        );
    }

    #[test]
    fn invalid_reserved_slot_list_uses_slot_diagnostic() {
        let mut external_defs = ExternalDefs::default();
        let package = compile(
            "fixture.invalid_slots",
            r#"
class Outcome:
    status: str
    value: dict[str, str] | None
    issues: list[str]

@const_eval
def describe(shape: dict[str, str]) -> Outcome:
    return Outcome("produced", {"sifr_method_slots": "target.Record::normalize"}, [])
"#,
            &external_defs,
        )
        .expect("malformed-slot package declaration compiles");
        collect_module_exports("fixture.invalid_slots", &package, &mut external_defs);
        let diagnostics = errors(compile(
            "target",
            r#"
from fixture.invalid_slots import describe

@const_specialize("fixture.invalid_slots", "describe")
class Record:
    value: str
"#,
            &external_defs,
        ));
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "SIFR-RUST-SLOT-0001")
        );
    }

    #[test]
    fn reexported_nominal_shapes_match_local_shapes_without_consumer_fallbacks() {
        let mut external_defs = ExternalDefs::default();
        let models = compile(
            "models",
            r#"
from enum import Enum

@metadata("fixture.kind", "color")
@metadata("enum_variant", "RED", "fixture.label", "red")
class Color(Enum):
    RED = 1
    BLUE = 2

@metadata("fixture.kind", "port")
class Port(int):
    pass

class Box[T]:
    value: T

    @metadata("fixture.callback", "construct")
    def __init__(self, value: T) -> None:
        self.value = value

    @metadata("fixture.callback", "compare")
    def __eq__(self, other: Box[T]) -> bool:
        return True

    @classmethod
    @metadata("fixture.callback", "from_value")
    def from_value(cls, value: T) -> Box[T]:
        return Box(value)

    @staticmethod
    @metadata("fixture.callback", "normalize")
    @metadata("parameter", "value", "fixture.role", "input")
    async def normalize(*, value: T) -> T:
        return value

class LocalUse:
    item: Box[int]
    color: Color
    port: Port
"#,
            &external_defs,
        )
        .expect("models compile");
        let describe_local = |field| {
            crate::describe_type_with_externals(
                "models",
                field_type(&models, "LocalUse", field),
                &models,
                &external_defs,
            )
        };
        let local_box = describe_local("item");
        let local_color = describe_local("color");
        let local_port = describe_local("port");
        collect_module_exports("models", &models, &mut external_defs);

        let facade = compile(
            "facade",
            "from models import Box as Renamed\nfrom models import Color\nfrom models import Port\n",
            &external_defs,
        )
        .expect("facade reexports compile");
        collect_module_exports("facade", &facade, &mut external_defs);
        let consumer = compile(
            "consumer",
            r#"
from facade import Renamed, Color, Port

class ImportedUse:
    item: Renamed[int]
    color: Color
    port: Port
"#,
            &external_defs,
        )
        .expect("consumer compiles");

        let imported_box = crate::describe_type_with_externals(
            "consumer",
            field_type(&consumer, "ImportedUse", "item"),
            &consumer,
            &external_defs,
        );
        let imported_color = crate::describe_type_with_externals(
            "consumer",
            field_type(&consumer, "ImportedUse", "color"),
            &consumer,
            &external_defs,
        );
        let imported_port = crate::describe_type_with_externals(
            "consumer",
            field_type(&consumer, "ImportedUse", "port"),
            &consumer,
            &external_defs,
        );

        assert_eq!(
            local_box.canonical_identity,
            imported_box.canonical_identity
        );
        assert_eq!(
            local_color.canonical_identity,
            imported_color.canonical_identity
        );
        assert_eq!(
            local_port.canonical_identity,
            imported_port.canonical_identity
        );
        assert!(imported_box.canonical_identity.contains("__init__:regular"));
        assert!(imported_box.canonical_identity.contains("__eq__:regular"));
        assert!(imported_box.canonical_identity.contains("from_value:class"));
        assert!(
            imported_box
                .canonical_identity
                .contains("normalize:static:none:true")
        );
        assert!(imported_box.canonical_identity.contains("fixture.role"));
        assert!(imported_color.canonical_identity.contains("models.Color"));
        assert!(imported_color.canonical_identity.contains("fixture.label"));
        assert!(imported_port.canonical_identity.contains("models.Port"));
        assert!(imported_port.canonical_identity.contains("fixture.kind"));
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
    fn package_cannot_forge_a_source_origin() {
        let mut external_defs = ExternalDefs::default();
        let forged = package_source("fatal", "shape_rejected", true)
            .replace("shape.declaration.origin", "SourceOrigin()");
        let package = compile("fixture.meta", &forged, &external_defs)
            .expect("forging package compiles before origin validation");
        collect_module_exports("fixture.meta", &package, &mut external_defs);

        let diagnostics = errors(compile("target", TARGET, &external_defs));
        assert_eq!(diagnostics[0].code, "SIFR-META-0003");
        assert!(diagnostics[0].message.contains("primary_origin"));
    }
}
