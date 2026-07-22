use super::implementation::{AnalysisHost, QueryResult};
use crate::snapshot::AnalysisQueryKind;

impl AnalysisHost {
    /// Return the compiler-owned Python interop plan for the current workspace
    /// snapshot. LSP consumers probe this plan instead of reparsing decorators
    /// or maintaining an editor-only declaration model.
    pub fn python_interop_plan(&mut self) -> QueryResult<sifr_driver::PythonInteropPlan> {
        let graph = self.context()?.module_graph();
        let mut lowered = Vec::with_capacity(graph.modules.len());
        for node in graph.modules {
            let name = node
                .canonical_path
                .as_path()
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("module")
                .to_string();
            let module = self
                .context_mut()?
                .hir_module_view(node.id)
                .into_value()
                .hir;
            lowered.push((name, module));
        }
        let plan = sifr_driver::interop_build_plan_for_named_modules(
            lowered
                .iter()
                .map(|(name, module)| (Some(name.as_str()), module)),
        )
        .python;
        Ok(self.result(AnalysisQueryKind::PythonInterop, plan))
    }
}
