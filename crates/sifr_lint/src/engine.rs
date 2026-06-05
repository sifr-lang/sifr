use crate::{LintOptions, LintResult, RuleMetadata, RULES};
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use sifr_frontend::{DiskSourceProvider, SourceProvider};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LintPhase {
    FileDiscovery,
    TokenTrivia,
    PhysicalLine,
    SyntaxNode,
    StatementRange,
    Hir,
    Workspace,
    SuppressionFiltering,
    PerFileIgnoreFiltering,
    FixFiltering,
    Sorting,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhaseExecution {
    pub phase: LintPhase,
    pub ran: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LintRun {
    pub result: LintResult,
    pub phases: Vec<PhaseExecution>,
}

pub struct LintRunner<'a> {
    options: &'a LintOptions,
}

impl<'a> LintRunner<'a> {
    pub fn new(options: &'a LintOptions) -> Self {
        Self { options }
    }

    pub fn run_source(&self, source: &str, file: Option<&Path>) -> LintRun {
        let mut phases = self.phase_plan(file);
        let mut diagnostics = Vec::new();
        let mut suppressions =
            crate::ParserAwareSuppressions::new(source, self.options.ignore_suppressions);
        diagnostics.extend(crate::suppression_shape_diagnostics(
            &suppressions,
            file,
            source,
            self.options,
        ));
        if mark_phase(&mut phases, LintPhase::PhysicalLine) {
            diagnostics.extend(crate::lint_physical_line_rules(
                source,
                file,
                self.options,
                &mut suppressions,
            ));
        }
        let parse_context = file.map(|path| path.display().to_string());
        let parsed = if parse_dependent_phase_ran(&phases) {
            sifr_syntax::parse_module(source, parse_context.as_deref()).ok()
        } else {
            None
        };
        if mark_phase(&mut phases, LintPhase::TokenTrivia) {
            if let Some(parsed) = parsed.as_ref() {
                diagnostics.extend(crate::rules::todo_comment::lint(
                    parsed.tokens(),
                    source,
                    file,
                    self.options,
                    &mut suppressions,
                ));
            }
        }
        if mark_phase(&mut phases, LintPhase::SyntaxNode) {
            if let Some(parsed) = parsed.as_ref() {
                diagnostics.extend(crate::rules::boolean_positional_argument::lint(
                    parsed.suite(),
                    source,
                    file,
                    self.options,
                    &mut suppressions,
                ));
            }
        }
        if mark_phase(&mut phases, LintPhase::Hir) {
            if parsed.is_some() {
                if let Some(lowered) = frontend_hir(source, file) {
                    diagnostics.extend(crate::rules::large_parameter_list::lint(
                        &lowered,
                        source,
                        file,
                        self.options,
                        &mut suppressions,
                    ));
                }
            }
        }
        if mark_phase(&mut phases, LintPhase::Workspace) {
            if let Some(parsed) = parsed.as_ref() {
                diagnostics.extend(crate::rules::duplicate_import::lint(
                    parsed.suite(),
                    source,
                    file,
                    self.options,
                    &mut suppressions,
                ));
            }
        }
        diagnostics.extend(crate::unused_suppression_diagnostics(
            &suppressions,
            file,
            source,
            self.options,
        ));
        if mark_phase(&mut phases, LintPhase::Sorting) {
            diagnostics.sort_by_key(crate::diagnostic_order_key);
        }
        LintRun {
            result: LintResult { diagnostics },
            phases,
        }
    }

    pub fn run_paths(&self, paths: &[PathBuf]) -> Result<LintRun, Vec<RenderedDiagnostic>> {
        let mut phases = empty_phase_plan();
        set_phase(&mut phases, LintPhase::FileDiscovery, true);
        let files = crate::collect_sifr_files_for_targets(paths, self.options)?;
        let mut diagnostics = Vec::new();
        let mut provider = DiskSourceProvider::new();
        for file in files {
            let source = provider.read_file(&file).map_err(|err| {
                vec![crate::diagnostic(
                    DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                    format!("could not read file {}: {err}", file.display()),
                    [("path", file.display().to_string())],
                    Vec::new(),
                    None,
                )]
            })?;
            let file_run = self.run_source(source.as_str(), Some(&file));
            merge_phases(&mut phases, &file_run.phases);
            diagnostics.extend(file_run.result.diagnostics);
        }
        if mark_phase(&mut phases, LintPhase::Sorting) {
            diagnostics.sort_by_key(crate::diagnostic_order_key);
        }
        Ok(LintRun {
            result: LintResult { diagnostics },
            phases,
        })
    }

    pub fn phase_plan(&self, file: Option<&Path>) -> Vec<PhaseExecution> {
        all_phases()
            .into_iter()
            .map(|phase| PhaseExecution {
                phase,
                ran: self.phase_has_enabled_rules(phase, file),
            })
            .collect()
    }

    fn phase_has_enabled_rules(&self, phase: LintPhase, file: Option<&Path>) -> bool {
        if self.options.mode == crate::DiagnosticMode::Off {
            return false;
        }
        match phase {
            LintPhase::FileDiscovery | LintPhase::StatementRange => false,
            LintPhase::FixFiltering => enabled_rules(file, self.options)
                .any(|rule| crate::fix_rule_allowed(rule.id, self.options)),
            LintPhase::PhysicalLine
            | LintPhase::TokenTrivia
            | LintPhase::SyntaxNode
            | LintPhase::Hir
            | LintPhase::Workspace => {
                enabled_rules(file, self.options).any(|rule| rule_phase(rule) == phase)
            }
            LintPhase::SuppressionFiltering => {
                !self.options.ignore_suppressions
                    && enabled_rules(file, self.options).next().is_some()
            }
            LintPhase::PerFileIgnoreFiltering => !self.options.per_file_ignores.is_empty(),
            LintPhase::Sorting => enabled_rules(file, self.options).next().is_some(),
        }
    }
}

fn all_phases() -> [LintPhase; 11] {
    [
        LintPhase::FileDiscovery,
        LintPhase::TokenTrivia,
        LintPhase::PhysicalLine,
        LintPhase::SyntaxNode,
        LintPhase::StatementRange,
        LintPhase::Hir,
        LintPhase::Workspace,
        LintPhase::SuppressionFiltering,
        LintPhase::PerFileIgnoreFiltering,
        LintPhase::FixFiltering,
        LintPhase::Sorting,
    ]
}

fn empty_phase_plan() -> Vec<PhaseExecution> {
    all_phases()
        .into_iter()
        .map(|phase| PhaseExecution { phase, ran: false })
        .collect()
}

fn enabled_rules<'a>(
    file: Option<&'a Path>,
    options: &'a LintOptions,
) -> impl Iterator<Item = &'static RuleMetadata> + 'a {
    RULES
        .iter()
        .filter(move |rule| crate::rule_enabled(rule.id, file, options))
}

fn rule_phase(rule: &RuleMetadata) -> LintPhase {
    match rule.id {
        "todo-comment" => LintPhase::TokenTrivia,
        "boolean-positional-argument" => LintPhase::SyntaxNode,
        "large-parameter-list" => LintPhase::Hir,
        "duplicate-import" => LintPhase::Workspace,
        _ => LintPhase::PhysicalLine,
    }
}

fn parse_dependent_phase_ran(phases: &[PhaseExecution]) -> bool {
    [
        LintPhase::TokenTrivia,
        LintPhase::SyntaxNode,
        LintPhase::Hir,
        LintPhase::Workspace,
    ]
    .into_iter()
    .any(|phase| mark_phase_readonly(phases, phase))
}

fn frontend_hir(source: &str, file: Option<&Path>) -> Option<sifr_ir::HirModule> {
    let path = file.unwrap_or_else(|| Path::new("main.sifr"));
    let input = sifr_frontend::FrontendInput {
        path: sifr_frontend::SourcePath::new(path),
        source: sifr_frontend::SourceText::new(source),
        mode: sifr_frontend::FrontendMode::SingleFile,
    };
    let mut context = sifr_frontend::FrontendContext::load_single_file(input).ok()?;
    let module = context.module_graph().entrypoint;
    Some(context.hir_module_view(module).into_value().hir)
}

fn set_phase(phases: &mut [PhaseExecution], phase: LintPhase, ran: bool) {
    if let Some(execution) = phases.iter_mut().find(|execution| execution.phase == phase) {
        execution.ran = ran;
    }
}

fn mark_phase(phases: &mut [PhaseExecution], phase: LintPhase) -> bool {
    mark_phase_readonly(phases, phase)
}

fn mark_phase_readonly(phases: &[PhaseExecution], phase: LintPhase) -> bool {
    phases
        .iter()
        .find(|execution| execution.phase == phase)
        .is_some_and(|execution| execution.ran)
}

fn merge_phases(target: &mut [PhaseExecution], source: &[PhaseExecution]) {
    for source_execution in source {
        if source_execution.ran {
            set_phase(target, source_execution.phase, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn physical_line_phase_is_skipped_when_all_rules_are_disabled() {
        let options = LintOptions {
            select: Vec::new(),
            ..LintOptions::default()
        };
        let run = LintRunner::new(&options).run_source("def main():  \n", None);
        assert!(
            !run.phases
                .iter()
                .find(|execution| execution.phase == LintPhase::PhysicalLine)
                .unwrap()
                .ran
        );
        assert!(run.result.diagnostics.is_empty());
    }

    #[test]
    fn physical_line_phase_runs_when_a_physical_rule_is_enabled() {
        let run = LintRunner::new(&LintOptions::default()).run_source("def main():  \n", None);
        assert!(
            run.phases
                .iter()
                .find(|execution| execution.phase == LintPhase::PhysicalLine)
                .unwrap()
                .ran
        );
        assert_eq!(run.result.diagnostics.len(), 1);
    }

    #[test]
    fn all_rule_phases_are_skipped_when_diagnostics_are_off() {
        let options = LintOptions {
            mode: crate::DiagnosticMode::Off,
            ..LintOptions::default()
        };
        let run = LintRunner::new(&options).run_source("def main():  \n", None);
        assert!(run.phases.iter().all(|execution| !execution.ran));
        assert!(run.result.diagnostics.is_empty());
    }

    #[test]
    fn file_discovery_phase_runs_for_path_lint() {
        let root = temp_dir("lint_runner_file_discovery");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.sifr"), "def main():  \n").unwrap();
        let run = LintRunner::new(&LintOptions::default())
            .run_paths(&[root.clone()])
            .unwrap();
        assert!(
            run.phases
                .iter()
                .find(|execution| execution.phase == LintPhase::FileDiscovery)
                .unwrap()
                .ran
        );
        assert_eq!(run.result.diagnostics.len(), 1);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn diagnostics_are_sorted_after_phase_execution() {
        let source = "value = 1  \n# sifr: ignore[not-a-rule]\n";
        let run = LintRunner::new(&LintOptions::default()).run_source(source, None);
        let codes = run
            .result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>();
        assert_eq!(codes, ["SIFR-LINT-0004", "SIFR-LINT-0001"]);
    }

    #[test]
    fn invalid_source_still_runs_source_independent_phases() {
        let run = LintRunner::new(&LintOptions::default()).run_source("def main(:  \n", None);
        assert_eq!(run.result.diagnostics.len(), 1);
        assert_eq!(run.result.diagnostics[0].code, "SIFR-LINT-0004");
    }

    #[test]
    fn large_source_smoke_keeps_phase_execution_bounded() {
        let mut source = String::new();
        for _ in 0..2_000 {
            source.push_str("value = 1\n");
        }
        source.push_str("value = 2  \n");
        let run = LintRunner::new(&LintOptions::default()).run_source(&source, None);
        assert_eq!(run.result.diagnostics.len(), 1);
    }

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("sifr_{name}_{unique}"))
    }
}
