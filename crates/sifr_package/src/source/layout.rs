use sifr_frontend::{SourceProvider, SourceProviderError};
use std::path::Path;

const CANONICAL_PURE_MARKER: &str =
    "// Pure Sifr package marker. Sifr source lives in the sifr.toml source root.";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MarkerValidation {
    PureMarker,
    NonTrivialRust { reason: String },
}

#[must_use]
pub fn canonical_pure_marker_source() -> &'static str {
    CANONICAL_PURE_MARKER
}

#[must_use]
pub fn validate_pure_marker_source(source: &str) -> MarkerValidation {
    let semantic_source = strip_rust_comments_and_whitespace(source);
    if semantic_source.is_empty() {
        return MarkerValidation::PureMarker;
    }

    MarkerValidation::NonTrivialRust {
        reason: classify_non_trivial_marker(&semantic_source).to_string(),
    }
}

pub fn validate_pure_marker_file(
    path: &Path,
    provider: &mut impl SourceProvider,
) -> Result<MarkerValidation, SourceProviderError> {
    provider
        .read_file(path)
        .map(|source| validate_pure_marker_source(source.as_str()))
}

fn strip_rust_comments_and_whitespace(source: &str) -> String {
    let mut output = String::new();
    let mut chars = source.chars().peekable();
    let mut in_line_comment = false;
    let mut block_comment_depth = 0_u32;
    while let Some(ch) = chars.next() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            continue;
        }
        if block_comment_depth > 0 {
            if ch == '/' && chars.peek() == Some(&'*') {
                let _ = chars.next();
                block_comment_depth = block_comment_depth.saturating_add(1);
            } else if ch == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                block_comment_depth = block_comment_depth.saturating_sub(1);
            }
            continue;
        }
        if ch == '/' && chars.peek() == Some(&'/') {
            let _ = chars.next();
            in_line_comment = true;
            continue;
        }
        if ch == '/' && chars.peek() == Some(&'*') {
            let _ = chars.next();
            block_comment_depth = 1;
            continue;
        }
        if !ch.is_whitespace() {
            output.push(ch);
        }
    }
    output
}

fn classify_non_trivial_marker(source: &str) -> &'static str {
    if source.contains("macro_rules!") || source.contains('!') {
        "macro invocation or definition"
    } else if source.contains("mod") {
        "module declaration"
    } else if source.contains("include") {
        "include or generated-code hook"
    } else if source.contains("pubuse") || source.contains("use") {
        "Rust import or re-export"
    } else if source.contains("#[cfg") || source.contains("cfg(") {
        "cfg-driven Rust implementation"
    } else {
        "Rust item or expression"
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_pure_marker_source, MarkerValidation};

    #[test]
    fn comment_only_marker_is_pure() {
        assert_eq!(
            validate_pure_marker_source(
                "// Pure Sifr package marker. Sifr source lives in the sifr.toml source root.\n"
            ),
            MarkerValidation::PureMarker
        );
    }

    #[test]
    fn module_declaration_is_non_trivial() {
        assert!(matches!(
            validate_pure_marker_source("pub mod generated;\n"),
            MarkerValidation::NonTrivialRust { .. }
        ));
    }

    #[test]
    fn nested_block_comments_are_ignored() {
        assert_eq!(
            validate_pure_marker_source("/* marker /* nested */ still marker */\n"),
            MarkerValidation::PureMarker
        );
    }
}
