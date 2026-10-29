pub(super) fn rewrite_identity_string_concat(
    expression: &mut syn::Expr,
    owned: &HashSet<String>,
    borrowed: &HashSet<String>,
    capacity_is_pure: impl Fn(&syn::Expr) -> bool,
) -> bool {
    let syn::Expr::Block(block) = expression else {
        return false;
    };
    let Some(syn::Stmt::Local(initial)) = block.block.stmts.first() else {
        return false;
    };
    let Some(buffer) = simple_pattern_name(&initial.pat) else {
        return false;
    };
    let Some(initializer) = &initial.init else {
        return false;
    };
    if !is_string_buffer_initializer(&initializer.expr)
        || !matches!(initializer.expr.as_ref(), syn::Expr::Call(call)
            if call.args.iter().all(capacity_is_pure))
    {
        return false;
    }
    let Some(syn::Stmt::Expr(syn::Expr::Path(tail), None)) = block.block.stmts.last() else {
        return false;
    };
    if !tail.path.is_ident(&buffer) {
        return false;
    }
    let mut value = None;
    for statement in &block.block.stmts[1..block.block.stmts.len() - 1] {
        let syn::Stmt::Expr(syn::Expr::MethodCall(push), Some(_)) = statement else {
            return false;
        };
        if push.method != "push_str"
            || push.args.len() != 1
            || !matches!(push.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident(&buffer))
        {
            return false;
        }
        let Some(argument) = push.args.first() else {
            return false;
        };
        if matches!(argument, syn::Expr::Lit(literal)
            if matches!(&literal.lit, syn::Lit::Str(text) if text.value().is_empty()))
        {
            continue;
        }
        if value.is_some() {
            return false;
        }
        value = copied_string_from_as_str(argument, owned, borrowed);
        if value.is_none() {
            return false;
        }
    }
    let Some(value) = value else {
        return false;
    };
    *expression = value;
    true
}

fn is_string_buffer_initializer(expression: &syn::Expr) -> bool {
    let syn::Expr::Call(call) = expression else {
        return false;
    };
    let syn::Expr::Path(path) = call.func.as_ref() else {
        return false;
    };
    let segments = path.path.segments.iter().collect::<Vec<_>>();
    segments.len() == 2
        && segments[0].ident == "String"
        && matches!(
            segments[1].ident.to_string().as_str(),
            "new" | "with_capacity"
        )
}

fn copied_string_from_as_str(
    expression: &syn::Expr,
    owned: &HashSet<String>,
    borrowed: &HashSet<String>,
) -> Option<syn::Expr> {
    if let syn::Expr::Path(path) = expression
        && path.qself.is_none()
        && path.path.segments.len() == 1
        && borrowed.contains(&path.path.segments[0].ident.to_string())
    {
        let receiver = syn::Expr::Path(path.clone());
        return Some(syn::parse_quote!(#receiver.to_string()));
    }
    let syn::Expr::MethodCall(as_str) = expression else {
        return None;
    };
    if as_str.method != "as_str" || !as_str.args.is_empty() {
        return None;
    }
    match as_str.receiver.as_ref() {
        syn::Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => {
            let receiver = syn::Expr::Path(path.clone());
            let name = path.path.segments.first()?.ident.to_string();
            if owned.contains(&name) {
                Some(syn::parse_quote!(#receiver.clone()))
            } else if borrowed.contains(&name) {
                Some(syn::parse_quote!(#receiver.to_string()))
            } else {
                None
            }
        }
        syn::Expr::MethodCall(clone)
            if clone.method == "clone"
                && clone.args.is_empty()
                && matches!(clone.receiver.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name| owned.contains(&name.to_string()))) =>
        {
            Some(syn::Expr::MethodCall(clone.clone()))
        }
        _ => None,
    }
}
