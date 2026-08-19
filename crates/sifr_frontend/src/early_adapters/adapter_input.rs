//! Closed declaration input passed to one package adapter invocation.

use super::{handler_plans::inherited_handler_plans, inheritance};
use crate::specialization_support::const_value;
use crate::ConstValue;
use sifr_lowering::{
    ClassAdapterSelection, DeclarationDescriptorKind, ExternalDefs, LoweringResult,
    TypedDeclarationDescriptor,
};
use std::collections::BTreeMap;

pub(super) fn adapter_input(
    module_name: &str,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
    declaration: &crate::class_declarations::ClassDeclaration,
    selection: &ClassAdapterSelection,
    descriptors: &[&TypedDeclarationDescriptor],
) -> Result<ConstValue, &'static str> {
    let mut descriptor_values = inherited_values(module_name, result, external_defs, selection)?;
    descriptor_values.extend(local_values(module_name, declaration, descriptors)?);
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
    let declaration =
        inheritance::declaration_value(module_name, result, external_defs, declaration, selection)?;
    Ok(ConstValue::Record(BTreeMap::from([
        ("declaration".to_string(), declaration),
        (
            "descriptors".to_string(),
            ConstValue::List(descriptor_values),
        ),
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

fn inherited_values(
    module_name: &str,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
    selection: &ClassAdapterSelection,
) -> Result<Vec<ConstValue>, &'static str> {
    inherited_handler_plans(module_name, result, external_defs, selection)?
        .into_iter()
        .map(|handler| {
            Ok(ConstValue::Record(BTreeMap::from([
                (
                    "target_kind".to_string(),
                    ConstValue::String("method".to_string()),
                ),
                (
                    "target_identity".to_string(),
                    ConstValue::String(format!(
                        "{}::{}",
                        handler
                            .callable
                            .owner
                            .as_deref()
                            .unwrap_or(&handler.callable.module),
                        handler.callable.symbol
                    )),
                ),
                (
                    "target_callable".to_string(),
                    ConstValue::CallableIdentity(handler.callable),
                ),
                ("value".to_string(), const_value(&handler.descriptor_value)?),
                (
                    "origin".to_string(),
                    ConstValue::SourceOrigin(handler.descriptor_origin),
                ),
            ])))
        })
        .collect()
}

fn local_values(
    module_name: &str,
    declaration: &crate::class_declarations::ClassDeclaration,
    descriptors: &[&TypedDeclarationDescriptor],
) -> Result<Vec<ConstValue>, &'static str> {
    descriptors
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
                (
                    "target_callable".to_string(),
                    descriptor
                        .target_callable
                        .clone()
                        .map(ConstValue::CallableIdentity)
                        .unwrap_or(ConstValue::None),
                ),
                ("value".to_string(), const_value(&descriptor.value)?),
                ("origin".to_string(), ConstValue::SourceOrigin(origin)),
            ])))
        })
        .collect()
}

fn canonical_target(module_name: &str, target: &str) -> String {
    let target = target.split_once(':').map_or(target, |(target, _)| target);
    format!("{module_name}.{target}")
}

const fn descriptor_kind(kind: DeclarationDescriptorKind) -> &'static str {
    match kind {
        DeclarationDescriptorKind::Field => "field",
        DeclarationDescriptorKind::Class => "class",
        DeclarationDescriptorKind::Method => "method",
        DeclarationDescriptorKind::Type => "type",
    }
}
