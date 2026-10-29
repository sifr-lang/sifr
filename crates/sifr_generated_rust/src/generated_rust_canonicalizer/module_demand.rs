use super::{all_item_identifier_names, item_dependency_names, member_demand};
use std::collections::HashSet;
use syn::visit::{self, Visit};

pub(super) fn parent_items_demanded_by_modules(
    items: &[syn::Item],
    definitions: &HashSet<String>,
    is_crate_root: bool,
) -> HashSet<String> {
    let mut roots = HashSet::new();
    for item in items {
        let syn::Item::Mod(module) = item else {
            continue;
        };
        let Some((_, nested)) = &module.content else {
            continue;
        };
        let mut collector = ParentScopeReferenceCollector {
            definitions,
            roots: &mut roots,
            is_crate_root,
            nested_module_depth: 0,
        };
        for nested_item in nested {
            collector.visit_item(nested_item);
        }
    }
    roots
}

struct ParentScopeReferenceCollector<'scope> {
    definitions: &'scope HashSet<String>,
    roots: &'scope mut HashSet<String>,
    is_crate_root: bool,
    nested_module_depth: usize,
}

impl ParentScopeReferenceCollector<'_> {
    fn collect_segments(&mut self, segments: &[String]) {
        let candidate = match segments {
            [qualifier, candidate, ..] if qualifier == "crate" && self.is_crate_root => {
                Some(candidate)
            }
            [qualifier, candidate, ..] if qualifier == "super" && self.nested_module_depth == 0 => {
                Some(candidate)
            }
            _ => None,
        };
        if let Some(candidate) = candidate
            && self.definitions.contains(candidate)
        {
            self.roots.insert(candidate.clone());
        }
    }

    fn collect_use_tree(&mut self, tree: &syn::UseTree, prefix: &mut Vec<String>) {
        match tree {
            syn::UseTree::Path(path) => {
                prefix.push(path.ident.to_string());
                self.collect_use_tree(&path.tree, prefix);
                prefix.pop();
            }
            syn::UseTree::Name(name) => {
                prefix.push(name.ident.to_string());
                self.collect_segments(prefix);
                prefix.pop();
            }
            syn::UseTree::Rename(rename) => {
                prefix.push(rename.ident.to_string());
                self.collect_segments(prefix);
                prefix.pop();
            }
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.collect_use_tree(tree, prefix);
                }
            }
            syn::UseTree::Glob(_) => {
                let imports_parent = matches!(prefix.as_slice(), [qualifier]
                    if qualifier == "super" && self.nested_module_depth == 0)
                    || matches!(prefix.as_slice(), [qualifier]
                        if qualifier == "crate" && self.is_crate_root);
                if imports_parent {
                    self.roots.extend(self.definitions.iter().cloned());
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for ParentScopeReferenceCollector<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        self.collect_segments(&segments);
        visit::visit_path(self, path);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.collect_use_tree(&item.tree, &mut Vec::new());
    }

    fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
        self.nested_module_depth += 1;
        visit::visit_item_mod(self, module);
        self.nested_module_depth -= 1;
    }
}

pub(super) fn module_roots_from_parent_scope(
    items: &[syn::Item],
    module_index: usize,
    module_name: &str,
    definitions: &HashSet<String>,
    used_names: &HashSet<String>,
) -> HashSet<String> {
    let mut roots = HashSet::new();
    for (index, item) in items.iter().enumerate() {
        if index == module_index {
            continue;
        }
        if let syn::Item::Use(item_use) = item {
            collect_module_use_roots(
                &item_use.tree,
                module_name,
                false,
                definitions,
                used_names,
                &mut roots,
            );
            continue;
        }
        if let syn::Item::Mod(module) = item {
            if let Some((_, nested)) = &module.content {
                let candidates = all_item_identifier_names(item);
                let referenced_names = item_dependency_names(item, &candidates);
                collect_nested_module_use_roots(
                    nested,
                    module_name,
                    definitions,
                    &referenced_names,
                    &mut roots,
                );
            }
            let mut collector = QualifiedModuleReferenceCollector {
                module_name,
                definitions,
                roots: &mut roots,
                block_depth: 0,
            };
            collector.visit_item(item);
            continue;
        }
        let mut collector = QualifiedModuleReferenceCollector {
            module_name,
            definitions,
            roots: &mut roots,
            block_depth: 0,
        };
        collector.visit_item(item);
    }
    roots.retain(|name| definitions.contains(name));
    roots
}

fn collect_nested_module_use_roots(
    items: &[syn::Item],
    module_name: &str,
    definitions: &HashSet<String>,
    referenced_names: &HashSet<String>,
    roots: &mut HashSet<String>,
) {
    for item in items {
        match item {
            syn::Item::Use(item_use) => collect_module_use_roots(
                &item_use.tree,
                module_name,
                false,
                definitions,
                referenced_names,
                roots,
            ),
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &module.content {
                    collect_nested_module_use_roots(
                        nested,
                        module_name,
                        definitions,
                        referenced_names,
                        roots,
                    );
                }
            }
            _ => {}
        }
    }
}

fn collect_module_use_roots(
    tree: &syn::UseTree,
    module_name: &str,
    inside_module: bool,
    definitions: &HashSet<String>,
    used_names: &HashSet<String>,
    roots: &mut HashSet<String>,
) {
    match tree {
        syn::UseTree::Path(path) => collect_module_use_roots(
            &path.tree,
            module_name,
            inside_module || path.ident == module_name,
            definitions,
            used_names,
            roots,
        ),
        syn::UseTree::Name(name)
            if inside_module && used_names.contains(&name.ident.to_string()) =>
        {
            roots.insert(name.ident.to_string());
        }
        syn::UseTree::Rename(rename)
            if inside_module && used_names.contains(&rename.rename.to_string()) =>
        {
            roots.insert(rename.ident.to_string());
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_module_use_roots(
                    item,
                    module_name,
                    inside_module,
                    definitions,
                    used_names,
                    roots,
                );
            }
        }
        syn::UseTree::Glob(_) if inside_module => {
            roots.extend(definitions.intersection(used_names).cloned());
        }
        syn::UseTree::Name(_) | syn::UseTree::Rename(_) | syn::UseTree::Glob(_) => {}
    }
}

struct QualifiedModuleReferenceCollector<'scope> {
    module_name: &'scope str,
    definitions: &'scope HashSet<String>,
    roots: &'scope mut HashSet<String>,
    block_depth: usize,
}

impl<'ast> Visit<'ast> for QualifiedModuleReferenceCollector<'_> {
    fn visit_block(&mut self, block: &'ast syn::Block) {
        self.block_depth += 1;
        visit::visit_block(self, block);
        self.block_depth -= 1;
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        if self.block_depth == 0 {
            return;
        }
        // Block-local imports must resolve even when their binding is unused.
        // Include renamed/grouped bindings as well as every possible glob target.
        let mut imported_names = all_item_identifier_names(&syn::Item::Use(item.clone()));
        imported_names.extend(self.definitions.iter().cloned());
        collect_module_use_roots(
            &item.tree,
            self.module_name,
            false,
            self.definitions,
            &imported_names,
            self.roots,
        );
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        self.visit_path(&rust_macro.path);
        if let Some(arguments) = member_demand::MacroArguments::parse(rust_macro) {
            arguments.visit(self);
        } else {
            self.opaque_tokens(rust_macro.tokens.clone());
        }
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        let segments = path.segments.iter().collect::<Vec<_>>();
        for pair in segments.windows(2) {
            if pair[0].ident == self.module_name {
                let candidate = pair[1].ident.to_string();
                if self.definitions.contains(&candidate) {
                    self.roots.insert(candidate);
                }
            }
        }
        visit::visit_path(self, path);
    }
}

impl QualifiedModuleReferenceCollector<'_> {
    fn opaque_tokens(&mut self, tokens: proc_macro2::TokenStream) {
        use proc_macro2::TokenTree;
        let tokens: Vec<_> = tokens.into_iter().collect();
        for window in tokens.windows(4) {
            if let [
                TokenTree::Ident(module),
                TokenTree::Punct(first),
                TokenTree::Punct(second),
                TokenTree::Ident(name),
            ] = window
                && module == self.module_name
                && first.as_char() == ':'
                && second.as_char() == ':'
                && self.definitions.contains(&name.to_string())
            {
                self.roots.insert(name.to_string());
            }
        }
        for token in tokens {
            if let TokenTree::Group(group) = token {
                self.opaque_tokens(group.stream());
            }
        }
    }
}
