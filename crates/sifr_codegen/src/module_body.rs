use crate::{RustEmitter, RustItem};
use sifr_hir::HirModule;

impl RustEmitter {
    pub(super) fn emit_module_body(
        &mut self,
        module: &HirModule,
        module_public: bool,
        test_mode: bool,
    ) {
        self.emit_module_classes(module, module_public);
        self.emit_module_functions(module, module_public, test_mode);
    }

    fn emit_module_classes(&mut self, module: &HirModule, module_public: bool) {
        // Emit class definitions first (structs + impls).
        for class in &module.classes {
            let output_len = self.output.len();
            self.emit_class(class, module, module_public);
            self.drain_emitted_output_item(output_len);
        }
    }

    fn emit_module_functions(&mut self, module: &HirModule, module_public: bool, test_mode: bool) {
        for func in &module.functions {
            let output_len = self.output.len();
            self.emit_function(func, module_public, test_mode);
            self.drain_emitted_output_item(output_len);
        }
    }

    fn drain_emitted_output_item(&mut self, output_len: usize) {
        if self.output.len() <= output_len {
            return;
        }
        let emitted = self.output[output_len..].to_string();
        self.output.truncate(output_len);
        if !emitted.trim().is_empty() {
            self.body_items.push(RustItem::RawCode(emitted));
        }
    }
}
