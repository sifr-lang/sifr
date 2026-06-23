use super::cli_model_and_entrypoint::{
    diagnostic_with_code, DiagnosticFormat, EXIT_SUCCESS, EXIT_USAGE_OR_CONFIG,
};
use super::diagnostic_rendering_and_run::render_diagnostics;
use sifr_diagnostics::DiagnosticCode;
use sifr_frontend::{DiskSourceProvider, SourceProvider};
use std::io::{self, Write as _};
use std::path::PathBuf;

pub(super) fn cmd_explain(code: &str, diagnostic_format: DiagnosticFormat) -> i32 {
    let explanation = diagnostic_explanation(code);
    if let Some(text) = explanation {
        match diagnostic_format {
            DiagnosticFormat::Human | DiagnosticFormat::Compact => {
                let _ = writeln!(io::stdout(), "{text}");
            }
            DiagnosticFormat::Json => {
                let value = serde_json::json!({ "code": code, "explanation": text });
                let _ = writeln!(
                    io::stdout(),
                    "{}",
                    serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
                );
            }
        }
        EXIT_SUCCESS
    } else {
        let diagnostic = diagnostic_with_code(
            format!("unknown diagnostic code '{code}'"),
            DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
        );
        render_diagnostics(&[diagnostic], diagnostic_format);
        EXIT_USAGE_OR_CONFIG
    }
}

pub(crate) fn diagnostic_explanation(code: &str) -> Option<String> {
    if code == "SIFR-PACKAGE-0105" {
        return Some(
            "SIFR-PACKAGE-0105 is retired. Cargo credential failures are reported as SIFR-PACKAGE-0101 so Sifr preserves Cargo's underlying error text with credential redaction."
                .to_string(),
        );
    }
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()?
        .parent()?
        .to_path_buf();
    let path = repo_root.join("docs/errors").join(format!("{code}.md"));
    let text = DiskSourceProvider::new().read_file(&path).ok()?;
    let mut lines = text
        .as_str()
        .lines()
        .filter(|line| !line.starts_with("<!--") && !line.starts_with('|'));
    let title = lines.find(|line| line.starts_with("# "))?;
    let summary = lines.find(|line| !line.trim().is_empty()).unwrap_or("");
    Some(format!(
        "{}\n\n{}\n\nDocs: https://docs.sifr.sh/errors/{code}",
        title.trim_start_matches("# "),
        summary,
    ))
}
