use super::implementation::{AnalysisHost, QueryResult};
use crate::snapshot::AnalysisQueryKind;
use sifr_frontend::FileId;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonInteropAnalysisPlan {
    pub plan: sifr_driver::PythonInteropPlan,
    pub module_files: BTreeMap<String, FileId>,
}

impl AnalysisHost {
    /// Return the compiler-owned Python interop plan for the current workspace
    /// snapshot. LSP consumers probe this plan instead of reparsing decorators
    /// or maintaining an editor-only declaration model.
    pub fn python_interop_plan(&mut self) -> QueryResult<PythonInteropAnalysisPlan> {
        let graph = self.context()?.module_graph();
        let mut lowered = Vec::with_capacity(graph.modules.len());
        let mut module_files = BTreeMap::new();
        for node in graph.modules {
            let name = format!("module:{}", node.id.as_u32());
            module_files.insert(name.clone(), node.file);
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
        Ok(self.result(
            AnalysisQueryKind::PythonInterop,
            PythonInteropAnalysisPlan { plan, module_files },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FrontendInput, SourceText};
    use sifr_frontend::FrontendMode;

    #[test]
    fn plan_preserves_declaration_module_file_identity() {
        let source = "from sifr.python import PythonError\n\n@python(math.sqrt)\ndef sqrt(value: float) -> Result[float, PythonError]: ...\n";
        let mut host = AnalysisHost::open_single_file(FrontendInput {
            path: sifr_frontend::SourcePath::new("main.sifr"),
            source: SourceText::new(source),
            mode: FrontendMode::SingleFile,
        })
        .expect("single-file analysis host should load");
        let file = host.files()[0];
        let result = host
            .python_interop_plan()
            .expect("Python interop plan should query")
            .into_value();
        let declaration = result
            .plan
            .declarations
            .first()
            .expect("Python declaration should be planned");
        let module = declaration
            .module_name
            .as_ref()
            .expect("declaration should retain module identity");

        assert_eq!(declaration.function_name, "sqrt");
        assert_eq!(result.module_files.get(module), Some(&file));
        assert_eq!(result.module_files.len(), 1);
    }
}
