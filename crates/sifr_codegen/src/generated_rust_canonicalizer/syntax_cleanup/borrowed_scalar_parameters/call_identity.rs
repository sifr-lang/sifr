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
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let mut candidates = Vec::new();
    let method_suffix = (segments.len() > 1).then(|| {
        format!(
            "{}::{}#{arguments}",
            segments[segments.len() - 2],
            segments[segments.len() - 1]
        )
    });
    if segments.len() == 1 {
        let mut local = modules.to_vec();
        local.extend(functions.iter().cloned());
        if local.last() == segments.first() {
            candidates.push(format!("function:{}#{arguments}", local.join("::")));
        }
        local.push(segments[0].clone());
        candidates.push(format!("function:{}#{arguments}", local.join("::")));
        if !functions.is_empty() {
            let mut parent = modules.to_vec();
            parent.extend(functions[..functions.len() - 1].iter().cloned());
            parent.push(segments[0].clone());
            candidates.push(format!("function:{}#{arguments}", parent.join("::")));
        }
    } else {
        let mut qualified = segments.clone();
        while matches!(
            qualified.first().map(String::as_str),
            Some("crate" | "self")
        ) {
            qualified.remove(0);
        }
        if !qualified.is_empty() {
            candidates.push(format!("function:{}#{arguments}", qualified.join("::")));
            candidates.push(format!("method:{}#{arguments}", qualified.join("::")));
        }
        let mut local_method = modules.to_vec();
        local_method.extend(segments.iter().rev().take(2).rev().cloned());
        candidates.push(format!("method:{}#{arguments}", local_method.join("::")));
    }
    candidates.sort();
    candidates.dedup();
    let mut matched = candidates.into_iter().filter_map(|key| plans.get(&key));
    if let Some(plan) = matched.next()
        && matched.next().is_none()
    {
        return Some(plan);
    }
    let function_name = &segments[segments.len() - 1];
    let top_level_function = format!("function:{function_name}#{arguments}");
    let function_suffix = format!("::{function_name}#{arguments}");
    let allow_function_fallback = segments.len() == 1;
    let mut fallback = plans.iter().filter_map(|(key, plan)| {
        let function_match = allow_function_fallback
            && key.starts_with("function:")
            && (key == &top_level_function || key.ends_with(&function_suffix));
        let method_match = method_suffix
            .as_ref()
            .is_some_and(|suffix| key.starts_with("method:") && key.ends_with(suffix));
        (function_match || method_match).then_some(plan)
    });
    let plan = fallback.next()?;
    fallback.next().is_none().then_some(plan)
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
