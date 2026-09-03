pub(super) fn remove_explicit_unit_tail(statements: &mut Vec<syn::Stmt>) {
    if matches!(
        statements.last(),
        Some(syn::Stmt::Expr(syn::Expr::Tuple(tuple), None)) if tuple.elems.is_empty()
    ) {
        statements.pop();
    }
}

pub(super) fn remove_redundant_iterator_into_iter(expression: &mut syn::Expr) {
    let syn::Expr::MethodCall(into_iter) = expression else {
        return;
    };
    if into_iter.method != "into_iter" || !into_iter.args.is_empty() {
        return;
    }
    let syn::Expr::MethodCall(producer) = into_iter.receiver.as_ref() else {
        return;
    };
    if !matches!(
        producer.method.to_string().as_str(),
        "sifr_generated_iter__" | "sifr_generated_reversed__"
    ) {
        return;
    }
    *expression = syn::Expr::MethodCall(producer.clone());
}

pub(super) fn rewrite_static_format_to_string(expression: &mut syn::Expr) {
    let syn::Expr::Macro(expression_macro) = expression else {
        return;
    };
    if expression_macro
        .mac
        .path
        .segments
        .last()
        .is_none_or(|segment| segment.ident != "format")
    {
        return;
    }
    let Ok(arguments) = expression_macro.mac.parse_body_with(
        syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
    ) else {
        return;
    };
    let [syn::Expr::Lit(literal)] = arguments.iter().collect::<Vec<_>>().as_slice() else {
        return;
    };
    let syn::Lit::Str(text) = &literal.lit else {
        return;
    };
    if text.value().contains(['{', '}']) {
        return;
    }
    *expression = syn::parse_quote!((#text).to_string());
}
