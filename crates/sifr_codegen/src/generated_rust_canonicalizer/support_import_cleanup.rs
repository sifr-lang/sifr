//! Resolve compiler-owned imports after all reference-removing canonicalization.
use std::collections::BTreeMap;

pub(super) fn refresh_support_imports(
    mut sources: BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, String> {
    for _ in 0..16 {
        let refreshed = refresh_once(sources.clone())?;
        if refreshed == sources {
            return Ok(sources);
        }
        sources = refreshed
            .into_iter()
            .map(|(module, source)| {
                if sources.get(&module) == Some(&source) {
                    Ok((module, source))
                } else {
                    super::canonicalize_named_source(source).map(|source| (module, source))
                }
            })
            .collect::<Result<_, _>>()?;
    }
    Err("generated support imports did not reach a fixed point".to_string())
}

fn refresh_once(sources: BTreeMap<String, String>) -> Result<BTreeMap<String, String>, String> {
    let owner = super::canonicalize_generated_rust_identifier("__sifr_generated_support");
    let mut support = None;
    for source in sources.values() {
        let file = syn::parse_file(source).map_err(|error| error.to_string())?;
        for item in file.items {
            if let syn::Item::Mod(module) = item
                && module.ident == owner
                && let Some((_, items)) = module.content
            {
                let mut file = syn::parse_file("").map_err(|error| error.to_string())?;
                file.items = items;
                support = Some(prettyplease::unparse(&file));
            }
        }
    }
    let Some(support) = support else {
        return Ok(sources);
    };
    sources
        .into_iter()
        .map(|(module, source)| {
            let mut file = syn::parse_file(&source).map_err(|error| error.to_string())?;
            if refresh_scope(&mut file.items, &owner, &support)? {
                Ok((module, prettyplease::unparse(&file)))
            } else {
                Ok((module, source))
            }
        })
        .collect()
}

fn refresh_scope(items: &mut Vec<syn::Item>, owner: &str, support: &str) -> Result<bool, String> {
    let mut changed = false;
    for item in items.iter_mut() {
        if let syn::Item::Mod(module) = item
            && let Some((_, nested)) = &mut module.content
        {
            if module.ident == owner {
                changed |= refresh_support_root_imports(nested)?;
            } else {
                changed |= refresh_scope(nested, owner, support)?;
            }
        }
    }
    let Some(index) = items.iter().position(|item| support_import(item, owner)) else {
        return Ok(changed);
    };
    let original = items[index].clone();
    items.retain(|item| !support_import(item, owner));
    let mut consumer = syn::parse_file("").map_err(|error| error.to_string())?;
    consumer.items = items.clone();
    let import = crate::generated_visibility::generated_support_import(
        &prettyplease::unparse(&consumer),
        support,
    )
    .replace("__sifr_generated_support", owner);
    if import.is_empty() {
        return Ok(true);
    }
    let replacement: syn::Item = syn::parse_str(&import).map_err(|error| error.to_string())?;
    use quote::ToTokens;
    changed |= original.to_token_stream().to_string() != replacement.to_token_stream().to_string();
    items.insert(index.min(items.len()), replacement);
    Ok(changed)
}

// Support's reverse edge imports project-owned nominal types from the root.
// A support helper can disappear during pruning, taking its last type reference
// with it. Resolve this compiler-owned flat import at the same final boundary.
fn refresh_support_root_imports(items: &mut Vec<syn::Item>) -> Result<bool, String> {
    let mut changed = false;
    for index in (0..items.len()).rev() {
        let syn::Item::Use(import) = &items[index] else {
            continue;
        };
        let syn::UseTree::Path(root) = &import.tree else {
            continue;
        };
        let syn::UseTree::Group(group) = root.tree.as_ref() else {
            continue;
        };
        if root.ident != "crate"
            || !group
                .items
                .iter()
                .all(|item| matches!(item, syn::UseTree::Name(_)))
        {
            continue;
        }
        let names = group
            .items
            .iter()
            .filter_map(|item| {
                if let syn::UseTree::Name(name) = item {
                    Some(name.ident.to_string())
                } else {
                    None
                }
            })
            .collect();
        let mut consumer = syn::parse_file("").map_err(|error| error.to_string())?;
        consumer.items = items
            .iter()
            .enumerate()
            .filter(|(position, _)| *position != index)
            .map(|(_, item)| item.clone())
            .collect();
        let required = crate::stdlib_filter::rust_source_unqualified_item_names(
            &prettyplease::unparse(&consumer),
            &names,
        )?;
        if required == names {
            continue;
        }
        changed = true;
        if required.is_empty() {
            items.remove(index);
        } else {
            let mut names = required.into_iter().collect::<Vec<_>>();
            names.sort();
            items[index] = syn::parse_str(&format!("use crate::{{{}}};", names.join(",")))
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(changed)
}

fn support_import(item: &syn::Item, owner: &str) -> bool {
    let syn::Item::Use(item) = item else {
        return false;
    };
    let syn::UseTree::Path(root) = &item.tree else {
        return false;
    };
    let syn::UseTree::Path(module) = root.tree.as_ref() else {
        return false;
    };
    root.ident == "crate" && module.ident == owner
}
