//! Effective public paths, including re-exports, for API-sensitive lint contracts.
use std::collections::{HashMap, HashSet};

pub(super) struct Exports {
    exported: HashSet<String>,
    aliases: HashMap<String, String>,
}

fn key(scope: &[String], name: &str) -> String {
    scope
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(name))
        .collect::<Vec<_>>()
        .join("::")
}

impl Exports {
    pub(super) fn collect(items: &[syn::Item]) -> Self {
        fn imports(
            tree: &syn::UseTree,
            path: &mut Vec<String>,
            output: &mut Vec<(String, Vec<String>)>,
        ) {
            match tree {
                syn::UseTree::Path(part) => {
                    path.push(part.ident.to_string());
                    imports(&part.tree, path, output);
                    path.pop();
                }
                syn::UseTree::Group(group) => {
                    for tree in &group.items {
                        imports(tree, path, output);
                    }
                }
                syn::UseTree::Name(name) => {
                    let mut target = path.clone();
                    let alias = if name.ident == "self" {
                        path.last().cloned().unwrap_or_default()
                    } else {
                        target.push(name.ident.to_string());
                        name.ident.to_string()
                    };
                    output.push((alias, target));
                }
                syn::UseTree::Rename(rename) => {
                    let mut target = path.clone();
                    target.push(rename.ident.to_string());
                    output.push((rename.rename.to_string(), target));
                }
                syn::UseTree::Glob(_) => {
                    output.push(("*".to_owned(), path.clone()));
                }
            }
        }
        fn walk(
            items: &[syn::Item],
            scope: &[String],
            public: &mut Vec<(String, String)>,
            aliases: &mut HashMap<String, String>,
        ) {
            for item in items {
                let declaration = match item {
                    syn::Item::Fn(item) => Some((&item.sig.ident, &item.vis)),
                    syn::Item::Struct(item) => Some((&item.ident, &item.vis)),
                    syn::Item::Enum(item) => Some((&item.ident, &item.vis)),
                    syn::Item::Type(item) => Some((&item.ident, &item.vis)),
                    syn::Item::Trait(item) => Some((&item.ident, &item.vis)),
                    syn::Item::Mod(item) => Some((&item.ident, &item.vis)),
                    _ => None,
                };
                if let Some((name, syn::Visibility::Public(_))) = declaration {
                    public.push((scope.join("::"), key(scope, &name.to_string())));
                }
                if let syn::Item::Use(item) = item
                    && item.leading_colon.is_none()
                {
                    let mut uses = Vec::new();
                    imports(&item.tree, &mut Vec::new(), &mut uses);
                    for (alias, target) in uses {
                        if let Some(target) =
                            super::super::syntax_cleanup::scoped_imports::qualified_path(
                                scope, &target,
                            )
                        {
                            let target = target.join("::");
                            if alias != "*" {
                                aliases.insert(key(scope, &alias), target.clone());
                            }
                            if matches!(item.vis, syn::Visibility::Public(_)) {
                                public.push((scope.join("::"), target));
                            }
                        }
                    }
                }
                if let syn::Item::Mod(item) = item
                    && let Some((_, children)) = &item.content
                {
                    let mut child = scope.to_vec();
                    child.push(item.ident.to_string());
                    walk(children, &child, public, aliases);
                }
            }
        }
        let mut public = Vec::new();
        let mut result = Self {
            exported: HashSet::from([String::new()]),
            aliases: HashMap::new(),
        };
        walk(items, &[], &mut public, &mut result.aliases);
        loop {
            let before = result.exported.len();
            for (scope, target) in &public {
                if result.exported.contains(scope) {
                    result.exported.insert(result.resolve(target));
                }
            }
            if result.exported.len() == before {
                break;
            }
        }
        result
    }

    fn resolve(&self, key: &str) -> String {
        let mut key = key.to_owned();
        let mut seen = HashSet::new();
        loop {
            let replacement = self
                .aliases
                .iter()
                .filter_map(|(alias, target)| {
                    key.strip_prefix(alias)
                        .filter(|suffix| suffix.is_empty() || suffix.starts_with("::"))
                        .map(|suffix| (alias, format!("{target}{suffix}")))
                })
                .max_by_key(|(alias, _)| alias.len());
            let Some((alias, next)) = replacement else {
                break;
            };
            if !seen.insert(alias.clone()) {
                break;
            }
            key = next;
        }
        key
    }

    pub(super) fn function(&self, scope: &[String], name: &syn::Ident) -> bool {
        self.exported.contains(&key(scope, &name.to_string()))
    }

    pub(super) fn owner(&self, scope: &[String], ty: &syn::Type) -> bool {
        let syn::Type::Path(path) = ty else {
            return false;
        };
        let parts = path
            .path
            .segments
            .iter()
            .map(|part| part.ident.to_string())
            .collect::<Vec<_>>();
        super::super::syntax_cleanup::scoped_imports::qualified_path(scope, &parts)
            .is_some_and(|path| self.exported.contains(&self.resolve(&path.join("::"))))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn public_reexports_expose_owners_but_not_private_support_functions() {
        let file: syn::File = syn::parse_quote! {
            mod support { pub fn helper() {} pub struct Owner; }
            pub use support::Owner as PublicOwner;
            pub mod visible { pub fn public_function() {} }
        };
        let exports = super::Exports::collect(&file.items);
        assert!(exports.owner(&[], &syn::parse_quote!(PublicOwner)));
        assert!(exports.owner(&["support".to_owned()], &syn::parse_quote!(Owner)));
        assert!(!exports.function(&["support".to_owned()], &syn::parse_quote!(helper)));
        assert!(exports.function(&["visible".to_owned()], &syn::parse_quote!(public_function)));
    }
}
