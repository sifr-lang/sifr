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
        let diagnostics = sifr_lint::lint_source(
            self.source_text_for_file(file)?,
            self.context()?.path_for_file(file),
            &sifr_lint::LintOptions::default(),
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
        AnalysisHost::open_single_file(FrontendInput {
            path: SourcePath::new(path),
            source: SourceText::new(source),
            mode: FrontendMode::SingleFile,
        })
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
