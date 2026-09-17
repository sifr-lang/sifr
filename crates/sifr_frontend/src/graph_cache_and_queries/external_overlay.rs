use super::*;

impl FrontendContext {
    pub(super) fn clear_module_caches(
        &mut self,
        modules: &[ModuleId],
        modules_with_source_changes: &[ModuleId],
    ) {
        let clear_parse_modules = modules_with_source_changes
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        for module in modules {
            let index = self.index_for_module(*module);
            let module_state = &mut self.modules[index];
            if clear_parse_modules.contains(module) {
                module_state.parsed = None;
            }
            self.external_defs
                .remove_module_overlay(&module_state.module_name);
            module_state.lowered = None;
            module_state.diagnostics = None;
            module_state.analysis = None;
        }
        self.reuse_caches.prune_unshared();
    }

    pub(super) fn rebuild_external_defs_from_lowered(&mut self) {
        for module in &self.modules {
            if let Some(lowered) = &module.lowered {
                collect_module_exports(&module.module_name, lowered, &mut self.external_defs);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FrontendInput, FrontendMode};
    fn context(source: &str, defs: ExternalDefs) -> FrontendContext {
        FrontendContext::load_single_file_with_external_defs(
            FrontendInput {
                path: SourcePath::new("main.sifr"),
                source: SourceText::new(source),
                mode: FrontendMode::SingleFile,
            },
            defs,
        )
        .unwrap()
    }
    #[test]
    fn source_overlay_failed_and_deleted_exports_are_invalidated() {
        let mut cx = context(
            "def value() -> int:\n    return 1\n",
            ExternalDefs::default(),
        );
        assert!(
            cx.diagnostics_for_module(ModuleId(0))
                .value()
                .diagnostics
                .is_empty()
        );
        assert!(
            cx.external_defs
                .functions
                .get("main")
                .unwrap()
                .contains_key("value")
        );
        cx.update_module_source(
            ModuleId(0),
            SourceText::new("def value() -> int:\n    return \"bad\"\n"),
            None,
        )
        .unwrap();
        assert!(
            !cx.diagnostics_for_module(ModuleId(0))
                .value()
                .diagnostics
                .is_empty()
        );
        assert!(!cx.external_defs.functions.contains_key("main"));
        cx.update_module_source(
            ModuleId(0),
            SourceText::new("def value() -> int:\n    return 2\n"),
            None,
        )
        .unwrap();
        assert!(
            cx.diagnostics_for_module(ModuleId(0))
                .value()
                .diagnostics
                .is_empty()
        );
        assert!(
            cx.external_defs
                .functions
                .get("main")
                .unwrap()
                .contains_key("value")
        );
        cx.update_module_source(ModuleId(0), SourceText::new(""), None)
            .unwrap();
        assert!(
            cx.diagnostics_for_module(ModuleId(0))
                .value()
                .diagnostics
                .is_empty()
        );
        assert!(
            !cx.external_defs
                .functions
                .get("main")
                .is_some_and(|m| m.contains_key("value"))
        );
    }
    #[test]
    fn source_overlay_projects_share_only_immutable_baseline() {
        let mut baseline = ExternalDefs::default();
        baseline
            .constants
            .entry("values".into())
            .or_default()
            .insert("ANSWER".into(), sifr_type_system::Type::Int);
        baseline.freeze_baseline();
        let mut first = context(
            "from values import ANSWER\ndef first() -> int:\n    return ANSWER\n",
            baseline.clone(),
        );
        let mut second = context(
            "from values import ANSWER\ndef second() -> int:\n    return ANSWER\n",
            baseline.clone(),
        );
        assert!(
            first
                .diagnostics_for_module(ModuleId(0))
                .value()
                .diagnostics
                .is_empty()
        );
        assert!(
            second
                .diagnostics_for_module(ModuleId(0))
                .value()
                .diagnostics
                .is_empty()
        );
        assert!(std::ptr::eq(
            first.external_defs.constants.get("values").unwrap(),
            second.external_defs.constants.get("values").unwrap()
        ));
        assert!(
            !first
                .external_defs
                .functions
                .get("main")
                .unwrap()
                .contains_key("second")
        );
        assert!(
            !second
                .external_defs
                .functions
                .get("main")
                .unwrap()
                .contains_key("first")
        );
        assert!(!baseline.functions.contains_key("main"));
    }
}
