use std::collections::HashSet;

use syn::visit_mut::{self, VisitMut};

pub(super) fn remove_explicit_unit_tail(statements: &mut Vec<syn::Stmt>) {
    if matches!(
        statements.last(),
        Some(syn::Stmt::Expr(syn::Expr::Tuple(tuple), None)) if tuple.elems.is_empty()
    ) {
        statements.pop();
    }
}

pub(super) fn remove_redundant_iterator_into_iter(expression: &mut syn::Expr) {
    if let syn::Expr::MethodCall(call) = expression
        && matches!(call.method.to_string().as_str(), "chain" | "extend" | "zip")
    {
        for argument in &mut call.args {
            strip_into_iter(argument);
        }
    }

    let syn::Expr::MethodCall(into_iter) = expression else {
        return;
    };
    if into_iter.method != "into_iter" || !into_iter.args.is_empty() {
        return;
    }
    let syn::Expr::MethodCall(producer) = into_iter.receiver.as_ref() else {
        return;
    };
    let producer_name = producer.method.to_string();
    if !matches!(
        producer_name.as_str(),
        "chain"
            | "cloned"
            | "copied"
            | "cycle"
            | "enumerate"
            | "filter"
            | "filter_map"
            | "flat_map"
            | "flatten"
            | "fuse"
            | "inspect"
            | "map"
            | "map_while"
            | "peekable"
            | "rev"
            | "scan"
            | "skip"
            | "step_by"
            | "take"
            | "zip"
    ) && !matches!(
        producer_name.as_str(),
        "sifr_generated_iter__" | "sifr_generated_reversed__"
    ) {
        return;
    }
    *expression = syn::Expr::MethodCall(producer.clone());
}

fn strip_into_iter(expression: &mut syn::Expr) {
    let syn::Expr::MethodCall(call) = expression else {
        return;
    };
    if call.method == "into_iter" && call.args.is_empty() {
        *expression = call.receiver.as_ref().clone();
    }
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

pub(super) fn rewrite_typed_redundant_len_closures(
    signature: &syn::Signature,
    block: &mut syn::Block,
) {
    let option_vec_names = signature
        .inputs
        .iter()
        .filter_map(|input| {
            let syn::FnArg::Typed(input) = input else {
                return None;
            };
            let syn::Pat::Ident(binding) = input.pat.as_ref() else {
                return None;
            };
            option_contains_vec(&input.ty).then(|| binding.ident.to_string())
        })
        .collect();
    TypedLenClosureCleanup { option_vec_names }.visit_block_mut(block);
}

fn option_contains_vec(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Reference(reference) => option_contains_vec(&reference.elem),
        syn::Type::Paren(paren) => option_contains_vec(&paren.elem),
        syn::Type::Group(group) => option_contains_vec(&group.elem),
        syn::Type::Path(path) => path.path.segments.last().is_some_and(|segment| {
            segment.ident == "Option"
                && matches!(&segment.arguments, syn::PathArguments::AngleBracketed(arguments)
                    if arguments.args.iter().any(|argument| matches!(argument,
                        syn::GenericArgument::Type(syn::Type::Path(inner))
                            if inner.path.segments.last().is_some_and(|inner| inner.ident == "Vec"))))
        }),
        _ => false,
    }
}

struct TypedLenClosureCleanup {
    option_vec_names: HashSet<String>,
}

impl VisitMut for TypedLenClosureCleanup {
    fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
        visit_mut::visit_expr_method_call_mut(self, call);
        if !matches!(
            call.method.to_string().as_str(),
            "map" | "map_or" | "map_or_else"
        ) || !receiver_is_known_vec_option_borrow(&call.receiver, &self.option_vec_names)
        {
            return;
        }
        for argument in &mut call.args {
            let syn::Expr::Closure(closure) = argument else {
                continue;
            };
            let [syn::Pat::Ident(binding)] = closure.inputs.iter().collect::<Vec<_>>().as_slice()
            else {
                continue;
            };
            if matches!(closure.body.as_ref(), syn::Expr::MethodCall(len)
                if len.method == "len"
                    && len.args.is_empty()
                    && matches!(len.receiver.as_ref(), syn::Expr::Path(path)
                        if path.qself.is_none() && path.path.is_ident(&binding.ident)))
            {
                *argument = syn::parse_quote!(::std::vec::Vec::len);
            }
        }
    }
}

fn receiver_is_known_vec_option_borrow(
    receiver: &syn::Expr,
    option_vec_names: &HashSet<String>,
) -> bool {
    let syn::Expr::MethodCall(borrow) = receiver else {
        return false;
    };
    matches!(borrow.method.to_string().as_str(), "as_ref" | "as_deref")
        && borrow.args.is_empty()
        && matches!(borrow.receiver.as_ref(), syn::Expr::Path(path)
            if path.qself.is_none()
                && path.path.segments.len() == 1
                && option_vec_names.contains(&path.path.segments[0].ident.to_string()))
}
