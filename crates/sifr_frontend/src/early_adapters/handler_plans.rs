//! Validation and inheritance of package-selected method handlers.

use super::inheritance;
use crate::specialization_support::static_program_value;
use crate::ConstValue;
use sifr_lowering::{
    AdapterHandlerPlan, ClassAdapterSelection, DeclarationDescriptorKind, ExternalDefs,
    LoweringResult,
};
use sifr_type_system::Type;
use std::collections::BTreeSet;

pub(super) fn validate_handlers(
    module_name: &str,
    declaration: &crate::class_declarations::ClassDeclaration,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
    selection: &ClassAdapterSelection,
    values: Vec<ConstValue>,
) -> Result<Vec<AdapterHandlerPlan>, String> {
    let mut eligible = inherited_handler_plans(module_name, result, external_defs, selection);
    let inherited_count = eligible.len();
    eligible.extend(
        result
            .declaration_descriptors
            .iter()
            .filter(|descriptor| {
                descriptor.owner == selection.owner
                    && descriptor.target_kind == DeclarationDescriptorKind::Method
            })
            .enumerate()
            .map(|(index, descriptor)| {
                let callable = descriptor.target_callable.clone().ok_or_else(|| {
                    "method descriptor target has no sealed callable identity".to_string()
                })?;
                let descriptor_origin = declaration
                    .origin_for_range(descriptor.range)
                    .ok_or_else(|| "method descriptor source origin is unavailable".to_string())?;
                Ok(AdapterHandlerPlan {
                    callable,
                    descriptor_type: descriptor.value_type.clone(),
                    descriptor_value: descriptor.value.clone(),
                    descriptor_origin,
                    descriptor_range: descriptor.range,
                    declaration_order: inherited_count + index,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
    );

    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .map(|value| validate_one(value, &eligible, &mut seen))
        .collect()
}

fn validate_one(
    value: ConstValue,
    eligible: &[AdapterHandlerPlan],
    seen: &mut BTreeSet<(String, Option<String>, String)>,
) -> Result<AdapterHandlerPlan, String> {
    let ConstValue::Record(mut fields) = value else {
        return Err("planned handler must be a record".to_string());
    };
    if fields.len() != 3 {
        return Err("planned handler must contain target, descriptor, and origin".to_string());
    }
    let callable = match fields.remove("target") {
        Some(ConstValue::CallableIdentity(callable)) => callable,
        _ => return Err("planned handler target must be a sealed callable identity".to_string()),
    };
    let descriptor = fields
        .remove("descriptor")
        .ok_or_else(|| "planned handler is missing descriptor".to_string())?;
    let origin = match fields.remove("origin") {
        Some(ConstValue::SourceOrigin(origin)) => origin,
        _ => return Err("planned handler origin must be a compiler source origin".to_string()),
    };
    if !fields.is_empty() {
        return Err("planned handler contains unknown fields".to_string());
    }
    let key = (
        callable.module.clone(),
        callable.owner.clone(),
        callable.symbol.clone(),
    );
    if !seen.insert(key) {
        return Err(format!(
            "planned handler '{}::{}' is duplicated",
            callable.owner.as_deref().unwrap_or(&callable.module),
            callable.symbol
        ));
    }
    let descriptor = static_program_value(&descriptor).map_err(str::to_string)?;
    eligible
        .iter()
        .find(|candidate| {
            candidate.callable == callable
                && candidate.descriptor_value == descriptor
                && candidate.descriptor_origin == origin
        })
        .cloned()
        .ok_or_else(|| {
            format!(
                "planned handler '{}::{}' does not match a method descriptor on the adapted declaration",
                callable.owner.as_deref().unwrap_or(&callable.module),
                callable.symbol
            )
        })
}

pub(super) fn inherited_handler_plans(
    module_name: &str,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
    selection: &ClassAdapterSelection,
) -> Vec<AdapterHandlerPlan> {
    if selection.data_parent.is_none() {
        return Vec::new();
    }
    let Some(parent_identity) = result
        .module
        .classes
        .iter()
        .find(|class| class.name == selection.owner)
        .and_then(|class| class.parent_type.as_ref())
        .and_then(|parent| match parent {
            Type::Class { identity, name, .. } => Some(inheritance::canonical_parent_identity(
                module_name,
                identity.as_deref(),
                name,
            )),
            _ => None,
        })
    else {
        return Vec::new();
    };
    inheritance::parent_selection(module_name, result, external_defs, &parent_identity)
        .map(|parent| parent.handler_plans.clone())
        .unwrap_or_default()
}
