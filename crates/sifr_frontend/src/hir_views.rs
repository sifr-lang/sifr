use crate::{FrontendContext, LoweredModuleView, ModuleId, QueryResult};

impl FrontendContext {
    pub fn hir_module_view(&mut self, module: ModuleId) -> QueryResult<LoweredModuleView> {
        self.lower_module(module)
    }
}
