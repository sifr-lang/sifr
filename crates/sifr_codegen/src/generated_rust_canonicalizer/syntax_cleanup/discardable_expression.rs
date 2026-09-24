use std::collections::HashSet;

pub(super) fn simple_binding_name(pattern: &syn::Pat) -> Option<String> {
    match pattern {
        syn::Pat::Ident(binding) if binding.subpat.is_none() => Some(binding.ident.to_string()),
        syn::Pat::Type(typed) => simple_binding_name(&typed.pat),
        syn::Pat::Paren(paren) => simple_binding_name(&paren.pat),
        _ => None,
    }
}

pub(super) fn expression_is_discardable(expression: &syn::Expr) -> bool {
    crate::discardability::syntax_expression_is_discardable(expression)
}

pub(super) fn expression_is_literal_unit(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Tuple(tuple) => tuple.elems.is_empty(),
        syn::Expr::Paren(paren) => expression_is_literal_unit(&paren.expr),
        _ => false,
    }
}

pub(super) fn disposable_typed_unit_binding(
    pattern: &syn::Pat,
    referenced_later: &HashSet<String>,
) -> bool {
    let syn::Pat::Type(typed) = pattern else {
        return false;
    };
    let syn::Type::Tuple(tuple) = typed.ty.as_ref() else {
        return false;
    };
    if !tuple.elems.is_empty() {
        return false;
    }
    match typed.pat.as_ref() {
        syn::Pat::Wild(_) => true,
        syn::Pat::Ident(binding) => !referenced_later.contains(&binding.ident.to_string()),
        _ => false,
    }
}
