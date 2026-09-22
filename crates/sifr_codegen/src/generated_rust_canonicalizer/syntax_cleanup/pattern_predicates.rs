pub(crate) fn is_wildcard_result_pattern(pattern: &syn::Pat, variant: &str) -> bool {
    let syn::Pat::TupleStruct(tuple) = pattern else {
        return false;
    };
    tuple
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == variant)
        && matches!(tuple.elems.first(), Some(syn::Pat::Wild(_)))
}

pub(super) fn is_wildcard_option_pattern(pattern: &syn::Pat, variant: &str) -> bool {
    is_wildcard_result_pattern(pattern, variant)
}

pub(super) fn is_none_pattern(pattern: &syn::Pat) -> bool {
    matches!(pattern,
        syn::Pat::Path(path) if path.path.is_ident("None")
    ) || matches!(pattern,
        syn::Pat::Ident(binding)
            if binding.ident == "None" && binding.subpat.is_none()
    )
}
