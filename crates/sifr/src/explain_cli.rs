use super::cli_model_and_entrypoint::{
    diagnostic_with_code, DiagnosticFormat, EXIT_SUCCESS, EXIT_USAGE_OR_CONFIG,
};
use super::diagnostic_rendering_and_run::render_diagnostics;
use sifr_diagnostics::codes::registry_entry;
use sifr_diagnostics::DiagnosticCode;
#[cfg(debug_assertions)]
use sifr_frontend::{DiskSourceProvider, SourceProvider};
use std::io::{self, Write as _};
#[cfg(debug_assertions)]
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
    if let Some(explanation) = source_tree_diagnostic_explanation(code) {
        return Some(explanation);
    }
    registry_entry(code).map(|entry| {
        format!(
            "{}\n\n{}\n\nDocs: https://docs.sifr.sh/errors/{code}",
            entry.id, entry.summary
        )
    })
}

#[cfg(debug_assertions)]
fn source_tree_diagnostic_explanation(code: &str) -> Option<String> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()?
        .parent()?
        .to_path_buf();
    let path = repo_root.join("docs/errors").join(format!("{code}.mdx"));
    let text = DiskSourceProvider::new().read_file(&path).ok()?;
    let title = mdx_frontmatter_value(text.as_str(), "sidebarTitle: ")?;
    let summary = mdx_frontmatter_value(text.as_str(), "description: ")?;
    Some(format!(
        "{title}\n\n{summary}\n\nDocs: https://docs.sifr.sh/errors/{code}"
    ))
}

#[cfg(debug_assertions)]
fn mdx_frontmatter_value<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let mut lines = text.lines();
    if lines.next()? != "---" {
        return None;
    }
    for line in lines {
        if line == "---" {
            return None;
        }
        if let Some(value) = line.strip_prefix(key) {
            return Some(value.trim().trim_matches('"')).filter(|value| !value.is_empty());
        }
    }
    None
}

#[cfg(not(debug_assertions))]
fn source_tree_diagnostic_explanation(_code: &str) -> Option<String> {
    None
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use super::source_tree_diagnostic_explanation;

    #[test]
    fn debug_explanation_reads_generated_mdx_frontmatter() {
        let explanation = source_tree_diagnostic_explanation("SIFR-IMPORT-0001")
            .expect("generated diagnostic MDX should be readable");
        assert_eq!(
            explanation,
            "SIFR-IMPORT-0001\n\nForbidden private sysroot declaration import.\n\nDocs: https://docs.sifr.sh/errors/SIFR-IMPORT-0001"
        );
    }
}
