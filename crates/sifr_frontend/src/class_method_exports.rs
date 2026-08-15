use sifr_lowering::{
    canonicalize_user_export_type, ExternalDefs, HirClass, MethodKind, StructuralMethodExport,
};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct ClassMethodExports {
    instance: HashMap<String, HashSet<String>>,
    consuming: HashMap<String, HashSet<String>>,
}

fn local_instance_methods(
    class: &HirClass,
    known: &HashMap<String, HashSet<String>>,
) -> HashSet<String> {
    let mut methods = class
        .parent_class
        .as_ref()
        .and_then(|parent| known.get(parent))
        .cloned()
        .unwrap_or_default();
    for method in &class.methods {
        if method.name == "new" {
            continue;
        }
        if method.method_kind == MethodKind::Regular {
            methods.insert(method.name.clone());
        } else {
            methods.remove(&method.name);
        }
    }
    methods
}

fn imported_instance_methods(
    external_defs: &ExternalDefs,
    module: &str,
    class_name: &str,
) -> HashSet<String> {
    external_defs
        .class_instance_methods
        .get(module)
        .and_then(|classes| classes.get(class_name))
        .cloned()
        .unwrap_or_default()
}

pub(crate) fn exported_structural_methods(
    class: &HirClass,
    local_classes: &HashMap<String, String>,
) -> Option<Vec<StructuralMethodExport>> {
    let methods = class
        .methods
        .iter()
        .map(|method| (method.name.as_str(), method))
        .chain(
            class
                .operator_impls
                .iter()
                .map(|(name, method)| (name.as_str(), method)),
        )
        .map(|(name, method)| StructuralMethodExport {
            name: name.to_string(),
            params: method
                .params
                .iter()
                .cloned()
                .map(|mut param| {
                    param.ty = canonicalize_user_export_type(&param.ty, local_classes);
                    param
                })
                .collect(),
            return_type: canonicalize_user_export_type(&method.return_type, local_classes),
            is_async: method.is_async,
            method_kind: method.method_kind,
            receiver: method.receiver,
        })
        .collect::<Vec<_>>();
    (!methods.is_empty()).then_some(methods)
}

impl ClassMethodExports {
    pub(crate) fn record_local(&mut self, class: &HirClass) {
        let methods = local_instance_methods(class, &self.instance);
        self.instance.insert(class.name.clone(), methods);
        let Some(selected) = sifr_lowering::rust_opaque_close_method(&class.rust_interop) else {
            return;
        };
        let consuming = class
            .methods
            .iter()
            .filter(|method| {
                method.name == selected
                    && method
                        .rust_interop
                        .iter()
                        .any(|declaration| declaration.consumes_receiver)
            })
            .map(|method| method.name.clone())
            .collect::<HashSet<_>>();
        if !consuming.is_empty() {
            self.consuming.insert(class.name.clone(), consuming);
        }
    }

    pub(crate) fn record_imported(
        &mut self,
        external_defs: &ExternalDefs,
        module: &str,
        source_name: &str,
        local_name: &str,
    ) {
        let methods = imported_instance_methods(external_defs, module, source_name);
        self.instance.insert(local_name.to_string(), methods);
        if let Some(consuming) = external_defs
            .rust_consuming_methods
            .get(module)
            .and_then(|classes| classes.get(source_name))
            .cloned()
        {
            self.consuming.insert(local_name.to_string(), consuming);
        }
    }

    pub(crate) fn store(self, external_defs: &mut ExternalDefs, module_name: &str) {
        external_defs
            .class_instance_methods
            .insert(module_name.to_string(), self.instance);
        if !self.consuming.is_empty() {
            external_defs
                .rust_consuming_methods
                .insert(module_name.to_string(), self.consuming);
        }
    }
}
