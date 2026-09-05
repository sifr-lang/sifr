use super::implementation::{
    LocalBindingCollector, collect_macro_token_refs_rec, impl_self_type_ident,
};
use proc_macro2::TokenStream;
use std::collections::HashSet;
use syn::visit::{self, Visit};

pub(super) fn referenced_item_names_via_ast(
    item: &syn::Item,
    item_names: &HashSet<String>,
    current_name: &str,
    global_types: &HashSet<String>,
) -> HashSet<String> {
    let mut local_bindings = LocalBindingCollector::default();
    local_bindings.visit_item(item);

    let mut collector = ItemRefCollector::new(
        item_names,
        current_name,
        global_types,
        local_bindings.locals,
    );
    collector.visit_item(item);
    collector.refs
}

struct ItemRefCollector<'a> {
    item_names: &'a HashSet<String>,
    current_name: &'a str,
    global_types: &'a HashSet<String>,
    locals: HashSet<String>,
    refs: HashSet<String>,
}

impl<'a> ItemRefCollector<'a> {
    fn new(
        item_names: &'a HashSet<String>,
        current_name: &'a str,
        global_types: &'a HashSet<String>,
        locals: HashSet<String>,
    ) -> Self {
        Self {
            item_names,
            current_name,
            global_types,
            locals,
            refs: HashSet::new(),
        }
    }

    fn try_insert_ref(&mut self, ident: &str) {
        if ident == self.current_name || self.global_types.contains(ident) {
            return;
        }
        if self.item_names.contains(ident) {
            self.refs.insert(ident.to_string());
        }
    }

    fn collect_macro_token_refs(&mut self, macro_tokens: &TokenStream) {
        let locals = self.locals.clone();
        collect_macro_token_refs_rec(macro_tokens, &locals, |ident| {
            self.try_insert_ref(ident);
        });
    }
}

impl<'ast> Visit<'ast> for ItemRefCollector<'_> {
    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        if let Some((trait_path, _)) = &node.trait_
            && let Some(first) = trait_path.segments.first()
        {
            self.try_insert_ref(&first.ident.to_string());
        }
        if let Some(name) = impl_self_type_ident(node.self_ty.as_ref()) {
            self.try_insert_ref(&name);
        }
        visit::visit_item_impl(self, node);
    }

    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        if let Some(first) = node.path.segments.first() {
            self.try_insert_ref(&first.ident.to_string());
        }
        visit::visit_type_path(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        if let Some(first) = node.segments.first() {
            let ident = first.ident.to_string();
            let is_single_local = node.leading_colon.is_none()
                && node.segments.len() == 1
                && self.locals.contains(&ident);
            if !is_single_local {
                self.try_insert_ref(&ident);
            }
        }
        visit::visit_path(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if node.qself.is_none()
            && let Some(first) = node.path.segments.first()
        {
            let ident = first.ident.to_string();
            let is_single_segment = node.path.segments.len() == 1;
            if !(is_single_segment && self.locals.contains(&ident)) {
                self.try_insert_ref(&ident);
            }
        }
        visit::visit_expr_path(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if let Some(first) = node.path.segments.first() {
            self.try_insert_ref(&first.ident.to_string());
        }
        self.collect_macro_token_refs(&node.tokens);
        for name in crate::generated_rust_canonicalizer::format_capture::names(node) {
            self.try_insert_ref(&name);
        }
        visit::visit_macro(self, node);
    }
}
