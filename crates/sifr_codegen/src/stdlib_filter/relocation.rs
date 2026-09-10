use std::collections::HashSet;
use syn::{GenericArgument, Item, PathArguments, Type};

/// Remove relocated nominal items while retaining a local child's conversion
/// into the canonical parent that replaces the removed definition.
pub(crate) fn strip_relocated_rust_items_by_name(
    rust_code: &str,
    names: &HashSet<&str>,
    local_conversion_sources: &HashSet<String>,
) -> String {
    partition_relocated_rust_items_by_name(rust_code, names, local_conversion_sources).0
}

/// Preserve removed items for the owner receiving late-generated contracts.
pub(crate) fn partition_relocated_rust_items_by_name(
    rust_code: &str,
    names: &HashSet<&str>,
    local_conversion_sources: &HashSet<String>,
) -> (String, Vec<Item>) {
    let Ok(parsed) = syn::parse_file(rust_code) else {
        return (rust_code.to_string(), Vec::new());
    };
    let (kept_items, relocated): (Vec<_>, Vec<_>) = parsed.items.into_iter().partition(|item| {
        let Some(name) = super::parse_item_name(item) else {
            return true;
        };
        !names.contains(name.as_str())
            || is_local_child_into_relocated_parent(item, local_conversion_sources, names)
    });
    (super::render_items(&kept_items), relocated)
}

fn is_local_child_into_relocated_parent(
    item: &Item,
    local_conversion_sources: &HashSet<String>,
    relocated_names: &HashSet<&str>,
) -> bool {
    let Item::Impl(item_impl) = item else {
        return false;
    };
    if item_impl.modifiers.require_empty().is_err() {
        return false;
    }
    let Some((trait_path, _)) = &item_impl.trait_ else {
        return false;
    };
    let Some(segment) = trait_path
        .segments
        .last()
        .filter(|segment| segment.ident == "From")
    else {
        return false;
    };
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    let Some(GenericArgument::Type(Type::Path(source))) = arguments.args.first() else {
        return false;
    };
    let Some(source_name) = source
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
    else {
        return false;
    };
    local_conversion_sources.contains(&source_name)
        && !relocated_names.contains(source_name.as_str())
}
