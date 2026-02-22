use crate::RustEmitter;
use sifr_hir::HirModule;

impl RustEmitter {
    pub(super) fn emit_module_body(&mut self, module: &HirModule, module_public: bool, test_mode: bool) {
        self.emit_module_classes(module, module_public);
        self.emit_module_functions(module, module_public, test_mode);
    }

    fn emit_module_classes(&mut self, module: &HirModule, module_public: bool) {
        // Emit class definitions first (structs + impls).
        for class in &module.classes {
            self.emit_class(class, module, module_public);
            self.output.push('\n');
        }
    }

    fn emit_module_functions(&mut self, module: &HirModule, module_public: bool, test_mode: bool) {
        for (index, func) in module.functions.iter().enumerate() {
            if index > 0 {
                self.output.push('\n');
            }
            self.emit_function(func, module_public, test_mode);
        }
    }
}
