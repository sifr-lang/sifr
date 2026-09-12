use proc_macro2::{Ident, TokenStream, TokenTree};
use std::collections::{BTreeMap, BTreeSet};
use syn::visit::{self, Visit};

use super::identifier_policy::{canonical_identifier_candidate, canonical_name_map};

pub(super) fn project_name_map<'a>(
    sources: impl IntoIterator<Item = &'a str>,
) -> Result<BTreeMap<String, String>, String> {
    let mut identifiers = IdentifierCollector::default();
    for source in sources {
        let file = syn::parse_file(source)
            .map_err(|error| format!("failed to parse assembled generated Rust: {error}"))?;
        identifiers.visit_file(&file);
    }
    Ok(canonical_name_map(&identifiers.names))
}

pub(super) fn canonicalize_identifiers(
    source: &str,
    names: &BTreeMap<String, String>,
) -> Result<String, String> {
    use quote::ToTokens;
    let mut file = syn::parse_file(source)
        .map_err(|error| format!("failed to parse assembled generated Rust: {error}"))?;
    let before = file.to_token_stream().to_string();
    super::field_name_cleanup::expand_shorthand(&mut file, names);
    let expanded;
    let source = if before == file.to_token_stream().to_string() {
        source
    } else {
        expanded = prettyplease::unparse(&file);
        &expanded
    };
    let file = syn::parse_file(source).map_err(|error| error.to_string())?;
    let mut canonicalizer = GeneratedIdentifierCanonicalizer {
        source,
        line_starts: line_starts(source),
        names: names.clone(),
        replacements: Vec::new(),
        error: None,
    };
    canonicalizer.visit_file(&file);
    canonicalizer.finish()
}

/// Return the canonical Rust spelling for one generated identifier.
///
/// Callers use this for physical module paths, which Rust resolves outside the
/// parsed source syntax tree.
#[must_use]
pub fn canonicalize_generated_rust_identifier(identifier: &str) -> String {
    canonical_identifier_candidate(identifier).unwrap_or_else(|| identifier.to_string())
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Replacement {
    start: usize,
    end: usize,
    canonical: String,
}

struct GeneratedIdentifierCanonicalizer<'source> {
    source: &'source str,
    line_starts: Vec<usize>,
    names: BTreeMap<String, String>,
    replacements: Vec<Replacement>,
    error: Option<String>,
}

impl GeneratedIdentifierCanonicalizer<'_> {
    fn collect_ident(&mut self, ident: &Ident) {
        let original = ident.to_string();
        let Some(canonical) = self.names.get(&original).cloned() else {
            return;
        };
        let span = ident.span();
        let Some(start) = self.byte_offset(span.start()) else {
            return;
        };
        let Some(end) = self.byte_offset(span.end()) else {
            return;
        };
        let Some(actual) = self.source.get(start..end) else {
            self.record_error(format!(
                "generated identifier span {start}..{end} is outside the assembled source"
            ));
            return;
        };
        if actual != original && actual.strip_prefix("r#") != Some(original.as_str()) {
            self.record_error(format!(
                "generated identifier span mismatch: parsed {original:?}, found {actual:?}"
            ));
            return;
        }
        self.replacements.push(Replacement {
            start,
            end,
            canonical,
        });
    }

    fn collect_tokens(&mut self, tokens: TokenStream) {
        for token in tokens {
            match token {
                TokenTree::Ident(ident) => self.collect_ident(&ident),
                TokenTree::Group(group) => self.collect_tokens(group.stream()),
                _ => {}
            }
        }
    }

    fn byte_offset(&mut self, location: proc_macro2::LineColumn) -> Option<usize> {
        let Some(line_start) = location
            .line
            .checked_sub(1)
            .and_then(|line| self.line_starts.get(line))
        else {
            self.record_error(format!(
                "generated identifier has invalid source location {}:{}",
                location.line, location.column
            ));
            return None;
        };
        Some(line_start + location.column)
    }

    fn record_error(&mut self, message: String) {
        if self.error.is_none() {
            self.error = Some(message);
        }
    }

    fn finish(mut self) -> Result<String, String> {
        if let Some(error) = self.error {
            return Err(error);
        }
        self.replacements.sort();
        self.replacements.dedup();

        let mut canonical = self.source.to_string();
        for replacement in self.replacements.into_iter().rev() {
            canonical.replace_range(replacement.start..replacement.end, &replacement.canonical);
        }
        Ok(canonical)
    }
}

#[derive(Default)]
struct IdentifierCollector {
    names: BTreeSet<String>,
}

impl IdentifierCollector {
    fn collect_tokens(&mut self, tokens: TokenStream) {
        for token in tokens {
            match token {
                TokenTree::Ident(ident) => {
                    self.names.insert(ident.to_string());
                }
                TokenTree::Group(group) => self.collect_tokens(group.stream()),
                _ => {}
            }
        }
    }
}

impl<'ast> Visit<'ast> for IdentifierCollector {
    fn visit_member(&mut self, _member: &'ast syn::Member) {}
    fn visit_field(&mut self, field: &'ast syn::Field) {
        self.visit_type(&field.ty);
    }
    fn visit_ident(&mut self, ident: &'ast Ident) {
        self.names.insert(ident.to_string());
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        visit::visit_macro(self, rust_macro);
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        } else {
            self.collect_tokens(rust_macro.tokens.clone());
        }
    }

    fn visit_meta_list(&mut self, meta: &'ast syn::MetaList) {
        visit::visit_meta_list(self, meta);
        self.collect_tokens(meta.tokens.clone());
    }
}

impl<'ast> Visit<'ast> for GeneratedIdentifierCanonicalizer<'_> {
    fn visit_member(&mut self, _member: &'ast syn::Member) {}
    fn visit_field(&mut self, field: &'ast syn::Field) {
        self.visit_type(&field.ty);
    }
    fn visit_ident(&mut self, ident: &'ast Ident) {
        self.collect_ident(ident);
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        visit::visit_macro(self, rust_macro);
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        } else {
            self.collect_tokens(rust_macro.tokens.clone());
        }
    }

    fn visit_meta_list(&mut self, meta: &'ast syn::MetaList) {
        visit::visit_meta_list(self, meta);
        self.collect_tokens(meta.tokens.clone());
    }
}

fn line_starts(source: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(
            source
                .bytes()
                .enumerate()
                .filter_map(|(index, byte)| (byte == b'\n').then_some(index + 1)),
        )
        .collect()
}
