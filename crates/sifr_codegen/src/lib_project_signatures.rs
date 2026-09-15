use crate::{HirModule, ModuleFuncSignatures};
use sifr_type_system::{ParamConvention, Type};
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

fn module_func_signatures(module: &HirModule) -> ModuleFuncSignatures {
    let mut signatures = HashMap::new();
    for function in &module.functions {
        let params = function
            .params
            .iter()
            .map(|param| (param.ty.clone(), param.convention))
            .collect::<Vec<_>>();
        signatures.insert(
            function.name.clone(),
            (params, function.return_type.clone()),
        );
    }
    for class in &module.classes {
        let mut has_constructor = false;
        for method in &class.methods {
            let params = method
                .params
                .iter()
                .map(|param| {
                    let convention = if method.name == "new" {
                        ParamConvention::own()
                    } else {
                        param.convention
                    };
                    (param.ty.clone(), convention)
                })
                .collect::<Vec<_>>();
            signatures.insert(
                format!("{}::{}", class.name, method.name),
                (params, method.return_type.clone()),
            );
            has_constructor |= method.name == "new";
        }
        if !has_constructor {
            let params = class
                .fields
                .iter()
                .map(|(_, ty)| (ty.clone(), ParamConvention::own()))
                .collect();
            signatures.insert(
                format!("{}::new", class.name),
                (
                    params,
                    Type::Class {
                        identity: None,
                        type_args: class
                            .type_params
                            .iter()
                            .cloned()
                            .map(Type::TypeVar)
                            .collect(),
                        name: class.name.clone(),
                        fields: class.fields.clone(),
                        methods: Vec::new(),
                        parent_class: class.semantic_parent_chain(),
                    },
                ),
            );
        }
    }
    signatures
}

fn module_class_fields(module: &HirModule) -> HashMap<String, Vec<(String, Type)>> {
    module
        .classes
        .iter()
        .map(|class| (class.name.clone(), class.fields.clone()))
        .collect()
}
