use super::attributes_derive;
use std::collections::HashSet;

pub(super) fn derived_copy_owners(items: &[syn::Item]) -> HashSet<String> {
    items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Enum(enum_)
                if attributes_derive(&enum_.attrs, "Copy")
                    && enum_
                        .variants
                        .iter()
                        .all(|variant| variant.fields.is_empty()) =>
            {
                Some(enum_.ident.to_string())
            }
            _ => None,
        })
        .collect()
}

pub(super) fn input_is_mutable_reference(argument: &syn::FnArg) -> bool {
    match argument {
        syn::FnArg::Receiver(receiver) => {
            matches!(receiver.kind, syn::ReceiverKind::Reference(_, _, Some(_)))
        }
        syn::FnArg::Typed(argument) => {
            matches!(argument.ty.as_ref(), syn::Type::Reference(reference) if reference.mutability.is_some())
        }
    }
}

pub(super) fn borrowed_parameter_names(signature: &syn::Signature) -> HashSet<String> {
    signature
        .inputs
        .iter()
        .filter_map(|argument| match argument {
            syn::FnArg::Receiver(receiver)
                if matches!(receiver.kind, syn::ReceiverKind::Reference(..)) =>
            {
                Some("self".to_string())
            }
            syn::FnArg::Typed(parameter)
                if matches!(parameter.ty.as_ref(), syn::Type::Reference(_)) =>
            {
                simple_pattern_name(&parameter.pat)
            }
            _ => None,
        })
        .collect()
}

pub(super) fn simple_pattern_name(pattern: &syn::Pat) -> Option<String> {
    match pattern {
        syn::Pat::Ident(binding) if binding.subpat.is_none() => Some(binding.ident.to_string()),
        syn::Pat::Type(typed) => simple_pattern_name(&typed.pat),
        syn::Pat::Paren(paren) => simple_pattern_name(&paren.pat),
        _ => None,
    }
}

pub(super) fn expression_is_rooted_in_borrowed_parameter(
    expression: &syn::Expr,
    borrowed_parameters: &HashSet<String>,
) -> bool {
    match expression {
        syn::Expr::Path(path) => {
            path.qself.is_none()
                && path
                    .path
                    .get_ident()
                    .is_some_and(|name| borrowed_parameters.contains(&name.to_string()))
        }
        syn::Expr::Field(field) => {
            expression_is_rooted_in_borrowed_parameter(&field.base, borrowed_parameters)
        }
        syn::Expr::Paren(paren) => {
            expression_is_rooted_in_borrowed_parameter(&paren.expr, borrowed_parameters)
        }
        _ => false,
    }
}

pub(super) fn expression_is_rooted_in_self(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Path(path) => path.qself.is_none() && path.path.is_ident("self"),
        syn::Expr::Field(field) => expression_is_rooted_in_self(&field.base),
        syn::Expr::Paren(paren) => expression_is_rooted_in_self(&paren.expr),
        _ => false,
    }
}
