use sifr_lowering::{ExternalDefs, HirModule};
use std::collections::HashMap;

pub(crate) fn should_export_callable(module_name: &str, callable_name: &str) -> bool {
    if module_name == "sifr.math"
        && matches!(callable_name, "dist_impl" | "fsum_impl" | "sumprod_impl")
    {
        return false;
    }
    !callable_name.starts_with('_')
        || matches!(
            (module_name, callable_name),
            (
                "sifr.heapq",
                "_heapify_max" | "_heappop_max" | "_heapreplace_max"
            )
        )
}

pub(crate) type GenericExports = HashMap<String, Vec<String>>;
pub(crate) type BoundExports = HashMap<String, HashMap<String, Vec<String>>>;

pub(crate) fn declared_generic_metadata(
    module_name: &str,
    module: &HirModule,
) -> (GenericExports, BoundExports, HashMap<String, String>) {
    let generics = module
        .generic_functions
        .iter()
        .filter(|(name, _)| should_export_callable(module_name, name))
        .map(|(name, params)| (name.clone(), params.clone()))
        .collect();
    let bounds = module
        .type_param_bounds
        .iter()
        .filter(|(owner, _)| {
            should_export_callable(
                module_name,
                owner.split('.').next().unwrap_or(owner.as_str()),
            )
        })
        .map(|(owner, bounds)| (owner.clone(), bounds.clone()))
        .collect();
    let classes = module
        .classes
        .iter()
        .map(|class| (class.name.clone(), format!("{module_name}.{}", class.name)))
        .collect();
    (generics, bounds, classes)
}

pub(crate) fn reexport_class_aliases(
    module: &HirModule,
    external_defs: &ExternalDefs,
) -> HashMap<String, HashMap<String, String>> {
    let mut aliases = HashMap::new();
    for import in &module.imports {
        let Some(classes) = external_defs.classes.get(&import.module) else {
            continue;
        };
        for name in &import.names {
            if !classes.contains_key(name) {
                continue;
            }
            let local = import
                .aliases
                .iter()
                .find(|(source, _)| source == name)
                .map_or_else(|| name.clone(), |(_, local)| local.clone());
            aliases
                .entry(import.module.clone())
                .or_insert_with(HashMap::new)
                .insert(name.clone(), local);
        }
    }
    aliases
}

pub(crate) fn copy_function_generic_metadata(
    external_defs: &ExternalDefs,
    source_module: &str,
    source_name: &str,
    local_name: &str,
    generic_exports: &mut HashMap<String, Vec<String>>,
    bound_exports: &mut HashMap<String, HashMap<String, Vec<String>>>,
) {
    if let Some(type_vars) = external_defs
        .generic_functions
        .get(source_module)
        .and_then(|module_generics| module_generics.get(source_name))
    {
        generic_exports.insert(local_name.to_string(), type_vars.clone());
    }
    if let Some(bounds) = external_defs
        .type_param_bounds
        .get(source_module)
        .and_then(|module_bounds| module_bounds.get(source_name))
    {
        bound_exports.insert(local_name.to_string(), bounds.clone());
    }
}

pub(crate) fn copy_class_generic_metadata(
    external_defs: &ExternalDefs,
    source_module: &str,
    source_name: &str,
    local_name: &str,
    class_type_params: &mut HashMap<String, Vec<String>>,
    generic_exports: &mut HashMap<String, Vec<String>>,
    bound_exports: &mut HashMap<String, HashMap<String, Vec<String>>>,
) {
    if let Some(type_params) = external_defs
        .class_type_params
        .get(source_module)
        .and_then(|module_params| module_params.get(source_name))
    {
        class_type_params.insert(local_name.to_string(), type_params.clone());
        generic_exports.insert(local_name.to_string(), type_params.clone());
    }
    let Some(module_bounds) = external_defs.type_param_bounds.get(source_module) else {
        return;
    };
    let prefix = format!("{source_name}.");
    for (owner, bounds) in module_bounds {
        if let Some(method) = owner.strip_prefix(&prefix) {
            bound_exports.insert(format!("{local_name}.{method}"), bounds.clone());
        }
    }
}
