//! Sifr-owned formatter API backed by the Sifr-aware Ruff formatter.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

use ruff_formatter::printer::LineEnding;
use ruff_python_formatter::{
    format_sifr_module_source, format_sifr_range as ruff_format_sifr_range, FormatModuleError,
    PyFormatOptions,
};
use ruff_text_size::{TextRange, TextSize};
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
    let ruff_options = ruff_options(file, options)?;
    let formatted = format_sifr_module_source(source, ruff_options)
        .map_err(|error| vec![format_module_error_diagnostic(source, file, &error)])?
        .into_code();
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
    let ruff_options = ruff_options(file, options)?;
    validate_range(source, range, file)?;
    let printed = ruff_format_sifr_range(source, range, ruff_options)
        .map_err(|error| vec![format_module_error_diagnostic(source, file, &error)])?;
    let edit_range = printed.source_range();
    if edit_range.is_empty() {
        return Ok(Vec::new());
    }
    let replacement = printed.into_code();
    if source_slice(source, edit_range).is_some_and(|current| current == replacement) {
        return Ok(Vec::new());
    }
    let roundtripped = source_with_edit(source, edit_range, &replacement)
        .ok_or_else(|| vec![invalid_range_diagnostic(source, file, edit_range)])?;
    parse_module(
        &roundtripped,
        file.map(|path| path.display().to_string()).as_deref(),
    )?;
    Ok(vec![TextEdit {
        range: edit_range,
        replacement,
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

fn ruff_options(
    file: Option<&Path>,
    options: FormatOptions,
) -> Result<PyFormatOptions, Vec<RenderedDiagnostic>> {
    if !options.final_newline {
        return Err(vec![unsupported_option_diagnostic(
            file,
            "final_newline=false",
            "the Ruff-backed Sifr formatter currently requires a final newline",
        )]);
    }
    let options = file.map_or_else(PyFormatOptions::default, PyFormatOptions::from_extension);
    Ok(options.with_line_ending(LineEnding::LineFeed))
}

fn validate_range(
    source: &str,
    range: TextRange,
    file: Option<&Path>,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let start = usize::from(range.start());
    let end = usize::from(range.end());
    if end > source.len()
        || start > end
        || !source.is_char_boundary(start)
        || !source.is_char_boundary(end)
    {
        return Err(vec![invalid_range_diagnostic(source, file, range)]);
    }
    Ok(())
}

fn source_slice(source: &str, range: TextRange) -> Option<&str> {
    source.get(usize::from(range.start())..usize::from(range.end()))
}

fn source_with_edit(source: &str, range: TextRange, replacement: &str) -> Option<String> {
    let start = usize::from(range.start());
    let end = usize::from(range.end());
    source.get(start..end)?;
    let mut output = String::with_capacity(
        source
            .len()
            .saturating_sub(end.saturating_sub(start))
            .saturating_add(replacement.len()),
    );
    output.push_str(source.get(..start)?);
    output.push_str(replacement);
    output.push_str(source.get(end..)?);
    Some(output)
}

fn format_module_error_diagnostic(
    source: &str,
    file: Option<&Path>,
    error: &FormatModuleError,
) -> RenderedDiagnostic {
    let (message, help) = match error {
        FormatModuleError::ParseError(_) => (
            "formatter could not parse Sifr source",
            Some("fix the syntax error before running `sifr fmt`"),
        ),
        FormatModuleError::FormatError(_) => (
            "formatter could not format Sifr source",
            Some("reduce the formatted range or report this formatter case"),
        ),
        FormatModuleError::PrintError(_) => (
            "formatter could not print Sifr source",
            Some("report this formatter print failure"),
        ),
    };
    let spans = error
        .range()
        .and_then(|range| span_for_range(source, file, range, "formatter failed here"))
        .into_iter()
        .collect();
    diagnostic(
        DiagnosticCode::FMT_FORMATTING_DRIFT,
        message,
        [
            ("path", file_display(file)),
            ("formatter_error", error.to_string()),
        ],
        spans,
        help,
    )
}

fn invalid_range_diagnostic(
    source: &str,
    file: Option<&Path>,
    range: TextRange,
) -> RenderedDiagnostic {
    let spans = span_for_range(source, file, range, "invalid formatter range")
        .into_iter()
        .collect();
    diagnostic(
        DiagnosticCode::FMT_FORMATTING_DRIFT,
        "formatter range is outside Sifr source bounds or not on UTF-8 boundaries",
        [("path", file_display(file))],
        spans,
        Some("request formatting for a valid source range"),
    )
}

fn unsupported_option_diagnostic(
    file: Option<&Path>,
    option: &str,
    help: &str,
) -> RenderedDiagnostic {
    diagnostic(
        DiagnosticCode::FMT_FORMATTING_DRIFT,
        format!("unsupported formatter option: {option}"),
        [("path", file_display(file)), ("option", option.to_string())],
        Vec::new(),
        Some(help),
    )
}

fn span_for_range(
    source: &str,
    file: Option<&Path>,
    range: TextRange,
    label: &str,
) -> Option<DiagnosticSpan> {
    let source_text = SourceText::new(source.to_string());
    let start = range.start().to_u32();
    let end = range.end().to_u32().max(start.saturating_add(1));
    let position = source_text.text_position(TextSize::new(start))?;
    Some(DiagnosticSpan {
        file: file.map(|path| path.display().to_string()),
        byte_start: start,
        byte_end: end,
        line: Some(position.line.saturating_add(1)),
        column: Some(position.character.saturating_add(1)),
        end_line: Some(position.line.saturating_add(1)),
        end_column: Some(position.character.saturating_add(2)),
        is_primary: true,
        label: Some(label.to_string()),
        lines: Vec::new(),
    })
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
    fn formatter_is_ruff_backed_and_preserves_string_contents() {
        let source = "def main():\n    value: str = \"kept \"\n    print(  value  )\n";
        let first = format_source(source, None, FormatOptions::default())
            .expect("source should format")
            .formatted;
        let second = format_source(&first, None, FormatOptions::default())
            .expect("formatted source should format")
            .formatted;
        assert_eq!(first, second);
        assert!(first.contains("\"kept \""));
        assert!(first.contains("print(value)"));
        parse_module(&first, None).expect("formatted source should parse");
    }

    #[test]
    fn formatter_canonicalizes_sifr_parameter_conventions() {
        let source = "def consume(mut own data: list[int]) -> Result[int, Error]:\n    match data:\n        case []:\n            return 0\n        case _:\n            return data[0]\n";
        let formatted = format_source(source, None, FormatOptions::default())
            .expect("source should format")
            .formatted;
        assert!(formatted.contains("def consume(own mut data: list[int]) -> Result[int, Error]:"));
        assert!(!formatted.contains("mut own data"));
        parse_module(&formatted, None).expect("formatted Sifr extensions should parse");
    }

    #[test]
    fn range_formatting_returns_minimal_text_edit() {
        let source = "def main():\n    x=1\n    y = 2\n";
        let range = TextRange::new(TextSize::new(16), TextSize::new(19));
        let edits = format_range(source, range, None, FormatOptions::default())
            .expect("range should format");
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].replacement, "    x = 1");
        let formatted = source_with_edit(source, edits[0].range, &edits[0].replacement)
            .expect("edit should apply");
        assert_eq!(formatted, "def main():\n    x = 1\n    y = 2\n");
    }

    #[test]
    fn check_reports_formatting_drift() {
        let check = check_source(
            "def main():\n    print( 1 )\n",
            None,
            FormatOptions::default(),
        )
        .expect("check should run");
        assert_eq!(check.diagnostics[0].code, "SIFR-FMT-0001");
    }

    #[test]
    fn invalid_source_reports_sifr_diagnostic() {
        let diagnostics =
            format_source("def broken(:\n", None, FormatOptions::default()).expect_err("invalid");
        assert_eq!(diagnostics[0].code, "SIFR-FMT-0001");
        assert_eq!(
            diagnostics[0].message,
            "formatter could not parse Sifr source"
        );
    }

    #[test]
    fn unsupported_final_newline_option_reports_diagnostic() {
        let diagnostics = format_source(
            "def main():\n    pass\n",
            None,
            FormatOptions {
                final_newline: false,
            },
        )
        .expect_err("unsupported option");
        assert_eq!(diagnostics[0].code, "SIFR-FMT-0001");
        assert_eq!(
            diagnostics[0].message,
            "unsupported formatter option: final_newline=false"
        );
    }
}
