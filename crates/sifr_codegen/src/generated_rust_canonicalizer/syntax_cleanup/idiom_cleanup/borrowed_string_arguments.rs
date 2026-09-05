//! String parameter planning. Call-site cleanup belongs to lexical type analysis.
use std::collections::HashSet;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

include!("borrowed_string_arguments/parameter_cleanup.rs");

fn simple_pattern_name(pattern: &syn::Pat) -> Option<String> {
    match pattern {
        syn::Pat::Ident(binding) if binding.subpat.is_none() => Some(binding.ident.to_string()),
        syn::Pat::Type(typed) => simple_pattern_name(&typed.pat),
        syn::Pat::Paren(paren) => simple_pattern_name(&paren.pat),
        _ => None,
    }
}
