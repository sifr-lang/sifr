//! Sifr-owned formatter foundation over `sifr_syntax`.
//!
//! The formatter deliberately starts with conservative, syntax-validated edits:
//! normalize line endings to LF, trim trailing horizontal whitespace outside
//! string tokens, and ensure a final newline. This keeps formatting idempotent
//! and parser-round-tripped while preserving comments and string contents.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

use ruff_text_size::{Ranged as _, TextRange, TextSize};
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, DiagnosticSpan, RenderedDiagnostic};
use sifr_syntax::{parse_module, SourceText};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormatOptions {
    pub final_newline: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            final_newline: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextEdit {
    pub range: TextRange,
    pub replacement: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatResult {
    pub formatted: String,
    pub changed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FormatCheck {
    pub formatted: String,
    pub diagnostics: Vec<RenderedDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormattedPath {
    pub path: PathBuf,
    pub changed: bool,
}

pub fn format_source(
    source: &str,
    file: Option<&Path>,
    options: FormatOptions,
) -> Result<FormatResult, Vec<RenderedDiagnostic>> {
    let parsed = parse_module(
        source,
        file.map(|path| path.display().to_string()).as_deref(),
    )?;
    let protected = parsed
        .tokens()
        .iter()
        .filter(|token| token.kind.as_str() == "String")
        .map(|token| token.range)
        .collect::<Vec<_>>();
    let formatted = normalize_source(source, &protected, options);
    parse_module(
        &formatted,
        file.map(|path| path.display().to_string()).as_deref(),
    )?;
    Ok(FormatResult {
        changed: formatted != source,
        formatted,
    })
}

pub fn check_source(
    source: &str,
    file: Option<&Path>,
    options: FormatOptions,
) -> Result<FormatCheck, Vec<RenderedDiagnostic>> {
    let result = format_source(source, file, options)?;
    let diagnostics = if result.changed {
        vec![formatting_drift_diagnostic(source, file, &result.formatted)]
    } else {
        Vec::new()
    };
    Ok(FormatCheck {
        formatted: result.formatted,
        diagnostics,
    })
}

pub fn format_range(
    source: &str,
    range: TextRange,
    file: Option<&Path>,
    options: FormatOptions,
) -> Result<Vec<TextEdit>, Vec<RenderedDiagnostic>> {
    let result = format_source(source, file, options)?;
    if !result.changed {
        return Ok(Vec::new());
    }
    let full = full_range(source);
    if range != full {
        return Ok(vec![TextEdit {
            range: full,
            replacement: result.formatted,
        }]);
    }
    Ok(vec![TextEdit {
        range,
        replacement: result.formatted,
    }])
}

pub fn format_path(path: &Path, check: bool) -> Result<FormattedPath, Vec<RenderedDiagnostic>> {
    let source = read_source(path)?;
    if check {
        let check = check_source(&source, Some(path), FormatOptions::default())?;
        return Ok(FormattedPath {
            path: path.to_path_buf(),
            changed: !check.diagnostics.is_empty(),
        });
    }
    let result = format_source(&source, Some(path), FormatOptions::default())?;
    if result.changed {
        write_source(path, &result.formatted)?;
    }
    Ok(FormattedPath {
        path: path.to_path_buf(),
        changed: result.changed,
    })
}

pub fn check_path(path: &Path) -> Result<Vec<RenderedDiagnostic>, Vec<RenderedDiagnostic>> {
    let source = read_source(path)?;
    check_source(&source, Some(path), FormatOptions::default()).map(|check| check.diagnostics)
}

pub fn collect_sifr_files(path: &Path) -> Result<Vec<PathBuf>, Vec<RenderedDiagnostic>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if !path.is_dir() {
        return Err(vec![io_diagnostic(
            DiagnosticCode::FMT_FORMATTING_DRIFT,
            format!("format target does not exist: {}", path.display()),
            Some(path),
        )]);
    }
    let mut files = Vec::new();
    collect_sifr_files_inner(path, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_sifr_files_inner(
    path: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), Vec<RenderedDiagnostic>> {
    for entry in fs::read_dir(path).map_err(|err| {
        vec![io_diagnostic(
            DiagnosticCode::FMT_FORMATTING_DRIFT,
            format!("could not read directory {}: {err}", path.display()),
            Some(path),
        )]
    })? {
        let entry = entry.map_err(|err| {
            vec![io_diagnostic(
                DiagnosticCode::FMT_FORMATTING_DRIFT,
                format!(
                    "could not read directory entry under {}: {err}",
                    path.display()
                ),
                Some(path),
            )]
        })?;
        let child = entry.path();
        if child.is_dir() {
            if is_default_excluded_dir(&child) {
                continue;
            }
            collect_sifr_files_inner(&child, files)?;
        } else if is_sifr_file(&child) {
            files.push(child);
        }
    }
    Ok(())
}

fn normalize_source(source: &str, protected: &[TextRange], options: FormatOptions) -> String {
    let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
    let mut output = String::with_capacity(normalized.len() + 1);
    let mut offset = 0usize;
    for segment in normalized.split_inclusive('\n') {
        let line = segment.strip_suffix('\n').unwrap_or(segment);
        let trimmed_len = trim_line_len_outside_protected(line, offset, protected);
        output.push_str(&line[..trimmed_len]);
        if segment.ends_with('\n') {
            output.push('\n');
        }
        offset = offset.saturating_add(segment.len());
    }
    if options.final_newline && !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn trim_line_len_outside_protected(
    line: &str,
    line_offset: usize,
    protected: &[TextRange],
) -> usize {
    let mut end = line.len();
    while end > 0 {
        let Some(ch) = line[..end].chars().next_back() else {
            break;
        };
        if ch != ' ' && ch != '\t' {
            break;
        }
        let byte_start = end.saturating_sub(ch.len_utf8());
        if protected_contains(protected, line_offset.saturating_add(byte_start)) {
            break;
        }
        end = byte_start;
    }
    end
}

fn protected_contains(protected: &[TextRange], offset: usize) -> bool {
    let Ok(raw) = u32::try_from(offset) else {
        return false;
    };
    let offset = TextSize::new(raw);
    protected
        .iter()
        .any(|range| range.start() <= offset && offset < range.end())
}

fn formatting_drift_diagnostic(
    source: &str,
    file: Option<&Path>,
    formatted: &str,
) -> RenderedDiagnostic {
    let first_diff = first_diff_offset(source, formatted);
    let source_text = SourceText::new(source.to_string());
    let span = source_text
        .text_position(TextSize::new(first_diff))
        .map(|position| DiagnosticSpan {
            file: file.map(|path| path.display().to_string()),
            byte_start: first_diff,
            byte_end: first_diff.saturating_add(1),
            line: Some(position.line.saturating_add(1)),
            column: Some(position.character.saturating_add(1)),
            end_line: Some(position.line.saturating_add(1)),
            end_column: Some(position.character.saturating_add(2)),
            is_primary: true,
            label: Some("formatting differs here".to_string()),
            lines: Vec::new(),
        });
    let mut diagnostic = diagnostic(
        DiagnosticCode::FMT_FORMATTING_DRIFT,
        "source is not formatted with sifr fmt",
        [("path", file_display(file))],
        span.into_iter().collect(),
        Some("run `sifr fmt` to apply formatter changes"),
    );
    diagnostic.message_template = "source is not formatted with sifr fmt".to_string();
    diagnostic
}

fn first_diff_offset(left: &str, right: &str) -> u32 {
    let offset = left
        .bytes()
        .zip(right.bytes())
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| left.len().min(right.len()));
    u32::try_from(offset).unwrap_or(u32::MAX)
}

fn read_source(path: &Path) -> Result<String, Vec<RenderedDiagnostic>> {
    fs::read_to_string(path).map_err(|err| {
        vec![io_diagnostic(
            DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            format!("could not read file {}: {err}", path.display()),
            Some(path),
        )]
    })
}

fn write_source(path: &Path, source: &str) -> Result<(), Vec<RenderedDiagnostic>> {
    fs::write(path, source).map_err(|err| {
        vec![io_diagnostic(
            DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            format!("could not write file {}: {err}", path.display()),
            Some(path),
        )]
    })
}

fn io_diagnostic(code: DiagnosticCode, message: String, file: Option<&Path>) -> RenderedDiagnostic {
    diagnostic(
        code,
        message,
        [("path", file_display(file))],
        Vec::new(),
        None,
    )
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

fn full_range(source: &str) -> TextRange {
    let end = u32::try_from(source.len()).unwrap_or(u32::MAX);
    TextRange::new(TextSize::new(0), TextSize::new(end))
}

fn file_display(file: Option<&Path>) -> String {
    file.map_or_else(|| "<memory>".to_string(), |path| path.display().to_string())
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
    fn formatter_is_idempotent_and_preserves_string_trailing_space() {
        let source = "def main():  \n    value: str = \"kept \"  \n    print(value)\n";
        let first = format_source(source, None, FormatOptions::default())
            .expect("source should format")
            .formatted;
        let second = format_source(&first, None, FormatOptions::default())
            .expect("formatted source should format")
            .formatted;
        assert_eq!(first, second);
        assert!(first.contains("\"kept \""));
        assert!(!first.contains("def main():  \n"));
    }

    #[test]
    fn check_reports_formatting_drift() {
        let check = check_source("def main():  \n    pass", None, FormatOptions::default())
            .expect("check should run");
        assert_eq!(check.diagnostics[0].code, "SIFR-FMT-0001");
    }
}
