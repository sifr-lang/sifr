fn rewrite_optional_borrowed_argument(
    argument: &mut syn::Expr,
    borrowed_options: &HashSet<String>,
    owned_options: &HashSet<String>,
) {
    if matches!(argument, syn::Expr::Path(path) if path.path.is_ident("None")) {
        return;
    }
    if matches!(argument, syn::Expr::MethodCall(call)
        if call.method == "as_ref" && call.args.is_empty())
    {
        return;
    }
    if let syn::Expr::Call(some) = argument
        && matches!(some.func.as_ref(), syn::Expr::Path(path) if path.path.is_ident("Some"))
        && some.args.len() == 1
    {
        let value = &mut some.args[0];
        if !matches!(value, syn::Expr::Reference(_)) {
            let inner = value.clone();
            *value = syn::parse_quote!(&#inner);
        }
        return;
    }
    if matches!(argument, syn::Expr::Path(path)
        if path.path.get_ident().is_some_and(|name|
            borrowed_options.contains(&name.to_string())))
    {
        return;
    }
    if matches!(argument, syn::Expr::Path(path)
        if path.path.get_ident().is_some_and(|name|
            owned_options.contains(&name.to_string())))
    {
        let value = argument.clone();
        *argument = syn::parse_quote!(#value.as_ref());
        return;
    }
    if let syn::Expr::Reference(reference) = argument {
        let value = reference.expr.as_ref();
        *argument = syn::parse_quote!((#value).as_ref());
        return;
    }
    if let syn::Expr::MethodCall(clone) = argument
        && clone.method == "clone"
        && clone.args.is_empty()
    {
        let value = clone.receiver.as_ref();
        *argument = syn::parse_quote!((#value).as_ref());
        return;
    }
    let value = argument.clone();
    *argument = syn::parse_quote!((#value).as_ref());
}
