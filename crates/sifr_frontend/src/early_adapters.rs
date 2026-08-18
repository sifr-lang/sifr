//! Erased marker selection and bounded early class-adapter execution.

mod adapter_input;
mod handler_plans;
mod inheritance;

use adapter_input::adapter_input;
use handler_plans::validate_handlers;

use crate::package_issues::{
    evaluation_error, issue_templates, replace_unknown_package, SpecializationDiagnostic,
    SpecializationDiagnostics,
};
use crate::specialization_support::{malformed, static_program_value};
use crate::{
    decode_const_specialization_outcome, package_note, ConstIssueSeverity, ConstPackageIssue,
    ConstValue, DeterministicConstEvaluator,
};
use sifr_lowering::{
    AdapterFieldDefault, AdapterFieldPlan, AdapterHandlerPlan, AppliedAdapterMetadata,
    ClassAdapterSelection, ConstSpecializationRequest, DeclarationMetadataTargetKind,
    LoweringWarningDiagnostic,
};
use sifr_lowering::{ExternalDefs, HirModule, LoweringResult};
use sifr_type_system::Type;
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
    let input = match adapter_input(
        module_name,
        result,
        external_defs,
        declaration,
        selection,
        &descriptors,
    ) {
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
    let canonical_input = crate::const_canonical::canonical_value(&input);
    let canonical_provider = crate::adapter_program_identity::canonical_const_functions(&functions);
    let invocation_identity = sifr_structural_identity::static_program_identity(
        sifr_structural_identity::ALGORITHM_VERSION,
        [
            ("contract", b"class-adapter-invocation-v1".as_slice()),
            ("provider_module", selection.provider_module.as_bytes()),
            ("provider_function", selection.provider_function.as_bytes()),
            ("provider_hir", canonical_provider.as_bytes()),
            ("input", canonical_input.as_bytes()),
        ],
    );
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
    let canonical_output = crate::const_canonical::canonical_value(&plan);
    let post_adapter_identity = sifr_structural_identity::static_program_identity(
        sifr_structural_identity::ALGORITHM_VERSION,
        [
            ("contract", b"class-adapter-post-v1".as_slice()),
            ("invocation", invocation_identity.as_bytes().as_slice()),
            ("output", canonical_output.as_bytes()),
        ],
    );
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
    match validate_plan(
        module_name,
        result,
        external_defs,
        declaration,
        selection,
        plan,
    ) {
        Ok(validated) => apply_plan(
            result,
            selection,
            validated,
            *invocation_identity.as_bytes(),
            *post_adapter_identity.as_bytes(),
        ),
        Err(problem) => diagnostics.push(malformed(
            &selection.provider_module,
            "adapter_plan",
            problem,
            selection.range,
        )),
    }
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
    if fields.len() != 6 {
        diagnostics.push(malformed(
            &selection.provider_module,
            "adapter_plan",
            "adapter plan must contain exactly these fields: fields, metadata, specialization_module, specialization_function, issues, and handlers",
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
    fields: Vec<AdapterFieldPlan>,
    handlers: Vec<AdapterHandlerPlan>,
    metadata: Vec<AppliedAdapterMetadata>,
    specialization: Option<(String, String)>,
}

fn validate_plan(
    module_name: &str,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
    declaration: &crate::class_declarations::ClassDeclaration,
    selection: &ClassAdapterSelection,
    value: ConstValue,
) -> Result<ValidatedPlan, String> {
    let ConstValue::Record(mut fields) = value else {
        return Err("adapter plan value must be a record".to_string());
    };
    if fields.len() != 5 {
        return Err("adapter plan contains unknown output fields".to_string());
    }
    let planned_fields = take_list(&mut fields, "fields")?;
    let planned_fields = validate_fields(
        module_name,
        selection,
        declaration,
        result,
        external_defs,
        planned_fields,
    )?;
    let metadata = take_list(&mut fields, "metadata")?;
    if metadata.len() > MAX_PLAN_METADATA {
        return Err(format!(
            "adapter plan contains more than {MAX_PLAN_METADATA} metadata values"
        ));
    }
    let metadata = metadata
        .into_iter()
        .map(|value| {
            parse_metadata(
                module_name,
                declaration,
                result,
                external_defs,
                selection,
                value,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let handlers = validate_handlers(
        module_name,
        declaration,
        result,
        external_defs,
        selection,
        take_list(&mut fields, "handlers")?,
    )?;
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
        fields: planned_fields,
        handlers,
        metadata,
        specialization,
    })
}

fn validate_fields(
    module_name: &str,
    selection: &ClassAdapterSelection,
    declaration: &crate::class_declarations::ClassDeclaration,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
    values: Vec<ConstValue>,
) -> Result<Vec<AdapterFieldPlan>, String> {
    let actual = values
        .into_iter()
        .map(|value| {
            let ConstValue::Record(mut fields) = value else {
                return Err("planned field must be a record".to_string());
            };
            if fields.len() != 6 {
                return Err("planned field must contain exactly identity, declared_type, default_kind, default_value, default_factory, and validation_policy".to_string());
            }
            let identity = take_string(&mut fields, "identity")?;
            let declared_type = take_string(&mut fields, "declared_type")?;
            let default_kind = take_string(&mut fields, "default_kind")?;
            let default_value = fields
                .remove("default_value")
                .ok_or_else(|| "planned field is missing default_value".to_string())?;
            let default_factory = fields
                .remove("default_factory")
                .ok_or_else(|| "planned field is missing default_factory".to_string())?;
            let validation_policy = fields
                .remove("validation_policy")
                .ok_or_else(|| "planned field is missing validation_policy".to_string())?;
            Ok((
                identity,
                declared_type,
                default_kind,
                default_value,
                default_factory,
                validation_policy,
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let expected =
        expected_field_contracts(module_name, result, external_defs, declaration, selection)?;
    if actual.len() != expected.len()
        || actual
            .iter()
            .zip(&expected)
            .any(|(actual, expected)| (&actual.0, &actual.1) != (&expected.0, &expected.1))
    {
        return Err(
            "adapter plan fields must preserve every declared field identity, order, and type"
                .to_string(),
        );
    }
    let class = result
        .module
        .classes
        .iter()
        .find(|class| class.name == selection.owner)
        .ok_or_else(|| "adapted class fields are unavailable".to_string())?;
    let normalized_types = normalized_field_types(
        module_name,
        result,
        external_defs,
        selection,
        class,
        &expected,
    )?;
    actual
        .into_iter()
        .zip(normalized_types)
        .map(
            |((identity, _, kind, value, factory, policy), (name, ty))| {
                let default = match (kind.as_str(), value, factory) {
                    ("required", ConstValue::None, ConstValue::None) => {
                        AdapterFieldDefault::Required
                    }
                    ("const", value, ConstValue::None) => {
                        if !crate::typed_descriptors::const_value_assignable(&value, &ty) {
                            return Err(format!(
                                "planned constant default for field '{name}' is not assignable to its declared type"
                            ));
                        }
                        AdapterFieldDefault::Const(
                            static_program_value(&value).map_err(str::to_string)?,
                        )
                    }
                    ("factory", ConstValue::None, ConstValue::CallableIdentity(factory)) => {
                        let contract = crate::callable_identities::contract_for_identity(
                            module_name,
                            result,
                            external_defs,
                            &factory,
                        )
                        .ok_or_else(|| {
                            format!(
                                "planned default factory for field '{name}' is not a checked callable"
                            )
                        })?;
                        if !contract.params.is_empty() {
                            return Err(format!(
                                "planned default factory for field '{name}' must accept no arguments"
                            ));
                        }
                        if !contract.return_type.is_assignable_to(&ty) {
                            return Err(format!(
                                "planned default factory for field '{name}' does not return its declared type"
                            ));
                        }
                        AdapterFieldDefault::Factory(factory)
                    }
                    _ => {
                        return Err(format!(
                            "planned field '{name}' must use a valid required, const, or factory default state"
                        ));
                    }
                };
                let validation_policy = match policy {
                    ConstValue::None => None,
                    ConstValue::String(value) => Some(sifr_lowering::StaticProgramValue::String(value)),
                    _ => return Err(format!(
                        "planned default validation policy for field '{name}' must be a string or None"
                    )),
                };
                Ok(AdapterFieldPlan {
                    identity,
                    name: name.clone(),
                    declared_type: ty,
                    default,
                    validation_policy,
                })
            },
        )
        .collect()
}

fn parse_metadata(
    module_name: &str,
    declaration: &crate::class_declarations::ClassDeclaration,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
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
            let fields = expected_field_contracts(
                module_name,
                result,
                external_defs,
                declaration,
                selection,
            )?;
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

fn normalized_field_types(
    module_name: &str,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
    selection: &ClassAdapterSelection,
    class: &sifr_lowering::HirClass,
    expected: &[(String, String)],
) -> Result<Vec<(String, Type)>, String> {
    let local_prefix = format!("{module_name}.{}.", selection.owner);
    let parent_name = selection.data_parent.as_deref();
    let parent_identity = class
        .parent_type
        .as_ref()
        .and_then(|parent| match parent {
            Type::Class { identity, name, .. } => {
                Some(identity.as_deref().unwrap_or(name).to_string())
            }
            _ => None,
        })
        .or_else(|| parent_name.map(|name| format!("{module_name}.{name}")));
    let parent = parent_name.and_then(|name| {
        inheritance::parent_selection(
            result,
            external_defs,
            parent_identity.as_deref().unwrap_or(""),
            name,
        )
    });
    let type_args = match class.parent_type.as_ref() {
        Some(Type::Class { type_args, .. }) => type_args.as_slice(),
        _ => &[],
    };
    let bindings = parent_name.map_or_else(HashMap::new, |name| {
        inheritance::parent_bindings(
            result,
            external_defs,
            parent_identity.as_deref().unwrap_or(""),
            name,
            type_args,
        )
    });
    expected
        .iter()
        .map(|(identity, _)| {
            let name = identity
                .rsplit_once('.')
                .map(|(_, name)| name.to_string())
                .ok_or_else(|| "planned field identity is malformed".to_string())?;
            let ty = if identity.starts_with(&local_prefix) {
                class
                    .fields
                    .iter()
                    .find(|(field, _)| field == &name)
                    .map(|(_, ty)| ty.clone())
                    .or_else(|| {
                        class.parent_type.as_ref().and_then(|parent| match parent {
                            Type::Class { fields, .. } => fields
                                .iter()
                                .find(|(field, _)| field == &name)
                                .map(|(_, ty)| ty.clone()),
                            _ => None,
                        })
                    })
            } else {
                parent
                    .into_iter()
                    .flat_map(|parent| parent.field_plans.iter())
                    .find(|field| field.identity == *identity)
                    .map(|field| {
                        sifr_lowering::substitute_type_vars(&field.declared_type, &bindings)
                    })
                    .or_else(|| {
                        class.parent_type.as_ref().and_then(|parent| match parent {
                            Type::Class { fields, .. } => fields
                                .iter()
                                .find(|(field, _)| field == &name)
                                .map(|(_, ty)| ty.clone()),
                            _ => None,
                        })
                    })
            }
            .ok_or_else(|| format!("normalized type for field '{name}' is unavailable"))?;
            Ok((name, ty))
        })
        .collect()
}

fn expected_field_contracts(
    module_name: &str,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
    declaration: &crate::class_declarations::ClassDeclaration,
    selection: &ClassAdapterSelection,
) -> Result<Vec<(String, String)>, String> {
    let value =
        inheritance::declaration_value(module_name, result, external_defs, declaration, selection)
            .map_err(str::to_string)?;
    let ConstValue::Record(fields) = value else {
        return Err("class declaration is not structural".to_string());
    };
    let Some(ConstValue::List(items)) = fields.get("items") else {
        return Err("class declaration items are unavailable".to_string());
    };
    items
        .iter()
        .filter_map(|item| {
            let ConstValue::Record(fields) = item else {
                return None;
            };
            if !matches!(fields.get("kind"), Some(ConstValue::String(kind)) if kind == "field") {
                return None;
            }
            Some(
                match (fields.get("identity"), fields.get("declared_type")) {
                    (Some(ConstValue::String(identity)), Some(ConstValue::String(ty))) => {
                        Ok((identity.clone(), ty.clone()))
                    }
                    _ => Err(format!(
                        "declared field identity or type is unavailable for '{}'",
                        fields
                            .get("name")
                            .and_then(|name| match name {
                                ConstValue::String(name) => Some(name.as_str()),
                                _ => None,
                            })
                            .unwrap_or("<unknown>")
                    )),
                },
            )
        })
        .collect()
}

fn apply_plan(
    result: &mut LoweringResult,
    selection: &ClassAdapterSelection,
    plan: ValidatedPlan,
    adapter_invocation_identity: [u8; 32],
    post_adapter_identity: [u8; 32],
) {
    if let Some(applied) = result
        .class_adapter_selections
        .iter_mut()
        .find(|candidate| candidate.owner == selection.owner)
    {
        applied.field_plans = plan.fields;
        applied.handler_plans = plan.handlers;
        applied.adapter_invocation_identity = adapter_invocation_identity;
        applied.post_adapter_identity = post_adapter_identity;
    }
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
