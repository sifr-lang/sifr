//! Field spelling belongs to a nominal declaration, not a file's identifier set.
use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet};
use syn::visit_mut::{self, VisitMut};

mod registry;
mod rewrite;
use registry::Registry;
use rewrite::Rewriter;

#[cfg(test)]
mod tests;

pub(super) fn canonicalize_fields(
    sources: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, String> {
    let mut files = sources
        .iter()
        .map(|(module, source)| {
            syn::parse_file(source)
                .map(|file| (module.clone(), file))
                .map_err(|error| {
                    format!("failed to parse assembled generated Rust: module {module}: {error}")
                })
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let registry = Registry::collect(&files);
    let mut result = BTreeMap::new();
    for (module, file) in &mut files {
        let before = file.to_token_stream().to_string();
        let mut rewriter = Rewriter::new(&registry, module);
        rewriter.visit_file_mut(file);
        if let Some(error) = rewriter.error {
            return Err(error);
        }
        let source = if before == file.to_token_stream().to_string() {
            sources[module].clone()
        } else {
            prettyplease::unparse(file)
        };
        result.insert(module.clone(), source);
    }
    Ok(result)
}

fn field_names(fields: &syn::Fields) -> BTreeMap<String, String> {
    let names = fields
        .iter()
        .filter_map(|field| field.ident.as_ref().map(ToString::to_string))
        .collect::<BTreeSet<_>>();
    let mut occupied = names
        .iter()
        .filter(|name| !name.starts_with('_'))
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut result = BTreeMap::new();
    for name in names.iter().filter(|name| name.starts_with('_')) {
        let significant = name.trim_start_matches('_');
        let base = significant.strip_prefix("sifr_").unwrap_or(significant);
        let mut candidate = if base.is_empty() {
            "underscore".to_string()
        } else {
            base.to_string()
        };
        if syn::parse_str::<syn::Ident>(&candidate).is_err() {
            let raw = format!("r#{candidate}");
            candidate = if syn::parse_str::<syn::Ident>(&raw).is_ok() {
                raw
            } else {
                format!("{candidate}_field")
            };
        }
        while occupied.contains(&candidate) {
            candidate.push_str("_field");
        }
        occupied.insert(candidate.clone());
        result.insert(name.clone(), candidate);
    }
    result
}

fn rename(member: &mut syn::Member, names: &BTreeMap<String, String>) -> bool {
    let syn::Member::Named(identifier) = member else {
        return false;
    };
    let Some(name) = names.get(&identifier.to_string()) else {
        return false;
    };
    *identifier = if let Some(raw) = name.strip_prefix("r#") {
        syn::Ident::new_raw(raw, identifier.span())
    } else {
        syn::Ident::new(name, identifier.span())
    };
    true
}

/// Values in shorthand syntax belong to the general identifier namespace;
/// members belong to their resolved owner even when both start with an underscore.
pub(super) fn expand_shorthand(file: &mut syn::File, names: &BTreeMap<String, String>) {
    struct Expand<'a>(&'a BTreeMap<String, String>);
    impl VisitMut for Expand<'_> {
        fn visit_field_value_mut(&mut self, field: &mut syn::FieldValue) {
            if let syn::Member::Named(name) = &field.member
                && self.0.contains_key(&name.to_string())
            {
                field.colon_token = Some(Default::default());
            }
            visit_mut::visit_field_value_mut(self, field);
        }
        fn visit_field_pat_mut(&mut self, field: &mut syn::FieldPat) {
            if let syn::Member::Named(name) = &field.member
                && self.0.contains_key(&name.to_string())
            {
                field.colon_token = Some(Default::default());
            }
            visit_mut::visit_field_pat_mut(self, field);
        }
    }
    Expand(names).visit_file_mut(file);
}

pub(super) fn compact_shorthand(file: &mut syn::File) -> bool {
    struct Compact(bool);
    impl VisitMut for Compact {
        fn visit_field_value_mut(&mut self, field: &mut syn::FieldValue) {
            if field.colon_token.is_some()
                && let syn::Member::Named(name) = &field.member
                && matches!(&field.expr, syn::Expr::Path(path) if path.qself.is_none() && path.path.is_ident(name))
            {
                field.colon_token = None;
                self.0 = true;
            }
            visit_mut::visit_field_value_mut(self, field);
        }
    }
    let mut compact = Compact(false);
    compact.visit_file_mut(file);
    compact.0
}
