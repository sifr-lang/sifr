use super::implementation::{
    LocalBindingCollector, collect_macro_token_refs_rec, collect_use_paths,
};
use std::collections::HashSet;
use syn::visit::{self, Visit};

/// Return references from `rust_code` to names defined by a separate generated
/// support owner. Declaration identifiers and local bindings are not references.
pub(crate) fn rust_source_referenced_item_names(
    rust_code: &str,
    candidate_names: &HashSet<String>,
) -> HashSet<String> {
    let Ok(parsed) = syn::parse_file(rust_code) else {
        return HashSet::new();
    };
    let mut local_bindings = LocalBindingCollector::default();
    local_bindings.visit_file(&parsed);
    let mut collector = ExternalItemRefCollector {
        candidate_names,
        locals: local_bindings.locals,
        refs: HashSet::new(),
    };
    collector.visit_file(&parsed);
    collector.refs
}

struct ExternalItemRefCollector<'a> {
    candidate_names: &'a HashSet<String>,
    locals: HashSet<String>,
    refs: HashSet<String>,
}

impl ExternalItemRefCollector<'_> {
    fn collect_path(&mut self, path: &syn::Path) {
        let single_local = path.leading_colon.is_none()
            && path.segments.len() == 1
            && self.locals.contains(&path.segments[0].ident.to_string());
        if single_local {
            return;
        }
        self.refs.extend(
            path.segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .filter(|name| self.candidate_names.contains(name)),
        );
    }
}

impl<'ast> Visit<'ast> for ExternalItemRefCollector<'_> {
    fn visit_item_use(&mut self, item_use: &'ast syn::ItemUse) {
        let mut paths = Vec::new();
        collect_use_paths(&item_use.tree, &mut Vec::new(), &mut paths);
        for path in paths {
            self.refs.extend(
                path.into_iter()
                    .filter(|name| self.candidate_names.contains(name)),
            );
        }
        visit::visit_item_use(self, item_use);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.collect_path(path);
        visit::visit_path(self, path);
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        let locals = self.locals.clone();
        collect_macro_token_refs_rec(&rust_macro.tokens, &locals, |name| {
            if self.candidate_names.contains(name) {
                self.refs.insert(name.to_string());
            }
        });
        visit::visit_macro(self, rust_macro);
    }
}
