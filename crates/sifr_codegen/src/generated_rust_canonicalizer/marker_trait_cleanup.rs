use std::collections::HashSet;

use super::{item_definition_name, item_dependency_names};

pub(super) fn prune_unused_marker_traits(items: &mut Vec<syn::Item>) {
    for item in items.iter_mut() {
        if let syn::Item::Mod(module) = item
            && let Some((_, nested)) = &mut module.content
        {
            prune_unused_marker_traits(nested);
        }
    }

    let candidates = items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Trait(trait_) if trait_.items.is_empty() => Some(trait_.ident.to_string()),
            _ => None,
        })
        .collect::<HashSet<_>>();
    if candidates.is_empty() {
        return;
    }
    let definitions = items
        .iter()
        .filter_map(item_definition_name)
        .collect::<HashSet<_>>();
    let used = candidates
        .iter()
        .filter(|candidate| {
            items.iter().any(|item| {
                !is_marker_trait_item(item, candidate)
                    && !is_marker_trait_impl(item, candidate)
                    && item_dependency_names(item, &definitions).contains(*candidate)
            })
        })
        .cloned()
        .collect::<HashSet<_>>();
    items.retain(|item| {
        !candidates.iter().any(|candidate| {
            !used.contains(candidate)
                && (is_marker_trait_item(item, candidate) || is_marker_trait_impl(item, candidate))
        })
    });
}

fn is_marker_trait_item(item: &syn::Item, name: &str) -> bool {
    matches!(item, syn::Item::Trait(trait_) if trait_.ident == name)
}

fn is_marker_trait_impl(item: &syn::Item, name: &str) -> bool {
    matches!(item, syn::Item::Impl(item_impl)
        if item_impl.trait_.as_ref().is_some_and(|(path, _)|
            path.segments.last().is_some_and(|segment| segment.ident == name)))
}
