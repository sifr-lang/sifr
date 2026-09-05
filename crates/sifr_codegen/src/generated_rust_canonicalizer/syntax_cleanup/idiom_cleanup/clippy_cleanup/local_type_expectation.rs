pub(super) fn add_complex_local_type_expectation(local: &mut syn::Local) {
    local.attrs.retain(|attribute| {
        !attribute.path().is_ident("expect")
            || !attribute.meta.to_token_stream().to_string().contains(
                "this generated carrier preserves nested typed Sifr error and tuple structure",
            ) && !attribute
                .meta
                .to_token_stream()
                .to_string()
                .contains("generated Rust preserves the typed Sifr mapping key")
    });
    if let syn::Pat::Type(typed) = &local.pat
        && type_contains_large_nested_result_tuple(&typed.ty)
    {
        let reason = syn::LitStr::new(
            "language necessity: this generated carrier preserves nested typed Sifr error and tuple structure; owner Item 12; remove when the carrier representation changes",
            proc_macro2::Span::call_site(),
        );
        local
            .attrs
            .push(syn::parse_quote!(#[expect(clippy::type_complexity, reason = #reason)]));
    }
}

fn type_contains_large_nested_result_tuple(ty: &syn::Type) -> bool {
    let syn::Type::Path(outer) = ty else {
        return false;
    };
    let Some(outer_result) = outer.path.segments.last() else {
        return false;
    };
    let syn::PathArguments::AngleBracketed(outer_arguments) = &outer_result.arguments else {
        return false;
    };
    let Some(syn::GenericArgument::Type(syn::Type::Path(inner))) = outer_arguments.args.first()
    else {
        return false;
    };
    let Some(inner_result) = inner.path.segments.last() else {
        return false;
    };
    let syn::PathArguments::AngleBracketed(inner_arguments) = &inner_result.arguments else {
        return false;
    };
    matches!(inner_arguments.args.first(), Some(syn::GenericArgument::Type(syn::Type::Tuple(tuple))) if tuple.elems.len() >= 6)
}
