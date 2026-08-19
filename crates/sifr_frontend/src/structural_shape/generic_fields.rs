//! Concrete field types selected by finalized adapted generic declarations.

use sifr_lowering::{
    substitute_type_vars_with_class_scopes, AdapterFieldPlan, ExternalDefs, HirClass,
    LoweringResult,
};
use sifr_type_system::Type;
use std::collections::HashMap;

#[allow(clippy::too_many_arguments)]
pub(super) fn effective_fields(
    local_class: Option<&HirClass>,
    source_module: &str,
    source_name: &str,
    type_args: &[Type],
    declared_fields: &[(String, Type)],
    field_plans: Option<&[AdapterFieldPlan]>,
    lowering: &LoweringResult,
    external_defs: &ExternalDefs,
) -> Vec<(String, Type)> {
    let owner_is_local = local_class.is_some();
    let Some(field_plans) = field_plans else {
        return declared_fields.to_vec();
    };
    let type_params = local_class
        .map(|class| class.type_params.as_slice())
        .or_else(|| {
            external_defs
                .class_type_params
                .get(source_module)
                .and_then(|classes| classes.get(source_name))
                .map(Vec::as_slice)
        })
        .unwrap_or_default();
    let bindings = type_params
        .iter()
        .cloned()
        .zip(type_args.iter().cloned())
        .collect::<HashMap<_, _>>();
    field_plans
        .iter()
        .map(|field| {
            (
                field.name.clone(),
                substitute_type_vars_with_class_scopes(
                    &field.declared_type,
                    &bindings,
                    &|identity, name| {
                        class_type_params(
                            identity,
                            name,
                            source_module,
                            owner_is_local,
                            lowering,
                            external_defs,
                        )
                    },
                ),
            )
        })
        .collect()
}

fn class_type_params(
    identity: Option<&str>,
    name: &str,
    module_name: &str,
    owner_is_local: bool,
    lowering: &LoweringResult,
    external_defs: &ExternalDefs,
) -> Option<Vec<String>> {
    let (source_module, source_name) = identity
        .and_then(|identity| identity.rsplit_once('.'))
        .unwrap_or((module_name, name));
    let canonical_identity = format!("{source_module}.{source_name}");
    if let Some(class) = lowering
        .module
        .classes
        .iter()
        .find(|class| class.identity.as_deref() == Some(&canonical_identity))
    {
        return Some(class.type_params.clone());
    }
    if let Some(type_params) = external_defs
        .class_type_params
        .get(source_module)
        .and_then(|classes| classes.get(source_name))
        .cloned()
    {
        return Some(type_params);
    }
    if owner_is_local && source_module == module_name {
        return lowering
            .module
            .classes
            .iter()
            .find(|class| class.name == source_name)
            .map(|class| class.type_params.clone());
    }
    None
}
