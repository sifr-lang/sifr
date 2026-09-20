use super::{
    DiagnosticCode, DocumentVersion, FrontendContext, InvalidationReport, ModuleId,
    ModuleSignature, QueryKind, RenderedDiagnostic, SourceText, UpdatedDocumentInfo,
    WorkspaceDirtyReason, WorkspaceDirtyScope, WorkspaceDirtyScopeReport, diagnostic_with_code,
    module_signature, source_hash,
};

impl FrontendContext {
    pub fn update_module_source(
        &mut self,
        module: ModuleId,
        source: SourceText,
        document_version: Option<DocumentVersion>,
    ) -> Result<InvalidationReport, Vec<RenderedDiagnostic>> {
        let Some(index) = self.module_by_id.get(&module).copied() else {
            return Err(vec![diagnostic_with_code(
                format!("unknown module id {}", module.as_u32()),
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            )]);
        };
        let _ = self.ensure_lowered(module);
        let previously_checked = self.modules[index].lowered.is_some();
        let previous_revision = self.graph_revision;
        let old_hash = self.modules[index].source_hash.clone();
        let new_hash = source_hash(source.as_str());
        let old_version = self.modules[index].document_version;
        let old_signature = self.modules[index].signature.clone();
        let file = self.modules[index].file;
        let path = self.modules[index].path.clone();
        let text_changed = old_hash != new_hash;
        let parsed =
            sifr_syntax::parse_module(source.as_str(), Some(&self.modules[index].module_name));
        let new_signature = parsed.as_ref().map_or_else(
            |_| ModuleSignature::default(),
            |parsed| module_signature(parsed.suite()),
        );
        self.modules[index].source = source;
        self.modules[index].source_hash = new_hash;
        self.modules[index].document_version = document_version;
        self.modules[index].signature = new_signature.clone();
        if text_changed {
            self.modules[index].source_file_view = None;
            self.source_revision.0 += 1;
            self.source_map_cache = None;
            self.module_graph_cache = None;
        }

        let mut invalidated_modules = Vec::new();
        let mut invalidated_queries = Vec::new();
        let dirty_scope_report = if text_changed {
            let imports_changed = old_signature.imports != new_signature.imports;
            let mut exports_changed = old_signature.exports != new_signature.exports
                || old_signature.semantic_body != new_signature.semantic_body;
            let parse_failed = parsed.is_err();
            self.clear_module_caches(&[module], &[module]);
            let _ = self.ensure_lowered(module);
            let still_checked = self.modules[index].lowered.is_some();
            exports_changed |= !previously_checked || !still_checked;
            let can_replace_module = previously_checked
                && still_checked
                && Self::signatures_can_replace_module_in_project(
                    &old_signature,
                    &new_signature,
                    parse_failed,
                );
            invalidated_modules = if can_replace_module {
                vec![module]
            } else {
                self.reverse_dependency_closure(module)
            };
            let importers: Vec<_> = invalidated_modules
                .iter()
                .copied()
                .filter(|candidate| *candidate != module)
                .collect();
            self.clear_module_caches(&importers, &[]);
            if !can_replace_module {
                self.external_defs = self.base_external_defs.clone();
                self.rebuild_external_defs_from_lowered();
                self.lowering_modules.clear();
                self.graph_revision.0 += 1;
                self.rebuild_edges();
            }
            invalidated_queries.extend([
                QueryKind::Parse,
                QueryKind::Lower,
                QueryKind::TypeCheck,
                QueryKind::ModuleDiagnostics,
                QueryKind::ProjectDiagnostics,
                QueryKind::ModuleAnalysis,
                QueryKind::ProjectAnalysis,
            ]);
            let mut reasons = vec![WorkspaceDirtyReason::SourceTextChanged];
            if imports_changed {
                reasons.push(WorkspaceDirtyReason::ImportSignatureChanged);
            }
            if exports_changed {
                reasons.push(WorkspaceDirtyReason::ExportSignatureChanged);
            }
            if parse_failed {
                reasons.push(WorkspaceDirtyReason::Unknown);
            }
            let scope = if parse_failed || imports_changed {
                WorkspaceDirtyScope::GraphStructure
            } else if exports_changed {
                WorkspaceDirtyScope::ReverseDependencies { path }
            } else {
                WorkspaceDirtyScope::OneModule { path }
            };
            WorkspaceDirtyScopeReport::new(scope, reasons)
        } else {
            WorkspaceDirtyScopeReport::new(
                WorkspaceDirtyScope::None,
                vec![WorkspaceDirtyReason::DocumentVersionOnly],
            )
        };
        self.reuse_caches.prune_unshared();

        Ok(InvalidationReport {
            previous_revision,
            next_revision: self.graph_revision,
            invalidated_modules,
            invalidated_queries,
            updated_documents: vec![UpdatedDocumentInfo {
                file,
                old_version,
                new_version: document_version,
                text_changed,
            }],
            dirty_scope_report,
        })
    }
}
