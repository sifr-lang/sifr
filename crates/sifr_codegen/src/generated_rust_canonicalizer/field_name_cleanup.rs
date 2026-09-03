use std::collections::{BTreeMap, BTreeSet};
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

const GENERATED_FIELD_PREFIX: &str = "sifr_generated_";

/// Remove the compiler namespace from generated field members.
///
/// Fields already have a type-owned namespace, so retaining the global generated
/// prefix is redundant and can make every member of a struct share one prefix.
/// The rewrite only touches field declarations and member positions; local values
/// keep their independently canonicalized names.
pub(super) fn canonicalize_generated_field_names(file: &mut syn::File) -> bool {
    let mut collector = FieldNameCollector::default();
    collector.visit_file(file);
    let names = canonical_field_name_map(&collector.names);
    if names.is_empty() {
        return false;
    }
    FieldNameCanonicalizer { names }.visit_file_mut(file);
    true
}

fn canonical_field_name_map(fields: &BTreeSet<String>) -> BTreeMap<String, String> {
    let candidates = fields
        .iter()
        .filter_map(|field| {
            field
                .strip_prefix(GENERATED_FIELD_PREFIX)
                .filter(|candidate| !candidate.is_empty())
                .map(|candidate| (field.clone(), recover_collision_suffix(candidate)))
        })
        .collect::<BTreeMap<_, _>>();
    let mut occupied = fields
        .iter()
        .filter(|field| !candidates.contains_key(*field))
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut canonical = BTreeMap::new();
    for (field, mut candidate) in candidates {
        if occupied.contains(&candidate) {
            candidate.push_str("_field");
            while occupied.contains(&candidate) {
                candidate.push_str("_generated");
            }
        }
        occupied.insert(candidate.clone());
        canonical.insert(field, candidate);
    }
    canonical
}

fn recover_collision_suffix(candidate: &str) -> String {
    let Some((base, encoded)) = candidate.rsplit_once('_') else {
        return candidate.to_string();
    };
    let Some(decoded) = decode_hex_identifier(encoded) else {
        return candidate.to_string();
    };
    if decoded.trim_start_matches('_') == base {
        base.to_string()
    } else {
        candidate.to_string()
    }
}

fn decode_hex_identifier(encoded: &str) -> Option<String> {
    if encoded.is_empty() || !encoded.len().is_multiple_of(2) {
        return None;
    }
    let (pairs, remainder) = encoded.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return None;
    }
    let bytes = pairs
        .iter()
        .map(|pair| {
            let pair = std::str::from_utf8(pair.as_slice()).ok()?;
            u8::from_str_radix(pair, 16).ok()
        })
        .collect::<Option<Vec<_>>>()?;
    String::from_utf8(bytes).ok()
}

#[derive(Default)]
struct FieldNameCollector {
    names: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for FieldNameCollector {
    fn visit_field(&mut self, field: &'ast syn::Field) {
        if let Some(identifier) = &field.ident {
            self.names.insert(identifier.to_string());
        }
        visit::visit_field(self, field);
    }
}

struct FieldNameCanonicalizer {
    names: BTreeMap<String, String>,
}

impl VisitMut for FieldNameCanonicalizer {
    fn visit_field_mut(&mut self, field: &mut syn::Field) {
        if let Some(identifier) = &mut field.ident {
            self.rename(identifier);
        }
        visit_mut::visit_field_mut(self, field);
    }

    fn visit_member_mut(&mut self, member: &mut syn::Member) {
        if let syn::Member::Named(identifier) = member {
            self.rename(identifier);
        }
        visit_mut::visit_member_mut(self, member);
    }

    fn visit_expr_field_mut(&mut self, field: &mut syn::ExprField) {
        self.visit_expr_mut(&mut field.base);
        if let syn::Member::Named(identifier) = &mut field.member {
            self.rename(identifier);
        }
    }

    fn visit_field_value_mut(&mut self, field: &mut syn::FieldValue) {
        let original = match &field.member {
            syn::Member::Named(identifier) => Some(identifier.clone()),
            syn::Member::Unnamed(_) => None,
        };
        if let syn::Member::Named(identifier) = &mut field.member {
            self.rename(identifier);
        }
        if field.colon_token.is_none()
            && let Some(original) = original
            && matches!(&field.member, syn::Member::Named(canonical) if *canonical != original)
        {
            field.colon_token = Some(syn::token::Colon::default());
            field.expr = syn::parse_quote!(#original);
        }
        self.visit_expr_mut(&mut field.expr);
        if let syn::Member::Named(member) = &field.member
            && matches!(field.expr, syn::Expr::Path(ref path)
                if path.qself.is_none() && path.path.is_ident(member))
        {
            field.colon_token = None;
        }
    }

    fn visit_field_pat_mut(&mut self, field: &mut syn::FieldPat) {
        if let syn::Member::Named(identifier) = &mut field.member {
            self.rename(identifier);
        }
        self.visit_pat_mut(&mut field.pat);
    }

    fn visit_macro_mut(&mut self, rust_macro: &mut syn::Macro) {
        rust_macro.tokens = self.rename_macro_field_members(rust_macro.tokens.clone());
        visit_mut::visit_macro_mut(self, rust_macro);
    }
}

impl FieldNameCanonicalizer {
    fn rename(&self, identifier: &mut proc_macro2::Ident) {
        if let Some(canonical) = self.names.get(&identifier.to_string()) {
            *identifier = proc_macro2::Ident::new(canonical, identifier.span());
        }
    }

    fn rename_macro_field_members(
        &self,
        tokens: proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let mut follows_dot = false;
        tokens
            .into_iter()
            .map(|token| {
                let rewritten = match token {
                    proc_macro2::TokenTree::Ident(mut identifier) if follows_dot => {
                        self.rename(&mut identifier);
                        proc_macro2::TokenTree::Ident(identifier)
                    }
                    proc_macro2::TokenTree::Group(group) => {
                        let mut renamed = proc_macro2::Group::new(
                            group.delimiter(),
                            self.rename_macro_field_members(group.stream()),
                        );
                        renamed.set_span(group.span());
                        proc_macro2::TokenTree::Group(renamed)
                    }
                    token => token,
                };
                follows_dot = matches!(&rewritten, proc_macro2::TokenTree::Punct(punct) if punct.as_char() == '.');
                rewritten
            })
            .collect()
    }
}
