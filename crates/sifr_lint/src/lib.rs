//! Sifr-owned policy-rule and suppression foundation.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, DiagnosticSpan, RenderedDiagnostic};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleSeverity {
    Ignore,
    Warn,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleStatus {
    Stable,
    Experimental,
    Deprecated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuleMetadata {
    pub id: &'static str,
    pub summary: &'static str,
    pub docs_url: &'static str,
    pub default_level: RuleSeverity,
    pub status: RuleStatus,
    pub source: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticMode {
    Off,
    OpenFiles,
    Workspace,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LintOptions {
    pub mode: DiagnosticMode,
    pub explicit_target: bool,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub respect_ignore_files: bool,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            mode: DiagnosticMode::Workspace,
            explicit_target: true,
            include: Vec::new(),
            exclude: Vec::new(),
            respect_ignore_files: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LintResult {
    pub diagnostics: Vec<RenderedDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Suppression {
    line: usize,
    rules: Vec<String>,
    used_rules: BTreeSet<String>,
}

pub const RULES: &[RuleMetadata] = &[
    RuleMetadata {
        id: "trailing-whitespace",
        summary: "Line ends with trailing horizontal whitespace.",
        docs_url: "https://sifr.sh/docs/errors/SIFR-LINT-0004",
        default_level: RuleSeverity::Warn,
        status: RuleStatus::Stable,
        source: "sifr_lint::rules::trailing_whitespace",
    },
    RuleMetadata {
        id: "unknown-suppression",
        summary: "Suppression references an unknown policy rule id.",
        docs_url: "https://sifr.sh/docs/errors/SIFR-LINT-0001",
        default_level: RuleSeverity::Warn,
        status: RuleStatus::Stable,
        source: "sifr_lint::suppressions",
    },
    RuleMetadata {
        id: "unused-suppression",
        summary: "Suppression did not suppress any diagnostic.",
        docs_url: "https://sifr.sh/docs/errors/SIFR-LINT-0002",
        default_level: RuleSeverity::Warn,
        status: RuleStatus::Stable,
        source: "sifr_lint::suppressions",
    },
    RuleMetadata {
        id: "blanket-suppression",
        summary: "Suppression must list explicit Sifr policy rule ids.",
        docs_url: "https://sifr.sh/docs/errors/SIFR-LINT-0003",
        default_level: RuleSeverity::Warn,
        status: RuleStatus::Stable,
        source: "sifr_lint::suppressions",
    },
];

pub fn rule_metadata(rule_id: &str) -> Option<&'static RuleMetadata> {
    RULES.iter().find(|rule| rule.id == rule_id)
}

pub fn lint_source(source: &str, file: Option<&Path>, options: &LintOptions) -> LintResult {
    if options.mode == DiagnosticMode::Off {
        return LintResult {
            diagnostics: Vec::new(),
        };
    }

    let mut suppressions = parse_suppressions(source, file);
    let mut diagnostics = Vec::new();
    diagnostics.extend(suppression_shape_diagnostics(&suppressions, file, source));

    for (line_index, line) in source.split_inclusive('\n').enumerate() {
        if line_has_trailing_whitespace(line) {
            let rule = "trailing-whitespace";
            if mark_suppressed(&mut suppressions, line_index, rule) {
                continue;
            }
            diagnostics.push(trailing_whitespace_diagnostic(
                file, source, line_index, line,
            ));
        }
    }

    diagnostics.extend(unused_suppression_diagnostics(&suppressions, file, source));
    diagnostics.sort_by_key(diagnostic_order_key);
    LintResult { diagnostics }
}

pub fn lint_path(
    path: &Path,
    options: &LintOptions,
) -> Result<LintResult, Vec<RenderedDiagnostic>> {
    let files = collect_sifr_files(path, options)?;
    let mut diagnostics = Vec::new();
    for file in files {
        let source = fs::read_to_string(&file).map_err(|err| {
            vec![diagnostic(
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                format!("could not read file {}: {err}", file.display()),
                [("path", file.display().to_string())],
                Vec::new(),
                None,
            )]
        })?;
        diagnostics.extend(lint_source(&source, Some(&file), options).diagnostics);
    }
    diagnostics.sort_by_key(diagnostic_order_key);
    Ok(LintResult { diagnostics })
}

pub fn collect_sifr_files(
    path: &Path,
    options: &LintOptions,
) -> Result<Vec<PathBuf>, Vec<RenderedDiagnostic>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if !path.is_dir() {
        return Err(vec![diagnostic(
            DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
            format!("lint target does not exist: {}", path.display()),
            [("path", path.display().to_string())],
            Vec::new(),
            None,
        )]);
    }
    let ignore_patterns = if options.respect_ignore_files {
        read_ignore_patterns(path)
    } else {
        Vec::new()
    };
    let mut files = Vec::new();
    collect_sifr_files_inner(path, path, options, &ignore_patterns, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_sifr_files_inner(
    root: &Path,
    path: &Path,
    options: &LintOptions,
    ignore_patterns: &[String],
    files: &mut Vec<PathBuf>,
) -> Result<(), Vec<RenderedDiagnostic>> {
    for entry in fs::read_dir(path).map_err(|err| {
        vec![diagnostic(
            DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
            format!("could not read directory {}: {err}", path.display()),
            [("path", path.display().to_string())],
            Vec::new(),
            None,
        )]
    })? {
        let entry = entry.map_err(|err| {
            vec![diagnostic(
                DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
                format!(
                    "could not read directory entry under {}: {err}",
                    path.display()
                ),
                [("path", path.display().to_string())],
                Vec::new(),
                None,
            )]
        })?;
        let child = entry.path();
        if should_exclude(root, &child, options, ignore_patterns) {
            continue;
        }
        if child.is_dir() {
            collect_sifr_files_inner(root, &child, options, ignore_patterns, files)?;
        } else if is_sifr_file(&child) {
            files.push(child);
        }
    }
    Ok(())
}

fn should_exclude(
    root: &Path,
    path: &Path,
    options: &LintOptions,
    ignore_patterns: &[String],
) -> bool {
    if options.explicit_target && path.is_file() {
        return false;
    }
    if is_default_excluded_dir(path) {
        return true;
    }
    let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
    options.exclude.iter().any(|pattern| rel.contains(pattern))
        || ignore_patterns.iter().any(|pattern| rel.contains(pattern))
}

fn parse_suppressions(source: &str, file: Option<&Path>) -> Vec<Suppression> {
    let mut suppressions = Vec::new();
    for (line_index, line) in source.lines().enumerate() {
        let Some(comment_start) = line.find('#') else {
            continue;
        };
        let comment = &line[comment_start..];
        let Some(ignore_start) = comment.find("sifr: ignore") else {
            continue;
        };
        let suffix = &comment[ignore_start + "sifr: ignore".len()..];
        let rules = if let Some(stripped) = suffix.strip_prefix('[') {
            stripped
                .split_once(']')
                .map(|(rule_list, _)| {
                    rule_list
                        .split(',')
                        .map(str::trim)
                        .filter(|rule| !rule.is_empty())
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let _ = file;
        suppressions.push(Suppression {
            line: line_index,
            rules,
            used_rules: BTreeSet::new(),
        });
    }
    suppressions
}

fn suppression_shape_diagnostics(
    suppressions: &[Suppression],
    file: Option<&Path>,
    source: &str,
) -> Vec<RenderedDiagnostic> {
    let mut diagnostics = Vec::new();
    for suppression in suppressions {
        if suppression.rules.is_empty() {
            diagnostics.push(suppression_diagnostic(
                DiagnosticCode::LINT_BLANKET_SUPPRESSION,
                "sifr suppression must list explicit policy rule ids",
                "blanket-suppression",
                suppression.line,
                file,
                source,
                Some("use `# sifr: ignore[rule-id]` with a Sifr policy rule id"),
            ));
            continue;
        }
        for rule in &suppression.rules {
            if rule_metadata(rule).is_none() {
                diagnostics.push(suppression_diagnostic(
                    DiagnosticCode::LINT_UNKNOWN_SUPPRESSION,
                    format!("unknown Sifr policy rule id '{rule}'"),
                    rule,
                    suppression.line,
                    file,
                    source,
                    Some("remove the suppression or use a known Sifr policy rule id"),
                ));
            }
        }
    }
    diagnostics
}

fn mark_suppressed(suppressions: &mut [Suppression], line: usize, rule: &str) -> bool {
    let Some(suppression) = suppressions.iter_mut().find(|suppression| {
        suppression.line == line && suppression.rules.iter().any(|candidate| candidate == rule)
    }) else {
        return false;
    };
    suppression.used_rules.insert(rule.to_string());
    true
}

fn unused_suppression_diagnostics(
    suppressions: &[Suppression],
    file: Option<&Path>,
    source: &str,
) -> Vec<RenderedDiagnostic> {
    let mut diagnostics = Vec::new();
    for suppression in suppressions {
        for rule in &suppression.rules {
            if rule_metadata(rule).is_some() && !suppression.used_rules.contains(rule) {
                diagnostics.push(suppression_diagnostic(
                    DiagnosticCode::LINT_UNUSED_SUPPRESSION,
                    format!("unused Sifr suppression for policy rule '{rule}'"),
                    rule,
                    suppression.line,
                    file,
                    source,
                    Some("remove the unused suppression"),
                ));
            }
        }
    }
    diagnostics
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

fn diagnostic(
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

fn diagnostic_order_key(diagnostic: &RenderedDiagnostic) -> (String, u32, u32, String) {
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

fn byte_offset_for_line(source: &str, line_index: usize) -> u32 {
    let mut offset = 0usize;
    for (index, line) in source.split_inclusive('\n').enumerate() {
        if index == line_index {
            break;
        }
        offset = offset.saturating_add(line.len());
    }
    u32::try_from(offset).unwrap_or(u32::MAX)
}

fn read_ignore_patterns(root: &Path) -> Vec<String> {
    [".gitignore", ".ignore"]
        .iter()
        .flat_map(|name| {
            fs::read_to_string(root.join(name))
                .unwrap_or_default()
                .lines()
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

fn is_sifr_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "sifr")
}

fn is_default_excluded_dir(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        matches!(
            name.to_string_lossy().as_ref(),
            ".git" | "target" | ".venv" | "venv" | "node_modules" | "sifr_output"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
