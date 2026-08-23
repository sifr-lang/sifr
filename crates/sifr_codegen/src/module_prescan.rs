use crate::{BUILTIN_ERROR_CLASSES, RustEmitter};
use sifr_ir::HirModule;
use std::collections::{HashMap, HashSet};

impl RustEmitter {
    pub(crate) fn prescan_module_metadata(&mut self, module: &HirModule) {
        self.collect_import_metadata(module);
        self.collect_display_class_metadata(module);
        self.collect_parent_field_metadata(module);
        self.collect_function_signature_metadata(module);
        let mut structural = HashMap::<String, HashSet<String>>::new();
        let mut string_structural = HashMap::<String, HashSet<String>>::new();
        let mut static_program = HashMap::<String, HashSet<String>>::new();
        let mut method_slots = HashMap::<String, HashSet<String>>::new();
        let mut context = HashMap::<String, HashSet<String>>::new();
        for (owner, bounds) in &module.type_param_bounds {
            for (name, values) in bounds {
                let [bound] = values.as_slice() else {
                    continue;
                };
                let destination = match bound.as_str() {
                    "Structural" => Some(&mut structural),
                    "StringStructural" => {
                        string_structural
                            .entry(owner.clone())
                            .or_default()
                            .insert(name.clone());
                        Some(&mut structural)
                    }
                    "StaticProgram" => Some(&mut static_program),
                    "MethodSlots" => {
                        method_slots
                            .entry(owner.clone())
                            .or_default()
                            .insert(name.clone());
                        Some(&mut static_program)
                    }
                    "Context" => Some(&mut context),
                    _ => None,
                };
                if let Some(destination) = destination {
                    destination
                        .entry(owner.clone())
                        .or_default()
                        .insert(name.clone());
                }
            }
        }
        self.structural_type_params = structural;
        self.string_structural_type_params = string_structural;
        self.static_program_type_params = static_program;
        self.method_slot_type_params = method_slots;
        self.context_type_params = context;
    }

    pub(crate) fn collect_import_metadata(&mut self, module: &HirModule) {
        for import in &module.imports {
            if !import.module.starts_with("sifr.") && !import.module.starts_with("_sifr.") {
                for name in &import.names {
                    let local = import
                        .aliases
                        .iter()
                        .find(|(source, _)| source == name)
                        .map_or(name, |(_, alias)| alias);
                    self.imported_project_functions.insert(local.clone());
                }
                continue;
            }

            self.used_stdlib_modules.insert(import.module.clone());
            let names_set = self
                .imported_stdlib_names
                .entry(import.module.clone())
                .or_default();
            for name in &import.names {
                names_set.insert(name.clone());
            }
        }
    }

    pub(crate) fn collect_display_class_metadata(&mut self, module: &HirModule) {
        for class in &module.classes {
            let has_auto_display =
                Self::class_emits_display(class, module, &mut std::collections::HashSet::new());
            if class.is_error_type
                || class.newtype_inner.is_some()
                || class
                    .operator_impls
                    .iter()
                    .any(|(name, _)| name == "__str__")
                || has_auto_display
            {
                self.display_classes.insert(class.name.clone());
            }
        }
        // Built-in error types all have Display impls (formatting self.message).
        for &error_name in BUILTIN_ERROR_CLASSES {
            self.display_classes.insert(error_name.to_string());
        }
    }

    pub(crate) fn collect_parent_field_metadata(&mut self, module: &HirModule) {
        for class in &module.classes {
            if let Some(ref parent_name) = class.parent_class {
                let parent_field_names = class
                    .parent_type
                    .as_ref()
                    .and_then(|parent| match parent.resolve_alias() {
                        sifr_type_system::Type::Class { fields, .. } => Some(
                            fields
                                .iter()
                                .map(|(name, _)| name.clone())
                                .collect::<HashSet<_>>(),
                        ),
                        _ => None,
                    })
                    .or_else(|| {
                        module
                            .classes
                            .iter()
                            .find(|candidate| candidate.name == *parent_name)
                            .map(|parent_class| {
                                parent_class
                                    .fields
                                    .iter()
                                    .map(|(name, _)| name.clone())
                                    .collect::<HashSet<_>>()
                            })
                    });
                if let Some(parent_field_names) = parent_field_names {
                    self.parent_fields.insert(
                        class.name.clone(),
                        (parent_name.clone(), parent_field_names),
                    );
                }
            }
        }
    }

    pub(crate) fn collect_function_signature_metadata(&mut self, module: &HirModule) {
        for func in &module.functions {
            let params = func
                .params
                .iter()
                .map(|param| (param.ty.clone(), param.convention))
                .collect::<Vec<_>>();
            self.func_signatures
                .insert(func.name.clone(), (params, func.return_type.clone()));
        }
        for class in &module.classes {
            let emitted_class_name = self.current_module_name.as_deref().map_or_else(
                || sifr_type_system::source_class_rust_name(&class.name),
                |module| {
                    if module.starts_with("sifr.") || module.starts_with("_sifr.") {
                        sifr_type_system::stdlib_class_rust_name(module, &class.name)
                    } else {
                        sifr_type_system::source_class_rust_name(&class.name)
                    }
                },
            );
            for method in &class.methods {
                let params = method
                    .params
                    .iter()
                    .map(|param| {
                        let convention = if method.name == "new" {
                            sifr_type_system::ParamConvention::own()
                        } else {
                            param.convention
                        };
                        (param.ty.clone(), convention)
                    })
                    .collect::<Vec<_>>();
                let signature = (params, method.return_type.clone());
                self.func_signatures.insert(
                    format!("{}::{}", class.name, method.name),
                    signature.clone(),
                );
                self.func_signatures
                    .insert(format!("{emitted_class_name}::{}", method.name), signature);
            }
        }
    }
}
