use std::collections::HashSet;
use syn::{GenericArgument, Item, PathArguments, Type};

/// Remove relocated nominal items while retaining a local child's conversion
/// into the canonical parent that replaces the removed definition.
pub(crate) fn strip_relocated_rust_items_by_name(
    rust_code: &str,
    names: &HashSet<&str>,
    local_conversion_sources: &HashSet<String>,
) -> crate::CodegenOutcome<String> {
    let parsed = syn::parse_file(rust_code).map_err(|error| {
        crate::CodegenError::new(format!(
            "failed to parse compiler-owned Rust during stdlib nominal relocation: {error}"
        ))
    })?;
    let kept_items = parsed
        .items
        .into_iter()
        .filter(|item| {
            let Some(name) = super::parse_item_name(item) else {
                return true;
            };
            !names.contains(name.as_str())
                || is_local_child_into_relocated_parent(item, local_conversion_sources, names)
        })
        .collect::<Vec<_>>();
    Ok(super::render_items(&kept_items))
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
