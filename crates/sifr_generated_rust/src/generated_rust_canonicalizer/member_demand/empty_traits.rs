//! Remove unused empty compiler traits together with their now-empty implementations.
use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};

pub(super) fn prune(file: &mut syn::File) {
    let mut definitions = Definitions::default();
    definitions.visit_file(file);
    let mut unused = definitions.empty;
    unused.retain(|name| definitions.counts.get(name) == Some(&1));
    loop {
        let candidates = unused.clone();
        References {
            candidates: &candidates,
            unused: &mut unused,
        }
        .visit_file(file);
        if unused.len() == candidates.len() {
            break;
        }
    }
    remove(&mut file.items, &unused);
}

#[derive(Default)]
struct Definitions {
    counts: HashMap<String, usize>,
    empty: HashSet<String>,
}

impl<'ast> Visit<'ast> for Definitions {
    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        let name = item.ident.to_string();
        *self.counts.entry(name.clone()).or_default() += 1;
        if item.items.is_empty() && name.starts_with("SifrGenerated") {
            self.empty.insert(name);
        }
        visit::visit_item_trait(self, item);
    }
}

struct References<'a> {
    candidates: &'a HashSet<String>,
    unused: &'a mut HashSet<String>,
}

impl<'ast> Visit<'ast> for References<'_> {
    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        if !self.candidates.contains(&item.ident.to_string()) {
            visit::visit_item_trait(self, item);
        }
    }
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        if empty_candidate_impl(item, self.candidates).is_none() {
            visit::visit_item_impl(self, item);
        }
    }
    fn visit_path(&mut self, path: &'ast syn::Path) {
        for segment in &path.segments {
            self.unused.remove(&segment.ident.to_string());
        }
        visit::visit_path(self, path);
    }
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        fn collect(tokens: proc_macro2::TokenStream, unused: &mut HashSet<String>) {
            for token in tokens {
                match token {
                    proc_macro2::TokenTree::Ident(name) => {
                        unused.remove(&name.to_string());
                    }
                    proc_macro2::TokenTree::Group(group) => collect(group.stream(), unused),
                    _ => {}
                }
            }
        }
        collect(item.tokens.clone(), self.unused);
    }
}

fn empty_candidate_impl(item: &syn::ItemImpl, candidates: &HashSet<String>) -> Option<String> {
    let path = &item.trait_.as_ref()?.0;
    if !item.items.is_empty() || path.segments.len() != 1 || path.leading_colon.is_some() {
        return None;
    }
    let name = path.segments[0].ident.to_string();
    candidates.contains(&name).then_some(name)
}

fn remove(items: &mut Vec<syn::Item>, unused: &HashSet<String>) {
    items.retain(|item| match item {
        syn::Item::Trait(item) => !unused.contains(&item.ident.to_string()),
        syn::Item::Impl(item) => empty_candidate_impl(item, unused).is_none(),
        _ => true,
    });
    for item in items {
        if let syn::Item::Mod(module) = item
            && let Some((_, items)) = &mut module.content
        {
            remove(items, unused);
        }
    }
}
