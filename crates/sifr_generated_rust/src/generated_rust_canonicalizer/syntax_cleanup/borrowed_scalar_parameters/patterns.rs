fn simple_pattern_name(pattern: &syn::Pat) -> Option<String> {
    match pattern {
        syn::Pat::Ident(binding) if binding.subpat.is_none() => Some(binding.ident.to_string()),
        syn::Pat::Type(typed) => simple_pattern_name(&typed.pat),
        syn::Pat::Paren(paren) => simple_pattern_name(&paren.pat),
        _ => None,
    }
}

fn pattern_binds_name(pattern: &syn::Pat, name: &str) -> bool {
    match pattern {
        syn::Pat::Ident(binding) => {
            binding.ident == name
                || binding
                    .subpat
                    .as_ref()
                    .is_some_and(|(_, subpattern)| pattern_binds_name(subpattern, name))
        }
        syn::Pat::Type(typed) => pattern_binds_name(&typed.pat, name),
        syn::Pat::Paren(paren) => pattern_binds_name(&paren.pat, name),
        syn::Pat::Reference(reference) => pattern_binds_name(&reference.pat, name),
        syn::Pat::Tuple(tuple) => tuple.elems.iter().any(|pat| pattern_binds_name(pat, name)),
        syn::Pat::TupleStruct(tuple) => tuple.elems.iter().any(|pat| pattern_binds_name(pat, name)),
        syn::Pat::Struct(struct_) => struct_
            .fields
            .iter()
            .any(|field| pattern_binds_name(&field.pat, name)),
        syn::Pat::Slice(slice) => slice.elems.iter().any(|pat| pattern_binds_name(pat, name)),
        syn::Pat::Or(or_) => or_.cases.iter().any(|pat| pattern_binds_name(pat, name)),
        syn::Pat::Guard(guard) => pattern_binds_name(&guard.pat, name),
        _ => false,
    }
}
fn consume_tail_operation_parameter(block: &mut syn::Block, name: &str) {
    let shadowed = block.stmts.iter().any(|statement| matches!(statement, syn::Stmt::Local(local) if pattern_binds_name(&local.pat, name)));
    if !shadowed && let Some(syn::Stmt::Expr(tail, _)) = block.stmts.last_mut() {
        consume_terminal_string_parameter(tail, name);
    }
    let [syn::Stmt::Expr(syn::Expr::Call(call), None)] = block.stmts.as_mut_slice() else {
        return;
    };
    if !matches!(call.func.as_ref(), syn::Expr::Path(path)
        if path.path.segments.iter().any(|segment| segment.ident == "ops"))
    {
        return;
    }
    let matching = call
        .args
        .iter()
        .enumerate()
        .filter_map(|(index, argument)| {
            matches!(argument, syn::Expr::Reference(reference)
                if reference.mutability.is_none()
                    && expression_is_name(&reference.expr, name))
            .then_some(index)
        })
        .collect::<Vec<_>>();
    let Some(last) = matching.last().copied() else {
        return;
    };
    let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
    for index in matching {
        call.args[index] = if index == last {
            syn::parse_quote!(#ident)
        } else {
            syn::parse_quote!(#ident.clone())
        };
    }
}

fn consume_terminal_string_parameter(expression: &mut syn::Expr, name: &str) {
    match expression {
        syn::Expr::Return(returned) => {
            if let Some(value) = &mut returned.expr {
                consume_terminal_string_parameter(value, name);
            }
        }
        syn::Expr::Call(call)
            if call.args.len() == 1
                && matches!(call.func.as_ref(), syn::Expr::Path(path) if path.path.is_ident("Ok") || path.path.is_ident("Some")) =>
        {
            consume_terminal_string_parameter(&mut call.args[0], name);
        }
        syn::Expr::MethodCall(call)
            if call.method == "to_string"
                && call.args.is_empty()
                && expression_is_name(&call.receiver, name) =>
        {
            let receiver = &call.receiver;
            *expression = syn::parse_quote!(String::from(#receiver));
        }
        _ => {}
    }
}
