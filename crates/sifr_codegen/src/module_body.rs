use crate::RustEmitter;
use sifr_ir::HirModule;

impl RustEmitter {
    pub(crate) fn emit_module_body(
        &mut self,
        module: &HirModule,
        module_public: bool,
        test_mode: bool,
    ) {
        self.python_opaque_classes = module
            .classes
            .iter()
            .filter_map(|class| {
                class
                    .python_opaque_declaration()
                    .cloned()
                    .map(|declaration| (class.name.clone(), declaration))
            })
            .collect();
        self.python_retained_callback_errors = collect_retained_callback_errors(module);
        if self
            .function_type_param_bounds
            .values()
            .flat_map(std::collections::HashMap::values)
            .flatten()
            .any(|bound| {
                matches!(
                    bound,
                    crate::function_generic_bounds::FunctionTypeParamBound::Trait(name)
                        if name == "__SifrAdd"
                )
            })
        {
            self.body_items
                .extend(crate::ownership_plan::addable_support_items());
        }
        self.emit_module_classes(module, module_public);
        self.emit_module_functions(module, module_public, test_mode);
    }

    pub(crate) fn emit_module_classes(&mut self, module: &HirModule, module_public: bool) {
        for class in &module.classes {
            self.emit_class(class, module, module_public);
        }
    }

    pub(crate) fn emit_module_functions(
        &mut self,
        module: &HirModule,
        module_public: bool,
        test_mode: bool,
    ) {
        for func in &module.functions {
            if func.compiler_intrinsic.is_some() {
                continue;
            }
            self.emit_function(func, module_public, test_mode);
        }
    }
}

fn collect_retained_callback_errors(
    module: &HirModule,
) -> std::collections::HashMap<String, Vec<sifr_type_system::Type>> {
    let mut by_owner = std::collections::HashMap::<String, Vec<sifr_type_system::Type>>::new();
    let functions = module
        .functions
        .iter()
        .chain(module.classes.iter().flat_map(|class| class.methods.iter()));
    for callback in functions
        .flat_map(|function| &function.python_interop)
        .flat_map(|declaration| &declaration.callbacks)
    {
        let (Some(owner), Some(error)) = (&callback.owner_class, &callback.handler_error_type)
        else {
            continue;
        };
        let errors = by_owner.entry(owner.clone()).or_default();
        if !errors.iter().any(|existing| existing == error) {
            errors.push(error.clone());
        }
    }
    by_owner
}
