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
