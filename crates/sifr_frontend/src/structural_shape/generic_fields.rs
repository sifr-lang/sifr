//! Concrete field types selected by finalized adapted generic declarations.

use sifr_lowering::{substitute_type_vars, AdapterFieldPlan, ExternalDefs, HirClass};
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
    external_defs: &ExternalDefs,
) -> Vec<(String, Type)> {
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
                substitute_type_vars(&field.declared_type, &bindings),
            )
        })
        .collect()
}
