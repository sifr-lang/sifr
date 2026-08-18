//! Canonical export and re-export storage for typed descriptor declarations.

use crate::module_export_storage::replace_module_entry;
use sifr_lowering::{canonicalize_user_export_type, ExternalDefs, HirModule, LoweringResult};
use sifr_type_system::Type;
use std::collections::HashMap;

pub(crate) fn add_aliases(
    lowering: &LoweringResult,
    local_classes: &HashMap<String, String>,
    exports: &mut HashMap<String, Type>,
) {
    exports.extend(
        lowering
            .type_aliases
            .iter()
            .filter(|(name, _)| !name.starts_with('_'))
            .map(|(name, ty)| {
                (
                    name.clone(),
                    canonicalize_user_export_type(ty, local_classes),
                )
            }),
    );
}

pub(crate) fn store(
    module_name: &str,
    module: &HirModule,
    lowering: &LoweringResult,
    external_defs: &mut ExternalDefs,
) {
    let mut providers = lowering
        .class_adapter_providers
        .iter()
        .filter(|declaration| {
            crate::query_diagnostics::should_export_callable(module_name, &declaration.function)
        })
        .map(|declaration| (declaration.function.clone(), declaration.clone()))
        .collect::<HashMap<_, _>>();
    let mut functions = lowering
        .descriptor_functions
        .iter()
        .filter(|declaration| {
            crate::query_diagnostics::should_export_callable(module_name, &declaration.function)
        })
        .map(|declaration| (declaration.function.clone(), declaration.clone()))
        .collect::<HashMap<_, _>>();

    for import in &module.imports {
        for name in &import.names {
            let local_name = import
                .aliases
                .iter()
                .find(|(original, _)| original == name)
                .map_or_else(|| name.clone(), |(_, alias)| alias.clone());
            if local_name.starts_with('_') {
                continue;
            }
            if let Some(declaration) = external_defs
                .class_adapter_providers
                .get(&import.module)
                .and_then(|exports| exports.get(name))
            {
                providers.insert(local_name.clone(), declaration.clone());
            }
            if let Some(declaration) = external_defs
                .descriptor_functions
                .get(&import.module)
                .and_then(|exports| exports.get(name))
            {
                functions.insert(local_name, declaration.clone());
            }
        }
    }

    replace_module_entry(
        &mut external_defs.class_adapter_providers,
        module_name,
        providers,
        HashMap::is_empty,
    );
    replace_module_entry(
        &mut external_defs.descriptor_functions,
        module_name,
        functions,
        HashMap::is_empty,
    );
    replace_module_entry(
        &mut external_defs.declaration_descriptors,
        module_name,
        lowering.declaration_descriptors.clone(),
        Vec::is_empty,
    );
}
