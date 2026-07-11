use crate::{helpers::is_auto_display_type, RustEmitter, BUILTIN_ERROR_CLASSES};
use sifr_ir::HirModule;
use std::collections::HashSet;

impl RustEmitter {
    pub(crate) fn prescan_module_metadata(&mut self, module: &HirModule) {
        self.collect_import_metadata(module);
        self.collect_display_class_metadata(module);
        self.collect_parent_field_metadata(module);
        self.collect_function_signature_metadata(module);
    }

    pub(crate) fn collect_import_metadata(&mut self, module: &HirModule) {
        for import in &module.imports {
            if !import.module.starts_with("sifr.") && !import.module.starts_with("_sifr.") {
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
            let has_auto_display = !class.fields.is_empty()
                && !class.is_protocol()
                && !class
                    .operator_impls
                    .iter()
                    .any(|(name, _)| name == "__str__" || name == "__repr__")
                && class.fields.iter().all(|(_, ty)| is_auto_display_type(ty));
            if class.is_error_type
                || class.newtype_inner.is_some()
                || class
                    .operator_impls
                    .iter()
                    .any(|(name, _)| name == "__str__" || name == "__repr__")
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
                if let Some(parent_class) = module
                    .classes
                    .iter()
                    .find(|candidate| candidate.name == *parent_name)
                {
                    let parent_field_names: HashSet<String> = parent_class
                        .fields
                        .iter()
                        .map(|(name, _)| name.clone())
                        .collect();
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
                self.func_signatures.insert(
                    format!("{}::{}", class.name, method.name),
                    (params, method.return_type.clone()),
                );
            }
        }
    }
}
