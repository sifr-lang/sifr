use super::liveness::references_after_statements;
use super::{expression_into_block, is_wildcard_result_pattern, suppress_unused_pattern_bindings};

pub(super) fn remove_unused_bindings(statements: &mut [syn::Stmt]) {
    for index in 0..statements.len() {
        let referenced_later = references_after_statements(&statements[index + 1..]);
        let syn::Stmt::Local(local) = &mut statements[index] else {
            continue;
        };
        let Some(init) = &mut local.init else {
            continue;
        };
        let Some((_, diverge)) = init.diverge.take() else {
            continue;
        };
        suppress_unused_pattern_bindings(&mut local.pat, &referenced_later);
        let predicate = if is_wildcard_result_pattern(&local.pat, "Some") {
            Some("is_none")
        } else if is_wildcard_result_pattern(&local.pat, "Ok") {
            Some("is_err")
        } else {
            None
        };
        let Some(predicate) = predicate else {
            init.diverge = Some((syn::token::Else::default(), diverge));
            continue;
        };
        let tested = init.expr.clone();
        let else_block = expression_into_block(diverge);
        statements[index] = if predicate == "is_none" {
            syn::parse_quote! {
                if (#tested).is_none() #else_block
            }
        } else {
            syn::parse_quote! {
                if (#tested).is_err() #else_block
            }
        };
    }
}
