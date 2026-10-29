//! String parameter planning. Call-site cleanup belongs to lexical type analysis.
use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/borrowed_string_arguments/parameter_cleanup.rs"
);
include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/borrowed_string_arguments/resolved_calls.rs"
);

fn simple_pattern_name(pattern: &syn::Pat) -> Option<String> {
    match pattern {
        syn::Pat::Ident(binding) if binding.subpat.is_none() => Some(binding.ident.to_string()),
        syn::Pat::Type(typed) => simple_pattern_name(&typed.pat),
        syn::Pat::Paren(paren) => simple_pattern_name(&paren.pat),
        _ => None,
    }
}
