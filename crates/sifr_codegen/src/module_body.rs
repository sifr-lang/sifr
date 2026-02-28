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
            self.drain_emitted_output_items(output_len, "module class emission");
        }
    }

    fn emit_module_functions(&mut self, module: &HirModule, module_public: bool, test_mode: bool) {
        for func in &module.functions {
            let output_len = self.output.len();
            self.emit_function(func, module_public, test_mode);
            self.drain_emitted_output_items(output_len, "module function emission");
        }
    }

    fn drain_emitted_output_items(&mut self, output_len: usize, context: &str) {
        if self.output.len() <= output_len {
            return;
        }
        let emitted = self.output[output_len..].to_string();
        self.output.truncate(output_len);
        self.push_syn_items_from_source(&emitted, context);
    }

    pub(crate) fn push_syn_items_from_source(&mut self, source: &str, context: &str) {
        if source.trim().is_empty() {
            return;
        }
        let parsed = syn::parse_file(source).unwrap_or_else(|err| {
            panic!("failed to parse emitted Rust items in {context}: {err}; source:\n{source}")
        });
        for item in parsed.items {
            let rendered = prettyplease::unparse(&syn::File {
                shebang: None,
                attrs: vec![],
                items: vec![item],
            });
            self.body_items.push(RustItem::SynItem(rendered));
        }
    }
}
