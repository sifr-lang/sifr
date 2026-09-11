//! Absolute Rust 2018 paths belong to the extern prelude, not generated owners.
//! Their generic arguments and locally introduced import aliases still belong
//! to the current crate and participate in its collision-aware spelling map.
use proc_macro2::{TokenStream, TokenTree};
use std::collections::BTreeMap;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

pub(super) fn visit_path<'ast, V: Visit<'ast>>(visitor: &mut V, path: &'ast syn::Path) {
    if path.leading_colon.is_some() {
        for segment in &path.segments {
            visitor.visit_path_arguments(&segment.arguments);
        }
    } else {
        visit::visit_path(visitor, path);
    }
}

pub(super) fn visit_import<'ast, V: Visit<'ast>>(visitor: &mut V, item: &'ast syn::ItemUse) {
    fn bindings<'ast, V: Visit<'ast>>(visitor: &mut V, tree: &'ast syn::UseTree) {
        match tree {
            syn::UseTree::Path(path) => bindings(visitor, &path.tree),
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    bindings(visitor, tree);
                }
            }
            syn::UseTree::Name(name) => visitor.visit_ident(&name.ident),
            syn::UseTree::Rename(rename) => visitor.visit_ident(&rename.rename),
            syn::UseTree::Glob(_) => {}
        }
    }
    if item.leading_colon.is_none() {
        visit::visit_item_use(visitor, item);
        return;
    }
    for attribute in &item.attrs {
        visitor.visit_attribute(attribute);
    }
    visitor.visit_visibility(&item.vis);
    bindings(visitor, &item.tree);
}

pub(super) fn alias_external_imports(file: &mut syn::File, names: &BTreeMap<String, String>) {
    struct Aliases<'a>(&'a BTreeMap<String, String>);
    impl Aliases<'_> {
        fn tree(&self, tree: &mut syn::UseTree) {
            match tree {
                syn::UseTree::Path(path) => self.tree(&mut path.tree),
                syn::UseTree::Group(group) => {
                    for tree in &mut group.items {
                        self.tree(tree);
                    }
                }
                syn::UseTree::Name(name) if self.0.contains_key(&name.ident.to_string()) => {
                    *tree = syn::UseTree::Rename(syn::UseRename {
                        ident: name.ident.clone(),
                        as_token: Default::default(),
                        rename: name.ident.clone(),
                    });
                }
                _ => {}
            }
        }
    }
    impl VisitMut for Aliases<'_> {
        fn visit_item_use_mut(&mut self, item: &mut syn::ItemUse) {
            if item.leading_colon.is_some() {
                self.tree(&mut item.tree);
            }
            visit_mut::visit_item_use_mut(self, item);
        }
    }
    Aliases(names).visit_file_mut(file);
}

// Unparsed macro bodies still contain ordinary qualified token paths (for
// example vec![::dependency::__make(local); count]). Protect those segments,
// while recursively visiting argument groups in the caller.
pub(super) fn classify_tokens(tokens: TokenStream) -> Vec<(TokenTree, bool)> {
    let tokens = tokens.into_iter().collect::<Vec<_>>();
    let colon =
        |index: usize| matches!(tokens.get(index), Some(TokenTree::Punct(p)) if p.as_char() == ':');
    tokens
        .iter()
        .enumerate()
        .map(|(index, token)| {
            let mut start = index;
            while start >= 3
                && colon(start - 1)
                && colon(start - 2)
                && matches!(tokens[start - 3], TokenTree::Ident(_))
            {
                start -= 3;
            }
            let external = matches!(token, TokenTree::Ident(_))
                && start >= 2
                && colon(start - 1)
                && colon(start - 2);
            (token.clone(), external)
        })
        .collect()
}
