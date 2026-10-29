pub fn syntax_expression_is_discardable(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Lit(_) => true,
        syn::Expr::Paren(paren) => syntax_expression_is_discardable(&paren.expr),
        syn::Expr::Tuple(tuple) => tuple.elems.iter().all(syntax_expression_is_discardable),
        syn::Expr::Array(array) => array.elems.iter().all(syntax_expression_is_discardable),
        _ => false,
    }
}

/// Extend the shared deletion policy with a lexical standard-library effect
/// proof. Callers must prove the receiver, result, and dispatch identity; names
/// or a generated binding prefix alone are never such a proof.
pub fn syntax_expression_is_discardable_with_standard_proof(
    expression: &syn::Expr,
    standard_proof: impl FnOnce(&syn::Expr) -> bool,
) -> bool {
    syntax_expression_is_discardable(expression) || standard_proof(expression)
}
