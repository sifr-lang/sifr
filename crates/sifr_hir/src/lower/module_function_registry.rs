use super::LowerCtx;
use std::collections::HashSet;

#[derive(Default)]
pub(super) struct ModuleFunctionRegistry {
    seen_module_decls: HashSet<String>,
    seen_lowered_defs: HashSet<String>,
}

impl ModuleFunctionRegistry {
    pub(super) fn note_module_decl(&mut self, function_name: &str, ctx: &mut LowerCtx) -> bool {
        if self.seen_module_decls.insert(function_name.to_string()) {
            return true;
        }
        ctx.error(format!(
            "duplicate function definition in module: '{function_name}'"
        ));
        false
    }

    pub(super) fn note_lowering(&mut self, function_name: &str) -> bool {
        self.seen_lowered_defs.insert(function_name.to_string())
    }
}
