use proc_macro2::{TokenStream, TokenTree};
use std::collections::{BTreeSet, HashSet};
use syn::parse::Parser;
use syn::visit::{self, Visit};

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
    let mut bindings = BindingCollector::default();
    bindings.visit_item(item);
    let mut collector = ScopedItemReferenceCollector {
        bindings: &bindings.names,
        definitions,
        references: HashSet::new(),
    };
    collector.visit_item(item);
    collector.references
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
    }

    fn visit_meta_list(&mut self, meta: &'ast syn::MetaList) {
        visit::visit_meta_list(self, meta);
        self.collect_tokens(meta.tokens.clone());
    }
}

#[derive(Default)]
struct BindingCollector {
    names: HashSet<String>,
}

impl<'ast> Visit<'ast> for BindingCollector {
    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        self.names.insert(pattern.ident.to_string());
        visit::visit_pat_ident(self, pattern);
    }

    fn visit_generic_param(&mut self, parameter: &'ast syn::GenericParam) {
        match parameter {
            syn::GenericParam::Type(type_) => {
                self.names.insert(type_.ident.to_string());
            }
            syn::GenericParam::Const(const_) => {
                self.names.insert(const_.ident.to_string());
            }
            syn::GenericParam::Lifetime(_) => {}
        }
        visit::visit_generic_param(self, parameter);
    }
}

struct ScopedItemReferenceCollector<'scope> {
    bindings: &'scope HashSet<String>,
    definitions: &'scope HashSet<String>,
    references: HashSet<String>,
}

impl ScopedItemReferenceCollector<'_> {
    fn collect_path(&mut self, path: &syn::Path) {
        let segments = path.segments.iter().collect::<Vec<_>>();
        let candidate = segments.first().and_then(|first| {
            if matches!(first.ident.to_string().as_str(), "crate" | "self" | "super") {
                segments.get(1)
            } else {
                Some(first)
            }
        });
        if let Some(segment) = candidate {
            let name = segment.ident.to_string();
            if self.definitions.contains(&name) && !self.bindings.contains(&name) {
                self.references.insert(name);
            }
        }
    }

    fn collect_macro_tokens(&mut self, tokens: TokenStream) {
        let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
        if let Ok(expressions) = parser.parse2(tokens.clone()) {
            for expression in &expressions {
                self.visit_expr(expression);
            }
            return;
        }
        let tokens = tokens.into_iter().collect::<Vec<_>>();
        for (index, token) in tokens.iter().enumerate() {
            if let TokenTree::Group(group) = token {
                self.collect_macro_tokens(group.stream());
                continue;
            }
            let TokenTree::Ident(identifier) = token else {
                continue;
            };
            let preceded_by_member_access = matches!(tokens.get(index.wrapping_sub(1)), Some(TokenTree::Punct(punctuation)) if punctuation.as_char() == '.');
            let followed_by_field_separator = matches!(tokens.get(index + 1), Some(TokenTree::Punct(first)) if first.as_char() == ':')
                && !matches!(tokens.get(index + 2), Some(TokenTree::Punct(second)) if second.as_char() == ':');
            let name = identifier.to_string();
            if !preceded_by_member_access
                && !followed_by_field_separator
                && self.definitions.contains(&name)
                && !self.bindings.contains(&name)
            {
                self.references.insert(name);
            }
        }
    }
}

impl<'ast> Visit<'ast> for ScopedItemReferenceCollector<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.collect_path(path);
        visit::visit_path(self, path);
    }

    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        visit::visit_expr_path(self, expression);
    }

    fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
        visit::visit_type_path(self, ty);
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        for name in super::format_capture::names(rust_macro) {
            if self.definitions.contains(&name) && !self.bindings.contains(&name) {
                self.references.insert(name);
            }
        }
        self.collect_macro_tokens(rust_macro.tokens.clone());
        visit::visit_macro(self, rust_macro);
    }
}
