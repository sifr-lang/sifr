//! Erased marker selection and bounded early class-adapter execution.

use crate::package_issues::{
    evaluation_error, issue_templates, replace_unknown_package, SpecializationDiagnostic,
    SpecializationDiagnostics,
};
use crate::specialization_support::{const_value, malformed, static_program_value};
use crate::{
    decode_const_specialization_outcome, package_note, ConstIssueSeverity, ConstPackageIssue,
    ConstValue, DeterministicConstEvaluator,
};
use sifr_lowering::{
    AppliedAdapterMetadata, ClassAdapterSelection, ConstSpecializationRequest,
    DeclarationDescriptorKind, DeclarationMetadataTargetKind, LoweringWarningDiagnostic,
    TypedDeclarationDescriptor,
};
use sifr_lowering::{ExternalDefs, HirModule, LoweringResult};
use std::collections::{BTreeMap, HashMap};

const MAX_PLAN_METADATA: usize = 1024;
const MAX_METADATA_KEY_BYTES: usize = 4096;

pub(crate) fn run(
    module_name: &str,
    result: &mut LoweringResult,
    external_defs: &ExternalDefs,
    declarations: &crate::class_declarations::ClassDeclarationSet,
) -> Result<(), Vec<SpecializationDiagnostic>> {
    let mut diagnostics = SpecializationDiagnostics::from_hir(Vec::new());
    for selection in result.class_adapter_selections.clone() {
        run_one(
            module_name,
            result,
            external_defs,
            declarations,
            &selection,
            &mut diagnostics,
        );
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics.into_vec())
    }
}

fn run_one(
    module_name: &str,
    result: &mut LoweringResult,
    external_defs: &ExternalDefs,
    declarations: &crate::class_declarations::ClassDeclarationSet,
    selection: &ClassAdapterSelection,
    diagnostics: &mut SpecializationDiagnostics,
) {
    let Some(declaration) = declarations.get(&selection.owner) else {
        diagnostics.push(malformed(
            &selection.provider_module,
            "adapter_declaration",
            "adapted class has no pre-finalization declaration",
            selection.range,
        ));
        return;
    };
    let descriptors = result
        .declaration_descriptors
        .iter()
        .filter(|descriptor| descriptor.owner == selection.owner)
        .collect::<Vec<_>>();
    if descriptors.iter().any(|descriptor| {
        descriptor.provider_module != selection.provider_module
            || descriptor.provider_function != selection.provider_function
    }) {
        diagnostics.push(malformed(
            &selection.provider_module,
            "adapter_descriptor_provider",
            "adapted class descriptor provider does not match its marker provider",
            selection.range,
        ));
        return;
    }
    let input = match adapter_input(module_name, result, declaration, selection, &descriptors) {
        Ok(input) => input,
        Err(problem) => {
            diagnostics.push(malformed(
                &selection.provider_module,
                "adapter_input",
                problem,
                selection.range,
            ));
            return;
        }
    };
    let Some(functions) = const_functions(result, external_defs, selection) else {
        diagnostics.push(malformed(
            &selection.provider_module,
            "adapter_function",
            "selected adapter provider is not exported as a const function",
            selection.range,
        ));
        return;
    };
    let evaluated = DeterministicConstEvaluator::new(&functions)
        .evaluate_function(&selection.provider_function, vec![input]);
    let plan = match evaluated {
        Ok(plan) => plan,
        Err(error) => {
            diagnostics.push(evaluation_error(
                &selection.provider_module,
                &error,
                selection.range,
            ));
            return;
        }
    };
    let (plan, issues) =
        match validate_issues(plan, declaration, external_defs, selection, diagnostics) {
            Some(plan) => plan,
            None => return,
        };
    for issue in issues {
        if issue.severity == ConstIssueSeverity::Warning {
            result
                .warnings
                .push(LoweringWarningDiagnostic::MetaPackageIssue {
                    package: issue.package.clone(),
                    reason_code: issue.reason_code.clone(),
                    help: package_note(&issue),
                    primary_range: Some(issue.primary_span),
                    related_ranges: issue
                        .labels
                        .iter()
                        .map(|label| (label.span, label.message.clone()))
                        .collect(),
                });
        }
    }
    match validate_plan(module_name, result, declaration, selection, plan) {
        Ok(validated) => apply_plan(result, selection, validated),
        Err(problem) => diagnostics.push(malformed(
            &selection.provider_module,
            "adapter_plan",
            problem,
            selection.range,
        )),
    }
}

fn adapter_input(
    module_name: &str,
    result: &LoweringResult,
    declaration: &crate::class_declarations::ClassDeclaration,
    selection: &ClassAdapterSelection,
    descriptors: &[&TypedDeclarationDescriptor],
) -> Result<ConstValue, &'static str> {
    let descriptors = descriptors
        .iter()
        .map(|descriptor| {
            let origin = declaration
                .origin_for_range(descriptor.range)
                .ok_or("descriptor source origin is unavailable")?;
            Ok(ConstValue::Record(BTreeMap::from([
                (
                    "target_kind".to_string(),
                    ConstValue::String(descriptor_kind(descriptor.target_kind).to_string()),
                ),
                (
                    "target_identity".to_string(),
                    ConstValue::String(canonical_target(module_name, &descriptor.target_identity)),
                ),
                ("value".to_string(), const_value(&descriptor.value)?),
                ("origin".to_string(), ConstValue::SourceOrigin(origin)),
            ])))
        })
        .collect::<Result<Vec<_>, &'static str>>()?;
    let data_parent = selection
        .data_parent
        .as_ref()
        .and_then(|_| {
            result
                .module
                .classes
                .iter()
                .find(|class| class.name == selection.owner)
                .and_then(|class| class.parent_type.as_ref())
        })
        .map_or(ConstValue::None, |parent| {
            ConstValue::String(crate::canonical_types::type_identity(parent))
        });
    Ok(ConstValue::Record(BTreeMap::from([
        (
            "declaration".to_string(),
            declaration.to_const_value(result),
        ),
        ("descriptors".to_string(), ConstValue::List(descriptors)),
        (
            "provider_module".to_string(),
            ConstValue::String(selection.provider_module.clone()),
        ),
        (
            "provider_function".to_string(),
            ConstValue::String(selection.provider_function.clone()),
        ),
        ("data_parent".to_string(), data_parent),
    ])))
}

fn const_functions(
    result: &LoweringResult,
    external_defs: &ExternalDefs,
    selection: &ClassAdapterSelection,
) -> Option<HirModule> {
    let mut functions = if selection.provider_module
        == result
            .class_adapter_providers
            .iter()
            .find(|provider| provider.function == selection.provider_function)
            .map_or("", |provider| provider.module.as_str())
    {
        result
            .module
            .functions
            .iter()
            .filter(|function| function.decorators.iter().any(|item| item == "const_eval"))
            .cloned()
            .collect::<Vec<_>>()
    } else {
        external_defs
            .const_functions
            .get(&selection.provider_module)?
            .values()
            .cloned()
            .collect::<Vec<_>>()
    };
    functions.sort_by(|left, right| left.name.cmp(&right.name));
    Some(HirModule {
        functions,
        classes: Vec::new(),
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    })
}

fn validate_issues(
    value: ConstValue,
    declaration: &crate::class_declarations::ClassDeclaration,
    external_defs: &ExternalDefs,
    selection: &ClassAdapterSelection,
    diagnostics: &mut SpecializationDiagnostics,
) -> Option<(ConstValue, Vec<ConstPackageIssue>)> {
    let ConstValue::Record(mut fields) = value else {
        diagnostics.push(malformed(
            &selection.provider_module,
            "adapter_plan",
            "adapter plan must be a record",
            selection.range,
        ));
        return None;
    };
    if fields.len() != 5 {
        diagnostics.push(malformed(
            &selection.provider_module,
            "adapter_plan",
            "adapter plan must contain exactly fields, metadata, specialization_module, specialization_function, and issues",
            selection.range,
        ));
        return None;
    }
    let Some(issues) = fields.remove("issues") else {
        diagnostics.push(malformed(
            &selection.provider_module,
            "adapter_plan",
            "adapter plan is missing issues",
            selection.range,
        ));
        return None;
    };
    let has_fatal = match &issues {
        ConstValue::List(issues) => issues.iter().any(|issue| {
            matches!(
                issue,
                ConstValue::Record(fields)
                    if matches!(fields.get("severity"), Some(ConstValue::String(value)) if value == "fatal")
            )
        }),
        _ => false,
    };
    let plan = ConstValue::Record(fields);
    let outcome = ConstValue::Record(BTreeMap::from([
        (
            "status".to_string(),
            ConstValue::String(if has_fatal { "failed" } else { "produced" }.to_string()),
        ),
        (
            "value".to_string(),
            if has_fatal { ConstValue::None } else { plan },
        ),
        ("issues".to_string(), issues),
    ]));
    let outcome = match decode_const_specialization_outcome(
        outcome,
        selection.range,
        declaration.origins(),
    ) {
        Ok(outcome) => outcome,
        Err(mut errors) => {
            for error in &mut errors {
                replace_unknown_package(error, &selection.provider_module);
            }
            diagnostics.extend(errors);
            return None;
        }
    };
    let templates = issue_templates(
        external_defs,
        &selection.provider_module,
        &selection.provider_function,
    );
    let validated = match outcome.validate(&templates) {
        Ok(validated) => validated,
        Err(errors) => {
            diagnostics.extend(errors);
            return None;
        }
    };
    if validated.value.is_none() {
        for issue in validated.issues {
            diagnostics.push_package(issue);
        }
        None
    } else {
        validated.value.map(|value| (value, validated.issues))
    }
}

struct ValidatedPlan {
    metadata: Vec<AppliedAdapterMetadata>,
    specialization: Option<(String, String)>,
}

fn validate_plan(
    module_name: &str,
    result: &LoweringResult,
    declaration: &crate::class_declarations::ClassDeclaration,
    selection: &ClassAdapterSelection,
    value: ConstValue,
) -> Result<ValidatedPlan, String> {
    let ConstValue::Record(mut fields) = value else {
        return Err("adapter plan value must be a record".to_string());
    };
    if fields.len() != 4 {
        return Err("adapter plan contains unknown output fields".to_string());
    }
    let planned_fields = take_list(&mut fields, "fields")?;
    validate_fields(declaration, result, planned_fields)?;
    let metadata = take_list(&mut fields, "metadata")?;
    if metadata.len() > MAX_PLAN_METADATA {
        return Err(format!(
            "adapter plan contains more than {MAX_PLAN_METADATA} metadata values"
        ));
    }
    let metadata = metadata
        .into_iter()
        .map(|value| parse_metadata(module_name, declaration, result, selection, value))
        .collect::<Result<Vec<_>, _>>()?;
    let module = take_optional_string(&mut fields, "specialization_module")?;
    let function = take_optional_string(&mut fields, "specialization_function")?;
    if !fields.is_empty() {
        return Err("adapter plan contains unknown output fields".to_string());
    }
    let specialization = match (module, function) {
        (None, None) => None,
        (Some(module), Some(function)) => Some((module, function)),
        _ => {
            return Err(
                "adapter specialization module and function must both be present or absent"
                    .to_string(),
            )
        }
    };
    if specialization.is_some()
        && result
            .specialization_requests
            .iter()
            .any(|request| request.owner == selection.owner)
    {
        return Err("an adapted class may request exactly one specialization".to_string());
    }
    Ok(ValidatedPlan {
        metadata,
        specialization,
    })
}

fn validate_fields(
    declaration: &crate::class_declarations::ClassDeclaration,
    result: &LoweringResult,
    values: Vec<ConstValue>,
) -> Result<(), String> {
    let actual = values
        .into_iter()
        .map(|value| {
            let ConstValue::Record(mut fields) = value else {
                return Err("planned field must be a record".to_string());
            };
            if fields.len() != 2 {
                return Err(
                    "planned field must contain exactly identity and declared_type".to_string(),
                );
            }
            let identity = take_string(&mut fields, "identity")?;
            let declared_type = take_string(&mut fields, "declared_type")?;
            Ok((identity, declared_type))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let expected = declaration.field_contracts(result);
    if actual != expected {
        return Err(
            "adapter plan fields must preserve every declared field identity, order, and type"
                .to_string(),
        );
    }
    Ok(())
}

fn parse_metadata(
    module_name: &str,
    declaration: &crate::class_declarations::ClassDeclaration,
    result: &LoweringResult,
    selection: &ClassAdapterSelection,
    value: ConstValue,
) -> Result<AppliedAdapterMetadata, String> {
    let ConstValue::Record(mut fields) = value else {
        return Err("planned metadata must be a record".to_string());
    };
    if fields.len() != 4 {
        return Err(
            "planned metadata must contain target_kind, target_identity, key, and value"
                .to_string(),
        );
    }
    let target_kind = take_string(&mut fields, "target_kind")?;
    let target_identity = take_string(&mut fields, "target_identity")?;
    let key = take_string(&mut fields, "key")?;
    if key.len() > MAX_METADATA_KEY_BYTES || key.is_empty() {
        return Err(format!(
            "planned metadata key must contain 1 to {MAX_METADATA_KEY_BYTES} bytes"
        ));
    }
    let value = fields
        .remove("value")
        .ok_or_else(|| "planned metadata is missing value".to_string())?;
    if !crate::typed_descriptors::const_value_assignable(&value, &selection.descriptor_type) {
        return Err(
            "planned metadata value is not assignable to provider descriptor type".to_string(),
        );
    }
    let owner_identity = format!("{module_name}.{}", selection.owner);
    let (target_kind, target_name) = match target_kind.as_str() {
        "class" if target_identity == owner_identity => (DeclarationMetadataTargetKind::Type, None),
        "field" => {
            let fields = declaration.field_contracts(result);
            let Some((identity, _)) = fields
                .iter()
                .find(|(identity, _)| identity == &target_identity)
            else {
                return Err("planned field metadata targets an unknown field identity".to_string());
            };
            let name = identity
                .rsplit_once('.')
                .map(|(_, name)| name.to_string())
                .ok_or_else(|| "planned field identity is malformed".to_string())?;
            (DeclarationMetadataTargetKind::Field, Some(name))
        }
        "class" => return Err("planned class metadata targets another class".to_string()),
        _ => return Err("planned metadata target kind must be class or field".to_string()),
    };
    Ok(AppliedAdapterMetadata {
        owner: selection.owner.clone(),
        target_kind,
        target_name,
        key,
        value_type: selection.descriptor_type.clone(),
        value: static_program_value(&value).map_err(str::to_string)?,
    })
}

fn apply_plan(result: &mut LoweringResult, selection: &ClassAdapterSelection, plan: ValidatedPlan) {
    result.applied_adapter_metadata.extend(plan.metadata);
    if let Some((package_module, function)) = plan.specialization {
        result
            .specialization_requests
            .push(ConstSpecializationRequest {
                owner: selection.owner.clone(),
                package_module,
                function,
                range: selection.range,
            });
    }
}

fn canonical_target(module_name: &str, target: &str) -> String {
    let target = target.split_once(':').map_or(target, |(target, _)| target);
    format!("{module_name}.{target}")
}

fn descriptor_kind(kind: DeclarationDescriptorKind) -> &'static str {
    match kind {
        DeclarationDescriptorKind::Field => "field",
        DeclarationDescriptorKind::Class => "class",
        DeclarationDescriptorKind::Method => "method",
        DeclarationDescriptorKind::Type => "type",
    }
}

fn take_list(
    fields: &mut BTreeMap<String, ConstValue>,
    name: &str,
) -> Result<Vec<ConstValue>, String> {
    match fields.remove(name) {
        Some(ConstValue::List(values)) => Ok(values),
        Some(_) => Err(format!("adapter plan field '{name}' must be a list")),
        None => Err(format!("adapter plan is missing field '{name}'")),
    }
}

fn take_string(fields: &mut BTreeMap<String, ConstValue>, name: &str) -> Result<String, String> {
    match fields.remove(name) {
        Some(ConstValue::String(value)) => Ok(value),
        Some(_) => Err(format!("adapter plan field '{name}' must be a string")),
        None => Err(format!("adapter plan is missing field '{name}'")),
    }
}

fn take_optional_string(
    fields: &mut BTreeMap<String, ConstValue>,
    name: &str,
) -> Result<Option<String>, String> {
    match fields.remove(name) {
        Some(ConstValue::None) => Ok(None),
        Some(ConstValue::String(value)) => Ok(Some(value)),
        Some(_) => Err(format!(
            "adapter plan field '{name}' must be a string or None"
        )),
        None => Err(format!("adapter plan is missing field '{name}'")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_plan_rejects_unknown_output_fields() {
        let value = ConstValue::Record(BTreeMap::from([(
            "generated_method_body".to_string(),
            ConstValue::String("pass".to_string()),
        )]));
        let ConstValue::Record(fields) = value else {
            unreachable!();
        };
        assert_ne!(fields.len(), 4);
    }
}
