//! Sifr-owned policy-rule, configuration, discovery, and suppression foundation.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

use globset::Glob;
use sifr_diagnostics::{
    DiagnosticArg, DiagnosticCode, DiagnosticSpan, RenderedDiagnostic, Severity,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod config;
mod discovery;
mod engine;
mod rules;
pub mod suppression;

pub use config::effective_lint_config;
pub use discovery::{collect_sifr_files, collect_sifr_files_for_targets};
pub use engine::{LintPhase, LintRun, LintRunner, PhaseExecution};
pub use suppression::ParserAwareSuppressions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleSeverity {
    Ignore,
    Warn,
    Error,
}

impl RuleSeverity {
    fn diagnostic_severity(self) -> Severity {
        match self {
            Self::Ignore => Severity::Note,
            Self::Warn => Severity::Warning,
            Self::Error => Severity::Error,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleStatus {
    Stable,
    Experimental,
    Deprecated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FixAvailability {
    None,
    Safe,
    Unsafe,
    Manual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuppressionComplexity {
    PhysicalLine,
    SingleNode,
    StatementRange,
    SymbolWorkspace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuleMetadata {
    pub id: &'static str,
    pub summary: &'static str,
    pub docs_url: &'static str,
    pub default_level: RuleSeverity,
    pub status: RuleStatus,
    pub category: &'static str,
    pub source: &'static str,
    pub fix_availability: FixAvailability,
    pub suppression_complexity: SuppressionComplexity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticMode {
    Off,
    OpenFiles,
    Workspace,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PerFileIgnore {
    pub pattern: String,
    pub rules: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LintOptions {
    pub mode: DiagnosticMode,
    pub explicit_target: bool,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub respect_gitignore: bool,
    pub force_exclude: bool,
    pub preview: bool,
    pub select: Vec<String>,
    pub extend_select: Vec<String>,
    pub ignore: Vec<String>,
    pub rule_levels: BTreeMap<String, RuleSeverity>,
    pub per_file_ignores: Vec<PerFileIgnore>,
    pub ignore_suppressions: bool,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            mode: DiagnosticMode::Workspace,
            explicit_target: true,
            include: vec!["*.sifr".to_string()],
            exclude: Vec::new(),
            respect_gitignore: true,
            force_exclude: false,
            preview: false,
            select: vec!["default".to_string()],
            extend_select: Vec::new(),
            ignore: Vec::new(),
            rule_levels: BTreeMap::new(),
            per_file_ignores: Vec::new(),
            ignore_suppressions: false,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LintConfigOverrides {
    pub select: Option<Vec<String>>,
    pub extend_select: Vec<String>,
    pub ignore: Vec<String>,
    pub per_file_ignores: Vec<PerFileIgnore>,
    pub extend_per_file_ignores: Vec<PerFileIgnore>,
    pub exclude: Vec<String>,
    pub extend_exclude: Vec<String>,
    pub respect_gitignore: Option<bool>,
    pub force_exclude: Option<bool>,
    pub preview: Option<bool>,
    pub ignore_suppressions: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EffectiveLintConfig {
    pub options: LintOptions,
    pub config_path: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LintResult {
    pub diagnostics: Vec<RenderedDiagnostic>,
}

pub const RULES: &[RuleMetadata] = &[
    RuleMetadata {
        id: "trailing-whitespace",
        summary: "Line ends with trailing horizontal whitespace.",
        docs_url: "https://sifr.sh/docs/errors/SIFR-LINT-0004",
        default_level: RuleSeverity::Warn,
        status: RuleStatus::Stable,
        category: "style-policy",
        source: "sifr_lint::rules::trailing_whitespace",
        fix_availability: FixAvailability::None,
        suppression_complexity: SuppressionComplexity::PhysicalLine,
    },
    RuleMetadata {
        id: "unknown-suppression",
        summary: "Suppression references an unknown policy rule id.",
        docs_url: "https://sifr.sh/docs/errors/SIFR-LINT-0001",
        default_level: RuleSeverity::Warn,
        status: RuleStatus::Stable,
        category: "suppression-policy",
        source: "sifr_lint::suppressions",
        fix_availability: FixAvailability::None,
        suppression_complexity: SuppressionComplexity::PhysicalLine,
    },
    RuleMetadata {
        id: "unused-suppression",
        summary: "Suppression did not suppress any diagnostic.",
        docs_url: "https://sifr.sh/docs/errors/SIFR-LINT-0002",
        default_level: RuleSeverity::Warn,
        status: RuleStatus::Stable,
        category: "suppression-policy",
        source: "sifr_lint::suppressions",
        fix_availability: FixAvailability::None,
        suppression_complexity: SuppressionComplexity::PhysicalLine,
    },
    RuleMetadata {
        id: "blanket-suppression",
        summary: "Suppression must list explicit Sifr policy rule ids.",
        docs_url: "https://sifr.sh/docs/errors/SIFR-LINT-0003",
        default_level: RuleSeverity::Warn,
        status: RuleStatus::Stable,
        category: "suppression-policy",
        source: "sifr_lint::suppressions",
        fix_availability: FixAvailability::None,
        suppression_complexity: SuppressionComplexity::PhysicalLine,
    },
    RuleMetadata {
        id: "todo-comment",
        summary: "Comment contains a tracked TODO or FIXME marker.",
        docs_url: "https://sifr.sh/docs/errors/SIFR-LINT-0005",
        default_level: RuleSeverity::Warn,
        status: RuleStatus::Stable,
        category: "comment-policy",
        source: "sifr_lint::rules::todo_comment",
        fix_availability: FixAvailability::None,
        suppression_complexity: SuppressionComplexity::PhysicalLine,
    },
    RuleMetadata {
        id: "boolean-positional-argument",
        summary: "Call passes a boolean literal positionally.",
        docs_url: "https://sifr.sh/docs/errors/SIFR-LINT-0006",
        default_level: RuleSeverity::Warn,
        status: RuleStatus::Stable,
        category: "readability-policy",
        source: "sifr_lint::rules::boolean_positional_argument",
        fix_availability: FixAvailability::None,
        suppression_complexity: SuppressionComplexity::SingleNode,
    },
    RuleMetadata {
        id: "large-parameter-list",
        summary: "Function has more parameters than the policy limit.",
        docs_url: "https://sifr.sh/docs/errors/SIFR-LINT-0007",
        default_level: RuleSeverity::Warn,
        status: RuleStatus::Stable,
        category: "complexity-policy",
        source: "sifr_lint::rules::large_parameter_list",
        fix_availability: FixAvailability::None,
        suppression_complexity: SuppressionComplexity::StatementRange,
    },
    RuleMetadata {
        id: "duplicate-import",
        summary: "Import duplicates a module/name pair already imported in the same source file.",
        docs_url: "https://sifr.sh/docs/errors/SIFR-LINT-0008",
        default_level: RuleSeverity::Warn,
        status: RuleStatus::Stable,
        category: "workspace-policy",
        source: "sifr_lint::rules::duplicate_import",
        fix_availability: FixAvailability::None,
        suppression_complexity: SuppressionComplexity::SymbolWorkspace,
    },
];

pub fn rule_metadata(rule_id: &str) -> Option<&'static RuleMetadata> {
    RULES.iter().find(|rule| rule.id == rule_id)
}

pub fn lint_source(source: &str, file: Option<&Path>, options: &LintOptions) -> LintResult {
    LintRunner::new(options).run_source(source, file).result
}

pub(crate) fn lint_physical_line_rules(
    source: &str,
    file: Option<&Path>,
    options: &LintOptions,
    suppressions: &mut ParserAwareSuppressions,
) -> Vec<RenderedDiagnostic> {
    if options.mode == DiagnosticMode::Off {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();

    for (line_index, line) in source.split_inclusive('\n').enumerate() {
        let rule = "trailing-whitespace";
        if line_has_trailing_whitespace(line) && rule_enabled(rule, file, options) {
            if suppressions.mark_suppressed(line_index, rule, SuppressionComplexity::PhysicalLine) {
                continue;
            }
            diagnostics.push(with_rule_severity(
                trailing_whitespace_diagnostic(file, source, line_index, line),
                rule,
                options,
            ));
        }
    }

    diagnostics
}

pub fn lint_path(
    path: &Path,
    options: &LintOptions,
) -> Result<LintResult, Vec<RenderedDiagnostic>> {
    lint_paths(&[path.to_path_buf()], options)
}

pub fn lint_paths(
    paths: &[PathBuf],
    options: &LintOptions,
) -> Result<LintResult, Vec<RenderedDiagnostic>> {
    LintRunner::new(options)
        .run_paths(paths)
        .map(|run| run.result)
}

pub(crate) fn rule_enabled(rule_id: &str, file: Option<&Path>, options: &LintOptions) -> bool {
    let Some(metadata) = rule_metadata(rule_id) else {
        return false;
    };
    if per_file_ignored(rule_id, file, options) {
        return false;
    }
    if options
        .ignore
        .iter()
        .any(|selector| selector_matches(selector, metadata))
    {
        return false;
    }
    let selected = options
        .select
        .iter()
        .chain(options.extend_select.iter())
        .any(|selector| selector_matches(selector, metadata));
    let level = options
        .rule_levels
        .get(rule_id)
        .copied()
        .unwrap_or(metadata.default_level);
    selected && level != RuleSeverity::Ignore
}

fn selector_matches(selector: &str, rule: &RuleMetadata) -> bool {
    match selector {
        "all" => rule.status != RuleStatus::Deprecated,
        "default" => {
            rule.default_level != RuleSeverity::Ignore && rule.status == RuleStatus::Stable
        }
        _ => selector == rule.id || selector == rule.category,
    }
}

fn per_file_ignored(rule_id: &str, file: Option<&Path>, options: &LintOptions) -> bool {
    let Some(file) = file else {
        return false;
    };
    options.per_file_ignores.iter().any(|ignore| {
        ignore.rules.iter().any(|rule| rule == rule_id)
            && Glob::new(&ignore.pattern)
                .ok()
                .is_some_and(|glob| glob.compile_matcher().is_match(file))
    })
}

pub(crate) fn suppression_shape_diagnostics(
    suppressions: &ParserAwareSuppressions,
    file: Option<&Path>,
    source: &str,
    options: &LintOptions,
) -> Vec<RenderedDiagnostic> {
    let mut diagnostics = Vec::new();
    for suppression in suppressions.directives() {
        if suppression.rules.is_empty() {
            push_if_enabled(
                &mut diagnostics,
                suppression_diagnostic(
                    DiagnosticCode::LINT_BLANKET_SUPPRESSION,
                    "sifr suppression must list explicit policy rule ids",
                    "blanket-suppression",
                    suppression.line,
                    file,
                    source,
                    Some("use `# sifr: ignore[rule-id]` with a Sifr policy rule id"),
                ),
                "blanket-suppression",
                file,
                options,
            );
            continue;
        }
        for rule in &suppression.rules {
            if rule_metadata(rule).is_none() {
                push_if_enabled(
                    &mut diagnostics,
                    suppression_diagnostic(
                        DiagnosticCode::LINT_UNKNOWN_SUPPRESSION,
                        format!("unknown Sifr policy rule id '{rule}'"),
                        rule,
                        suppression.line,
                        file,
                        source,
                        Some("remove the suppression or use a known Sifr policy rule id"),
                    ),
                    "unknown-suppression",
                    file,
                    options,
                );
            }
        }
    }
    diagnostics
}

fn push_if_enabled(
    diagnostics: &mut Vec<RenderedDiagnostic>,
    diagnostic: RenderedDiagnostic,
    rule: &str,
    file: Option<&Path>,
    options: &LintOptions,
) {
    if rule_enabled(rule, file, options) {
        diagnostics.push(with_rule_severity(diagnostic, rule, options));
    }
}

pub(crate) fn unused_suppression_diagnostics(
    suppressions: &ParserAwareSuppressions,
    file: Option<&Path>,
    source: &str,
    options: &LintOptions,
) -> Vec<RenderedDiagnostic> {
    let mut diagnostics = Vec::new();
    for suppression in suppressions.directives() {
        for rule in &suppression.rules {
            if rule_metadata(rule).is_some() && !suppression.is_used_for(rule) {
                push_if_enabled(
                    &mut diagnostics,
                    suppression_diagnostic(
                        DiagnosticCode::LINT_UNUSED_SUPPRESSION,
                        format!("unused Sifr suppression for policy rule '{rule}'"),
                        rule,
                        suppression.line,
                        file,
                        source,
                        Some("remove the unused suppression"),
                    ),
                    "unused-suppression",
                    file,
                    options,
                );
            }
        }
    }
    diagnostics
}

pub(crate) fn with_rule_severity(
    mut diagnostic: RenderedDiagnostic,
    rule: &str,
    options: &LintOptions,
) -> RenderedDiagnostic {
    if let Some(level) = options.rule_levels.get(rule) {
        diagnostic.severity = level.diagnostic_severity();
    }
    diagnostic
}

pub(crate) struct LintDiagnosticSpec<'a> {
    pub code: DiagnosticCode,
    pub message: String,
    pub message_template: &'static str,
    pub rule: &'static str,
    pub file: Option<&'a Path>,
    pub source: &'a str,
    pub byte_start: u32,
    pub byte_end: u32,
    pub label: &'static str,
    pub help: Option<&'static str>,
    pub extra_args: Vec<(&'static str, String)>,
}

pub(crate) fn lint_rule_diagnostic(spec: LintDiagnosticSpec<'_>) -> RenderedDiagnostic {
    let (line, column) = line_column_for_byte(spec.source, spec.byte_start);
    let (end_line, end_column) = line_column_for_byte(spec.source, spec.byte_end);
    let mut args = vec![("rule", spec.rule.to_string())];
    args.extend(spec.extra_args);
    let mut diagnostic = diagnostic(
        spec.code,
        spec.message,
        args,
        vec![DiagnosticSpan {
            file: spec.file.map(|path| path.display().to_string()),
            byte_start: spec.byte_start,
            byte_end: spec.byte_end.max(spec.byte_start.saturating_add(1)),
            line: Some(line),
            column: Some(column),
            end_line: Some(end_line),
            end_column: Some(end_column),
            is_primary: true,
            label: Some(spec.label.to_string()),
            lines: Vec::new(),
        }],
        spec.help,
    );
    diagnostic.message_template = spec.message_template.to_string();
    diagnostic
}

fn trailing_whitespace_diagnostic(
    file: Option<&Path>,
    source: &str,
    line_index: usize,
    line: &str,
) -> RenderedDiagnostic {
    let visible_line = line.trim_end_matches('\n');
    let trimmed = visible_line.trim_end_matches([' ', '\t']);
    let column = u32::try_from(trimmed.chars().count().saturating_add(1)).unwrap_or(u32::MAX);
    let byte_start = byte_offset_for_line(source, line_index)
        .saturating_add(u32::try_from(trimmed.len()).unwrap_or(u32::MAX));
    let byte_end = byte_start.saturating_add(
        u32::try_from(visible_line.len().saturating_sub(trimmed.len())).unwrap_or(1),
    );
    let mut diagnostic = diagnostic(
        DiagnosticCode::LINT_TRAILING_WHITESPACE,
        "line has trailing whitespace",
        [("rule", "trailing-whitespace".to_string())],
        vec![DiagnosticSpan {
            file: file.map(|path| path.display().to_string()),
            byte_start,
            byte_end,
            line: Some(u32::try_from(line_index.saturating_add(1)).unwrap_or(u32::MAX)),
            column: Some(column),
            end_line: Some(u32::try_from(line_index.saturating_add(1)).unwrap_or(u32::MAX)),
            end_column: Some(column.saturating_add(1)),
            is_primary: true,
            label: Some("trailing whitespace".to_string()),
            lines: Vec::new(),
        }],
        Some("run `sifr fmt` to remove trailing whitespace"),
    );
    diagnostic.message_template = "line has trailing whitespace".to_string();
    diagnostic
}

fn suppression_diagnostic(
    code: DiagnosticCode,
    message: impl Into<String>,
    rule: &str,
    line_index: usize,
    file: Option<&Path>,
    source: &str,
    help: Option<&str>,
) -> RenderedDiagnostic {
    let line_start = byte_offset_for_line(source, line_index);
    let mut diagnostic = diagnostic(
        code,
        message,
        [("rule", rule.to_string())],
        vec![DiagnosticSpan {
            file: file.map(|path| path.display().to_string()),
            byte_start: line_start,
            byte_end: line_start.saturating_add(1),
            line: Some(u32::try_from(line_index.saturating_add(1)).unwrap_or(u32::MAX)),
            column: Some(1),
            end_line: Some(u32::try_from(line_index.saturating_add(1)).unwrap_or(u32::MAX)),
            end_column: Some(2),
            is_primary: true,
            label: Some("suppression comment".to_string()),
            lines: Vec::new(),
        }],
        help,
    );
    diagnostic.message_template = match code {
        DiagnosticCode::LINT_UNKNOWN_SUPPRESSION => "unknown Sifr policy rule id '{rule}'",
        DiagnosticCode::LINT_UNUSED_SUPPRESSION => {
            "unused Sifr suppression for policy rule '{rule}'"
        }
        DiagnosticCode::LINT_BLANKET_SUPPRESSION => {
            "sifr suppression must list explicit policy rule ids"
        }
        _ => "{message}",
    }
    .to_string();
    diagnostic
}

pub(crate) fn lint_config_diagnostic(message: impl Into<String>) -> RenderedDiagnostic {
    diagnostic(
        DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
        message.into(),
        [],
        Vec::new(),
        None,
    )
}

pub(crate) fn diagnostic(
    code: DiagnosticCode,
    message: impl Into<String>,
    args: impl IntoIterator<Item = (&'static str, String)>,
    spans: Vec<DiagnosticSpan>,
    help: Option<&str>,
) -> RenderedDiagnostic {
    let message = message.into();
    let mut rendered_args = BTreeMap::new();
    for (key, value) in args {
        rendered_args.insert(key.to_string(), DiagnosticArg::String(value));
    }
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message,
        message_template: "{message}".to_string(),
        args: rendered_args,
        url: code.docs_url(),
        spans,
        children: Vec::new(),
        help: help.map(str::to_string),
        suggestions: Vec::new(),
    }
}

pub(crate) fn diagnostic_order_key(diagnostic: &RenderedDiagnostic) -> (String, u32, u32, String) {
    let span = diagnostic.spans.iter().find(|span| span.is_primary);
    (
        span.and_then(|span| span.file.clone()).unwrap_or_default(),
        span.map_or(u32::MAX, |span| span.byte_start),
        span.map_or(u32::MAX, |span| span.byte_end),
        diagnostic.code.clone(),
    )
}

fn line_has_trailing_whitespace(line: &str) -> bool {
    let visible = line.trim_end_matches('\n');
    visible.ends_with(' ') || visible.ends_with('\t')
}

pub(crate) fn byte_offset_for_line(source: &str, line_index: usize) -> u32 {
    let mut offset = 0usize;
    for (index, line) in source.split_inclusive('\n').enumerate() {
        if index == line_index {
            break;
        }
        offset = offset.saturating_add(line.len());
    }
    u32::try_from(offset).unwrap_or(u32::MAX)
}

pub(crate) fn line_index_for_byte(source: &str, byte: u32) -> usize {
    let target = usize::try_from(byte).unwrap_or(usize::MAX);
    let mut offset = 0usize;
    for (index, line) in source.split_inclusive('\n').enumerate() {
        let next = offset.saturating_add(line.len());
        if target < next {
            return index;
        }
        offset = next;
    }
    source.lines().count().saturating_sub(1)
}

fn line_column_for_byte(source: &str, byte: u32) -> (u32, u32) {
    let target = usize::try_from(byte).unwrap_or(usize::MAX);
    let mut offset = 0usize;
    for (index, line) in source.split_inclusive('\n').enumerate() {
        let next = offset.saturating_add(line.len());
        if target <= next {
            return (
                u32::try_from(index.saturating_add(1)).unwrap_or(u32::MAX),
                u32::try_from(target.saturating_sub(offset).saturating_add(1)).unwrap_or(u32::MAX),
            );
        }
        offset = next;
    }
    (
        u32::try_from(source.lines().count().max(1)).unwrap_or(u32::MAX),
        1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn suppression_only_suppresses_matching_policy_rule() {
        let source = "def main():  # sifr: ignore[trailing-whitespace]  \n    pass\n";
        let result = lint_source(source, None, &LintOptions::default());
        assert!(result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "SIFR-LINT-0004"));
    }

    #[test]
    fn unknown_and_unused_suppressions_are_reported() {
        let source = "def main(): # sifr: ignore[not-a-rule, trailing-whitespace]\n    pass\n";
        let result = lint_source(source, None, &LintOptions::default());
        let codes = result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<BTreeSet<_>>();
        assert!(codes.contains("SIFR-LINT-0001"));
        assert!(codes.contains("SIFR-LINT-0002"));
    }

    #[test]
    fn blanket_suppression_is_reported() {
        let source = "def main(): # sifr: ignore\n    pass\n";
        let result = lint_source(source, None, &LintOptions::default());
        assert_eq!(result.diagnostics[0].code, "SIFR-LINT-0003");
    }

    #[test]
    fn config_rule_ignore_disables_rule() {
        let root = temp_dir("lint_config_rule_ignore");
        fs::write(
            root.join("sifr.toml"),
            "[lint.rules]\ntrailing-whitespace = \"ignore\"\n",
        )
        .unwrap();
        let config =
            effective_lint_config(&root, &[], false, &LintConfigOverrides::default()).unwrap();
        let result = lint_source("def main():  \n    pass\n", None, &config.options);
        assert!(result.diagnostics.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn discovery_respects_gitignore_unless_disabled() {
        let root = temp_dir("lint_discovery_gitignore");
        fs::create_dir_all(root.join("ignored")).unwrap();
        fs::write(root.join(".gitignore"), "ignored/\n").unwrap();
        fs::write(root.join("main.sifr"), "def main():\n    pass\n").unwrap();
        fs::write(
            root.join("ignored").join("skip.sifr"),
            "def skip():\n    pass\n",
        )
        .unwrap();
        let files = collect_sifr_files(&root, &LintOptions::default()).unwrap();
        assert_eq!(files.len(), 1);
        let options = LintOptions {
            respect_gitignore: false,
            ..LintOptions::default()
        };
        let files = collect_sifr_files(&root, &options).unwrap();
        assert_eq!(files.len(), 2);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn per_file_ignore_filters_rule() {
        let options = LintOptions {
            per_file_ignores: vec![PerFileIgnore {
                pattern: "*.sifr".to_string(),
                rules: vec!["trailing-whitespace".to_string()],
            }],
            ..LintOptions::default()
        };
        let result = lint_source(
            "def main():  \n    pass\n",
            Some(Path::new("main.sifr")),
            &options,
        );
        assert!(result.diagnostics.is_empty());
    }

    fn temp_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("sifr_{name}_{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
