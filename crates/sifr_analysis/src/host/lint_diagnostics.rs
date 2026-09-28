//! One latest lint result per live file, owned and released by the analysis host.
use super::AnalysisHost;
use crate::snapshot::AnalysisError;
use sifr_diagnostics::RenderedDiagnostic;
use sifr_frontend::{CacheFamily, CacheKeyFingerprint, FileId, QueryPolicyFingerprint};

pub(super) struct LintCacheEntry {
    fingerprint: CacheKeyFingerprint,
    diagnostics: Vec<RenderedDiagnostic>,
}

impl AnalysisHost {
    pub(super) fn lint_diagnostics(
        &mut self,
        file: FileId,
    ) -> Result<Vec<RenderedDiagnostic>, AnalysisError> {
        let module = self.module_for_file(file)?;
        // Analysis uses the lint engine's default policy, not project lint-path
        // configuration. Keep that existing semantic boundary on both hits and
        // misses; the frontend owns identity and graph invalidation inputs.
        let fingerprint = self.context()?.lint_cache_fingerprint(
            module,
            QueryPolicyFingerprint::default_for_cache_family(CacheFamily::Lint),
        );
        if let Some(entry) = self.lint_cache.get(&file) {
            if entry.fingerprint == fingerprint {
                return Ok(entry.diagnostics.clone());
            }
        }
        let source = self.source_text_for_file(file)?.to_string();
        let path = self
            .context()?
            .path_for_file(file)
            .map(std::path::Path::to_path_buf);
        let parsed = self.context_mut()?.parse_module(module).into_value().parsed;
        let hir = self.context_mut()?.hir_module_view(module).into_value().hir;
        let diagnostics = sifr_lint::lint_frontend_snapshot(
            &source,
            path.as_deref(),
            &sifr_lint::LintOptions::default(),
            Some(&parsed),
            Some(&hir),
        )
        .diagnostics;
        self.lint_cache.insert(
            file,
            LintCacheEntry {
                fingerprint,
                diagnostics: diagnostics.clone(),
            },
        );
        Ok(diagnostics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_frontend::{DocumentVersion, FrontendInput, FrontendMode, SourcePath, SourceText};

    fn host(path: &str, source: &str) -> AnalysisHost {
        AnalysisHost::open_single_file(
            &sifr_compiler_services::CompilerContext::for_test_tokens(
                crate::compiled_input_tokens(),
                "sifr_analysis-tests",
            ),
            FrontendInput {
                path: SourcePath::new(path),
                source: SourceText::new(source),
                mode: FrontendMode::SingleFile,
            },
        )
        .expect("host loads")
    }

    fn assert_engine_equivalence(host: &mut AnalysisHost, file: FileId) {
        let expected = sifr_lint::lint_source(
            host.source_text_for_file(file).expect("source"),
            host.context().expect("context").path_for_file(file),
            &sifr_lint::LintOptions::default(),
        )
        .diagnostics;
        assert_eq!(host.lint_diagnostics(file).expect("miss"), expected);
        assert_eq!(host.lint_diagnostics(file).expect("hit"), expected);
        assert_eq!(host.lint_cache.len(), 1);
    }

    #[test]
    fn current_result_is_reused_across_version_only_updates_and_replaced_on_edits() {
        let source = "# TODO: inspect\ndef main():\n    configure(True)\n";
        let mut host = host("main.sifr", source);
        let file = host.files()[0];
        assert_engine_equivalence(&mut host, file);
        let identity = host.lint_cache[&file].fingerprint.clone();
        let allocation = host.lint_cache[&file].diagnostics.as_ptr();
        assert!(!host.lint_cache[&file].diagnostics.is_empty());
        host.update_document(file, DocumentVersion::new(2), SourceText::new(source))
            .expect("version update");
        assert_engine_equivalence(&mut host, file);
        assert_eq!(host.lint_cache[&file].fingerprint, identity);
        assert_eq!(host.lint_cache[&file].diagnostics.as_ptr(), allocation);
        for (version, replacement) in [
            (3, "def main():\n    pass\n"),
            (4, "def broken(:\n"),
            (5, source),
        ] {
            host.update_document(
                file,
                DocumentVersion::new(version),
                SourceText::new(replacement),
            )
            .expect("source update");
            assert_engine_equivalence(&mut host, file);
            if replacement != source {
                assert_ne!(host.lint_cache[&file].fingerprint, identity);
            }
        }
    }

    #[test]
    fn e02_hir_policy_and_fix_actions_reuse_current_snapshot() {
        use super::super::text_edits::full_range;
        use crate::{CodeActionContext, DiagnosticId};

        let source = "def configure(a: int, b: int, c: int, d: int, e: int, f: int) -> int:  \n    return a\n";
        let mut host = host("main.sifr", source);
        let file = host.files()[0];
        let current = host.snapshot();
        let diagnostics = host.lint_diagnostics(file).expect("lint miss");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "SIFR-LINT-0007")
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "SIFR-LINT-0004")
        );
        let allocation = host.lint_cache[&file].diagnostics.as_ptr();
        let fingerprint = host.lint_cache[&file].fingerprint.clone();

        let actions = host
            .code_actions(
                file,
                full_range(source).expect("range"),
                &CodeActionContext {
                    diagnostics: vec![DiagnosticId::policy(
                        "SIFR-LINT-0004",
                        "trailing-whitespace",
                    )],
                },
            )
            .expect("actions")
            .into_value();
        assert!(
            actions
                .iter()
                .any(|action| action.kind == "quickfix.sifr.applySafeFix")
        );
        assert_eq!(host.lint_cache[&file].diagnostics.as_ptr(), allocation);
        let edit = host
            .safe_fix_all_action(file)
            .expect("fix all")
            .into_value();
        assert_eq!(host.lint_cache[&file].diagnostics.as_ptr(), allocation);
        let replacement = &edit.edits[0].edits[0].replacement;
        assert_eq!(replacement, &source.replace(":  ", ":"));

        host.update_document(file, DocumentVersion::new(2), SourceText::new(replacement))
            .expect("apply fix as new revision");
        assert!(!host.is_snapshot_current(&current));
        let new_diagnostics = host.lint_diagnostics(file).expect("new snapshot lint");
        assert!(
            !new_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "SIFR-LINT-0004")
        );
        assert!(
            new_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "SIFR-LINT-0007")
        );
        assert_ne!(host.lint_cache[&file].fingerprint, fingerprint);
        let stale_actions = host
            .code_actions(
                file,
                full_range(replacement).expect("new range"),
                &CodeActionContext {
                    diagnostics: vec![DiagnosticId::policy(
                        "SIFR-LINT-0004",
                        "trailing-whitespace",
                    )],
                },
            )
            .expect("current actions")
            .into_value();
        assert!(
            !stale_actions
                .iter()
                .any(|action| action.kind.starts_with("quickfix.sifr"))
        );
    }

    #[test]
    fn e02_moved_rule_does_not_offer_action_on_old_line() {
        use super::super::text_edits::full_range;
        use crate::{CodeActionContext, DiagnosticId};
        use ruff_text_size::{TextRange, TextSize};

        let mut host = host("main.sifr", "def main():  \n    value: int = 1\n");
        let file = host.files()[0];
        host.lint_diagnostics(file).expect("original lint");
        let current = "def main():\n    value: int = 1  \n";
        host.update_document(file, DocumentVersion::new(2), SourceText::new(current))
            .expect("move rule to a different line");
        let context = CodeActionContext {
            diagnostics: vec![DiagnosticId::policy(
                "SIFR-LINT-0004",
                "trailing-whitespace",
            )],
        };
        let old_line = host
            .code_actions(
                file,
                TextRange::new(TextSize::new(0), TextSize::new(0)),
                &context,
            )
            .expect("old line actions")
            .into_value();
        assert!(
            !old_line
                .iter()
                .any(|action| action.kind == "quickfix.sifr.suppress")
        );

        let current_actions = host
            .code_actions(file, full_range(current).expect("current range"), &context)
            .expect("current actions")
            .into_value();
        let suppression = current_actions
            .iter()
            .find(|action| action.kind == "quickfix.sifr.suppress")
            .and_then(|action| action.edit.as_ref())
            .expect("current suppression");
        assert_eq!(
            usize::from(suppression.edits[0].edits[0].range.start()),
            current.len() - 1,
        );
    }

    #[test]
    fn e02_suppressed_diagnostic_has_no_safe_fix_action() {
        use super::super::text_edits::full_range;
        use crate::{CodeActionContext, DiagnosticId};

        let source = "def main():  # sifr: ignore[trailing-whitespace]  \n    pass\n";
        let mut host = host("main.sifr", source);
        let file = host.files()[0];
        assert!(
            !host
                .lint_diagnostics(file)
                .expect("lint")
                .iter()
                .any(|diagnostic| diagnostic.code == "SIFR-LINT-0004")
        );
        let actions = host
            .code_actions(
                file,
                full_range(source).expect("range"),
                &CodeActionContext {
                    diagnostics: vec![DiagnosticId::policy(
                        "SIFR-LINT-0004",
                        "trailing-whitespace",
                    )],
                },
            )
            .expect("actions")
            .into_value();
        assert!(
            !actions
                .iter()
                .any(|action| action.kind == "quickfix.sifr.applySafeFix")
        );
        assert!(
            host.safe_fix_all_action(file)
                .expect("fix all")
                .into_value()
                .edits
                .is_empty()
        );
    }

    #[test]
    fn paths_and_explicit_policy_are_part_of_identity_and_reopen_has_no_cache() {
        let source = "# TODO: inspect\ndef main():\n    pass\n";
        let mut first = host("first.sifr", source);
        let mut second = host("second.sifr", source);
        let first_file = first.files()[0];
        let second_file = second.files()[0];
        assert_engine_equivalence(&mut first, first_file);
        assert_engine_equivalence(&mut second, second_file);
        assert_ne!(
            first.lint_cache[&first_file].fingerprint,
            second.lint_cache[&second_file].fingerprint
        );
        let context = first.context().expect("context");
        let module = first.module_for_file(first_file).expect("module");
        assert_ne!(
            first.lint_cache[&first_file].fingerprint,
            context.lint_cache_fingerprint(module, QueryPolicyFingerprint::new("different-policy")),
        );
        drop(first);
        assert!(host("first.sifr", source).lint_cache.is_empty());
    }
}
