// Structural Rust type queries used by lexical cleanup.
fn same_type(left: &syn::Type, right: &syn::Type) -> bool {
    left.to_token_stream().to_string() == right.to_token_stream().to_string()
}

fn named(ty: &syn::Type, name: &str) -> bool {
    matches!(ty, syn::Type::Path(path) if path.path.is_ident(name))
}

fn generic<'a>(ty: &'a syn::Type, name: &str) -> Option<&'a syn::Type> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    // A matching final segment does not establish standard-container identity.
    // Qualified external lookalikes must retain their declared operations.
    if path.qself.is_some() || path.path.leading_colon.is_some() || path.path.segments.len() != 1 {
        return None;
    }
    let segment = path.path.segments.last()?;
    if segment.ident != name {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    match args.args.first()? {
        syn::GenericArgument::Type(ty) => Some(ty),
        _ => None,
    }
}

fn unreference(ty: &syn::Type) -> &syn::Type {
    if let syn::Type::Reference(reference) = ty {
        unreference(&reference.elem)
    } else {
        ty
    }
}

fn callable_inputs(ty: &syn::Type) -> Option<Vec<syn::Type>> {
    match ty {
        syn::Type::Reference(reference) => callable_inputs(&reference.elem),
        syn::Type::FnPtr(function) => Some(
            function
                .inputs
                .iter()
                .map(|input| input.ty.clone())
                .collect(),
        ),
        syn::Type::ImplTrait(trait_) => bound_inputs(&trait_.bounds),
        syn::Type::TraitObject(trait_) => bound_inputs(&trait_.bounds),
        _ => generic(ty, "Box").and_then(callable_inputs),
    }
}

fn bound_inputs(
    bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::Token![+]>,
) -> Option<Vec<syn::Type>> {
    bounds.iter().find_map(|bound| {
        let syn::TypeParamBound::Trait(bound) = bound else {
            return None;
        };
        let segment = bound.path.segments.last()?;
        if !matches!(
            segment.ident.to_string().as_str(),
            "Fn" | "FnMut" | "FnOnce"
        ) {
            return None;
        }
        let syn::PathArguments::Parenthesized(arguments) = &segment.arguments else {
            return None;
        };
        Some(
            arguments
                .inputs
                .iter()
                .map(|input| input.ty.clone())
                .collect(),
        )
    })
}

fn parameter_is_concrete(input: &syn::FnArg, generics: &syn::Generics) -> bool {
    let mut names = std::collections::HashSet::new();
    super::mutability_cleanup::collect_token_identifiers(input.to_token_stream(), &mut names);
    generics.params.iter().all(|parameter| {
        let name = match parameter {
            syn::GenericParam::Type(parameter) => &parameter.ident,
            syn::GenericParam::Const(parameter) => &parameter.ident,
            syn::GenericParam::Lifetime(parameter) => &parameter.lifetime.ident,
        };
        !names.contains(&name.to_string())
    })
}
