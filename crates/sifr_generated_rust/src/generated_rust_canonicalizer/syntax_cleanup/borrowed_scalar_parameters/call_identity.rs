fn scoped_signature_key(
    modules: &[String],
    functions: &[String],
    owner: Option<&str>,
    signature: &syn::Signature,
) -> String {
    let arguments = signature
        .inputs
        .iter()
        .filter(|argument| matches!(argument, syn::FnArg::Typed(_)))
        .count();
    let mut path = modules.to_vec();
    path.extend(functions.iter().cloned());
    let kind = if let Some(owner) = owner {
        path.push(owner.to_string());
        "method"
    } else {
        "function"
    };
    path.push(signature.ident.to_string());
    format!("{kind}:{}#{arguments}", path.join("::"))
}

fn call_plan<'plans>(
    plans: &'plans HashMap<String, ScalarBorrowPlan>,
    modules: &[String],
    functions: &[String],
    path: &syn::Path,
    arguments: usize,
) -> Option<&'plans ScalarBorrowPlan> {
    if path.leading_colon.is_some() {
        return None;
    }
    let segments = path.segments.iter().map(|part| part.ident.to_string()).collect::<Vec<_>>();
    if segments.len() == 1 {
        // Rust block-local functions shadow enclosing declarations, never sibling modules.
        for depth in (0..=functions.len()).rev() {
            let mut local = modules.to_vec();
            local.extend(functions[..depth].iter().cloned());
            local.extend(segments.iter().cloned());
            if let Some(plan) = plans.get(&format!("function:{}#{arguments}", local.join("::"))) {
                return Some(plan);
            }
        }
        return None;
    }
    let qualified = super::scoped_imports::qualified_path(modules, &segments)?;
    ["function", "method"].into_iter().find_map(|kind| {
        plans.get(&format!("{kind}:{}#{arguments}", qualified.join("::")))
    })
}

fn type_owner_name(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(path) => path.path.segments.last().map_or_else(
            || quote::quote!(#ty).to_string(),
            |segment| segment.ident.to_string(),
        ),
        _ => quote::quote!(#ty).to_string(),
    }
}

fn typed_input(signature: &syn::Signature, index: usize) -> Option<&syn::PatType> {
    signature
        .inputs
        .iter()
        .filter_map(|argument| {
            let syn::FnArg::Typed(parameter) = argument else {
                return None;
            };
            Some(parameter)
        })
        .nth(index)
}

fn typed_input_mut(signature: &mut syn::Signature, index: usize) -> Option<&mut syn::PatType> {
    signature
        .inputs
        .iter_mut()
        .filter_map(|argument| {
            let syn::FnArg::Typed(parameter) = argument else {
                return None;
            };
            Some(parameter)
        })
        .nth(index)
}
