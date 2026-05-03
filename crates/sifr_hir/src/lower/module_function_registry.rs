use super::LowerCtx;
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use std::collections::HashSet;

#[derive(Default)]
pub(super) struct ModuleFunctionRegistry {
    seen_module_decls: HashSet<String>,
    seen_lowered_defs: HashSet<String>,
}

impl ModuleFunctionRegistry {
    pub(super) fn note_module_decl(
        &mut self,
        function_name: &str,
        name_range: TextRange,
        ctx: &mut LowerCtx,
    ) -> bool {
        if self.seen_module_decls.insert(function_name.to_string()) {
            return true;
        }
        ctx.error_with_code_at(
            DiagnosticCode::NAME_DUPLICATE_DEFINITION,
            format!("duplicate function definition in module: '{function_name}'"),
            name_range,
        );
        false
    }

    pub(super) fn note_lowering(&mut self, function_name: &str) -> bool {
        self.seen_lowered_defs.insert(function_name.to_string())
    }
}
