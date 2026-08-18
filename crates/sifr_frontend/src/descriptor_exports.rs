//! Canonical export and re-export storage for typed descriptor declarations.

use crate::module_export_storage::replace_module_entry;
use sifr_lowering::{canonicalize_user_export_type, ExternalDefs, HirModule, LoweringResult};
use sifr_type_system::Type;
use std::collections::HashMap;

pub fn erase_marker_imports(module: &mut HirModule, external_defs: &ExternalDefs) {
    for import in &mut module.imports {
        let Some(markers) = external_defs.class_adapter_markers.get(&import.module) else {
            continue;
        };
        import.names.retain(|name| !markers.contains_key(name));
        import
            .aliases
            .retain(|(original, _)| !markers.contains_key(original));
    }
    module.imports.retain(|import| !import.names.is_empty());
}

pub(crate) fn add_aliases(
    lowering: &LoweringResult,
    local_classes: &HashMap<String, String>,
    exports: &mut HashMap<String, Type>,
) {
    exports.extend(
        lowering
            .class_adapter_markers
            .iter()
            .filter(|marker| !marker.symbol.starts_with('_'))
            .map(|marker| {
                (
                    marker.symbol.clone(),
                    Type::Class {
                        identity: Some(format!("{}.{}", marker.module, marker.symbol)),
                        type_args: Vec::new(),
                        name: marker.symbol.clone(),
                        fields: Vec::new(),
                        methods: Vec::new(),
                        parent_class: None,
                    },
                )
            }),
    );
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
    let mut markers = lowering
        .class_adapter_markers
        .iter()
        .filter(|marker| !marker.symbol.starts_with('_'))
        .map(|marker| (marker.symbol.clone(), marker.clone()))
        .collect::<HashMap<_, _>>();
    let mut selections = lowering
        .class_adapter_selections
        .iter()
        .filter(|selection| !selection.owner.starts_with('_'))
        .map(|selection| (selection.owner.clone(), selection.clone()))
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
                functions.insert(local_name.clone(), declaration.clone());
            }
            if let Some(marker) = external_defs
                .class_adapter_markers
                .get(&import.module)
                .and_then(|exports| exports.get(name))
            {
                markers.insert(local_name.clone(), marker.clone());
            }
            if let Some(selection) = external_defs
                .class_adapter_selections
                .get(&import.module)
                .and_then(|exports| exports.get(name))
            {
                selections.insert(local_name.clone(), selection.clone());
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
        &mut external_defs.class_adapter_markers,
        module_name,
        markers,
        HashMap::is_empty,
    );
    replace_module_entry(
        &mut external_defs.class_adapter_selections,
        module_name,
        selections,
        HashMap::is_empty,
    );
    replace_module_entry(
        &mut external_defs.descriptor_functions,
        module_name,
        functions,
        HashMap::is_empty,
    );
    replace_module_entry(
        &mut external_defs.applied_adapter_metadata,
        module_name,
        lowering.applied_adapter_metadata.clone(),
        Vec::is_empty,
    );
    replace_module_entry(
        &mut external_defs.declaration_descriptors,
        module_name,
        lowering.declaration_descriptors.clone(),
        Vec::is_empty,
    );
}
