use sifr_lowering::{
    ExternalDefs, HirClass, HirFunction, RustInteropDecoratorKind,
    canonicalize_user_export_function_type,
};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

pub(crate) fn exported_function_type(
    function: &HirFunction,
    local_classes: &HashMap<String, String>,
) -> FunctionType {
    let function_type = FunctionType {
        receiver: function.receiver,
        params: function
            .params
            .iter()
            .map(|param| (param.name.clone(), param.ty.clone(), param.convention))
            .collect(),
        return_type: Box::new(function.return_type.clone()),
    };
    canonicalize_user_export_function_type(&function_type, local_classes)
}

#[derive(Default)]
pub(crate) struct RustCallbackExports(HashMap<String, Vec<usize>>);

impl RustCallbackExports {
    pub(crate) fn record_function(&mut self, function: &HirFunction) {
        if !has_threadsafe_callback_contract(function) {
            return;
        }
        self.0.insert(
            function.name.clone(),
            retained_callback_param_indices(function),
        );
    }

    pub(crate) fn record_class(&mut self, class: &HirClass) {
        for method in &class.methods {
            if has_threadsafe_callback_contract(method) {
                self.0.insert(
                    format!("{}.{}", class.name, method.name),
                    retained_callback_param_indices(method),
                );
            }
        }
    }

    pub(crate) fn copy_imported(
        &mut self,
        external_defs: &ExternalDefs,
        module: &str,
        source_name: &str,
        local_name: &str,
    ) {
        if let Some(indices) = external_defs
            .rust_threadsafe_callback_targets
            .get(module)
            .and_then(|module_exports| module_exports.get(source_name))
        {
            self.0.insert(local_name.to_string(), indices.clone());
        }
        let Some(module_exports) = external_defs.rust_threadsafe_callback_targets.get(module)
        else {
            return;
        };
        let source_prefix = format!("{source_name}.");
        for (callable, indices) in module_exports {
            if let Some(method) = callable.strip_prefix(&source_prefix) {
                self.0
                    .insert(format!("{local_name}.{method}"), indices.clone());
            }
        }
    }

    pub(crate) fn store(self, external_defs: &mut ExternalDefs, module: &str) {
        external_defs
            .rust_threadsafe_callback_targets
            .insert(module.to_string(), self.0);
    }
}

fn has_threadsafe_callback_contract(function: &HirFunction) -> bool {
    function
        .rust_interop
        .iter()
        .any(|declaration| declaration.kind == RustInteropDecoratorKind::Callback)
}

fn retained_callback_param_indices(function: &HirFunction) -> Vec<usize> {
    function
        .params
        .iter()
        .enumerate()
        .filter_map(|(index, param)| {
            matches!(param.ty.resolve_alias(), Type::Callable(..)).then_some(index)
        })
        .collect()
}
