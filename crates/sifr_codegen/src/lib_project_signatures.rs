use crate::{module_class_fields, module_func_signatures, HirModule, ModuleFuncSignatures};
use sifr_type_system::Type;
use std::collections::HashMap;

pub(crate) type ProjectClassFields = HashMap<String, HashMap<String, Vec<(String, Type)>>>;

pub(crate) fn project_func_signatures(
    modules: &[(&str, &HirModule)],
) -> HashMap<String, ModuleFuncSignatures> {
    let mut signatures = modules
        .iter()
        .map(|(name, module)| ((*name).to_string(), module_func_signatures(module)))
        .collect::<HashMap<_, _>>();
    for _ in 0..modules.len() {
        let previous = signatures.clone();
        let mut changed = false;
        for (module_name, module) in modules {
            let target = signatures.entry((*module_name).to_string()).or_default();
            for import in &module.imports {
                let Some(source) = previous.get(&import.module) else {
                    continue;
                };
                for name in &import.names {
                    let local = import
                        .aliases
                        .iter()
                        .find(|(original, _)| original == name)
                        .map_or(name.as_str(), |(_, alias)| alias.as_str());
                    if let Some(signature) = source.get(name) {
                        changed |= target.insert(local.to_string(), signature.clone()).as_ref()
                            != Some(signature);
                    }
                    let prefix = format!("{name}::");
                    for (source_name, signature) in source {
                        let Some(method) = source_name.strip_prefix(&prefix) else {
                            continue;
                        };
                        let local_name = format!("{local}::{method}");
                        changed |= target.insert(local_name, signature.clone()).as_ref()
                            != Some(signature);
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    signatures
}

pub(crate) fn project_class_fields(modules: &[(&str, &HirModule)]) -> ProjectClassFields {
    let mut fields = modules
        .iter()
        .map(|(name, module)| ((*name).to_string(), module_class_fields(module)))
        .collect::<ProjectClassFields>();
    for _ in 0..modules.len() {
        let previous = fields.clone();
        let mut changed = false;
        for (module_name, module) in modules {
            let target = fields.entry((*module_name).to_string()).or_default();
            for import in &module.imports {
                let Some(source) = previous.get(&import.module) else {
                    continue;
                };
                for name in &import.names {
                    let local = import
                        .aliases
                        .iter()
                        .find(|(original, _)| original == name)
                        .map_or(name.as_str(), |(_, alias)| alias.as_str());
                    if let Some(class_fields) = source.get(name) {
                        changed |= target
                            .insert(local.to_string(), class_fields.clone())
                            .as_ref()
                            != Some(class_fields);
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    fields
}
