use proc_macro2::{TokenStream, TokenTree};
use std::collections::{BTreeSet, HashSet};
use syn::visit::{self, Visit};

mod scoped_references;

pub(super) fn item_definition_name(item: &syn::Item) -> Option<String> {
    match item {
        syn::Item::Const(item) => Some(item.ident.to_string()),
        syn::Item::Enum(item) => Some(item.ident.to_string()),
        syn::Item::Fn(item) => Some(item.sig.ident.to_string()),
        syn::Item::Impl(item) => impl_self_type_name(item.self_ty.as_ref()),
        syn::Item::Static(item) => Some(item.ident.to_string()),
        syn::Item::Struct(item) => Some(item.ident.to_string()),
        syn::Item::Trait(item) => Some(item.ident.to_string()),
        syn::Item::Type(item) => Some(item.ident.to_string()),
        syn::Item::Union(item) => Some(item.ident.to_string()),
        _ => None,
    }
}

pub(super) fn impl_self_type_name(ty: &syn::Type) -> Option<String> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    path.path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

pub(super) fn item_dependency_names(
    item: &syn::Item,
    definitions: &HashSet<String>,
) -> HashSet<String> {
    scoped_references::item_references(item, definitions)
}

pub(super) fn all_item_identifier_names(item: &syn::Item) -> HashSet<String> {
    let mut collector = IdentifierCollector::default();
    collector.visit_item(item);
    collector.names.into_iter().collect()
}

#[derive(Default)]
pub(super) struct IdentifierCollector {
    pub(super) names: BTreeSet<String>,
}

impl IdentifierCollector {
    fn collect_tokens(&mut self, tokens: TokenStream) {
        for token in tokens {
            match token {
                TokenTree::Ident(identifier) => {
                    self.names.insert(identifier.to_string());
                }
                TokenTree::Group(group) => self.collect_tokens(group.stream()),
                _ => {}
            }
        }
    }
}

impl<'ast> Visit<'ast> for IdentifierCollector {
    fn visit_ident(&mut self, identifier: &'ast proc_macro2::Ident) {
        self.names.insert(identifier.to_string());
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        visit::visit_macro(self, rust_macro);
        self.names.extend(super::format_capture::names(rust_macro));
        self.collect_tokens(rust_macro.tokens.clone());
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        }
    }

    fn visit_meta_list(&mut self, meta: &'ast syn::MetaList) {
        visit::visit_meta_list(self, meta);
        self.collect_tokens(meta.tokens.clone());
    }
}
