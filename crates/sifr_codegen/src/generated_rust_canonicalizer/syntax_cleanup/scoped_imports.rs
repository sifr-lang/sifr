//! Resolve declared Rust imports, including re-exports, without basename lookup.
use std::collections::HashMap;

pub(super) fn expand<T: Clone>(file: &syn::File, definitions: &mut HashMap<String, T>) {
    let mut aliases = Vec::new();
    collect(&file.items, &[], &mut aliases);
    for _ in 0..aliases.len() {
        let mut additions = Vec::new();
        for (alias, target) in &aliases {
            for (key, value) in definitions.iter() {
                if let Some(suffix) = key.strip_prefix(target)
                    && (suffix.is_empty() || suffix.starts_with("::") || suffix.starts_with('#'))
                {
                    let key = format!("{alias}{suffix}");
                    if !definitions.contains_key(&key) {
                        additions.push((key, value.clone()));
                    }
                }
            }
        }
        if additions.is_empty() {
            break;
        }
        definitions.extend(additions);
    }
}

fn collect(items: &[syn::Item], scope: &[String], aliases: &mut Vec<(String, String)>) {
    for item in items {
        match item {
            syn::Item::Use(import) if import.leading_colon.is_none() => {
                use_tree(&import.tree, &mut Vec::new(), scope, aliases);
            }
            syn::Item::Mod(module) => {
                if let Some((_, items)) = &module.content {
                    let mut nested = scope.to_vec();
                    nested.push(module.ident.to_string());
                    collect(items, &nested, aliases);
                }
            }
            _ => {}
        }
    }
}

fn use_tree(
    tree: &syn::UseTree,
    path: &mut Vec<String>,
    scope: &[String],
    aliases: &mut Vec<(String, String)>,
) {
    match tree {
        syn::UseTree::Path(part) => {
            path.push(part.ident.to_string());
            use_tree(&part.tree, path, scope, aliases);
            path.pop();
        }
        syn::UseTree::Group(group) => {
            for tree in &group.items {
                use_tree(tree, path, scope, aliases);
            }
        }
        syn::UseTree::Name(name) => record(
            path,
            &name.ident.to_string(),
            &name.ident.to_string(),
            scope,
            aliases,
        ),
        syn::UseTree::Rename(rename) => record(
            path,
            &rename.ident.to_string(),
            &rename.rename.to_string(),
            scope,
            aliases,
        ),
        syn::UseTree::Glob(_) => {}
    }
}

fn record(
    path: &[String],
    name: &str,
    alias: &str,
    scope: &[String],
    aliases: &mut Vec<(String, String)>,
) {
    let mut relative = path.to_vec();
    if name != "self" {
        relative.push(name.to_string());
    }
    let mut absolute = scope.to_vec();
    match relative.first().map(String::as_str) {
        Some("crate") => {
            absolute.clear();
            relative.remove(0);
        }
        Some("self") => {
            relative.remove(0);
        }
        Some("super") => {
            while relative.first().is_some_and(|part| part == "super") {
                absolute.pop();
                relative.remove(0);
            }
        }
        _ => {}
    }
    absolute.extend(relative);
    let mut local = scope.to_vec();
    local.push(if alias == "self" {
        path.last().cloned().unwrap_or_default()
    } else {
        alias.to_string()
    });
    aliases.push((local.join("::"), absolute.join("::")));
}
