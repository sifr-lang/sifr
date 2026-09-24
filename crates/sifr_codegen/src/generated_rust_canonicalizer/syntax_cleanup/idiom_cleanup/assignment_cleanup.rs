use std::collections::HashSet;

use super::super::mutability_cleanup::statements_mutate_name;

pub(super) fn fold_assignment_conditionals(
    statements: &mut Vec<syn::Stmt>,
    mutating_methods: &HashSet<String>,
    discardable: &impl Fn(&syn::Local) -> bool,
    references: &super::super::identifier_collection::InitializerReferences,
) {
    let mut index = 0;
    while index + 1 < statements.len() {
        let Some(name) = mutable_local_name(&statements[index]) else {
            index += 1;
            continue;
        };
        let Some(init_value) = (match &statements[index] {
            syn::Stmt::Local(local) => local
                .init
                .as_ref()
                .filter(|init| init.diverge.is_none())
                .map(|init| *init.expr.clone()),
            _ => None,
        }) else {
            index += 1;
            continue;
        };
        if !matches!(&statements[index], syn::Stmt::Local(local) if discardable(local))
            || references.expression(&init_value, &name)
        {
            index += 1;
            continue;
        }
        let Some((condition, then_value, else_value)) =
            assignment_if_values(&statements[index + 1], &name, init_value, references)
        else {
            index += 1;
            continue;
        };
        if references.expression(&condition, &name) {
            index += 1;
            continue;
        }
        let mutated_later =
            statements_mutate_name(&statements[index + 2..], &name, mutating_methods);
        let syn::Stmt::Local(local) = &mut statements[index] else {
            index += 1;
            continue;
        };
        if !mutated_later {
            remove_simple_pattern_mutability(&mut local.pat);
        }
        if let Some(init) = &mut local.init {
            *init.expr = syn::parse_quote! {
                if #condition { #then_value } else { #else_value }
            };
        }
        statements.remove(index + 1);
        index += 1;
    }
}

pub(super) fn expression_uses_identifier(expression: &syn::Expr, name: &str) -> bool {
    super::super::identifier_collection::expression_may_reference_name(expression, name)
}

fn mutable_local_name(statement: &syn::Stmt) -> Option<String> {
    let syn::Stmt::Local(local) = statement else {
        return None;
    };
    simple_mutable_pattern_name(&local.pat)
}

fn simple_mutable_pattern_name(pattern: &syn::Pat) -> Option<String> {
    match pattern {
        syn::Pat::Ident(binding) if binding.mutability.is_some() && binding.subpat.is_none() => {
            Some(binding.ident.to_string())
        }
        syn::Pat::Type(typed) => simple_mutable_pattern_name(&typed.pat),
        _ => None,
    }
}

fn remove_simple_pattern_mutability(pattern: &mut syn::Pat) {
    match pattern {
        syn::Pat::Ident(binding) => binding.mutability = None,
        syn::Pat::Type(typed) => remove_simple_pattern_mutability(&mut typed.pat),
        _ => {}
    }
}

fn assignment_if_values(
    statement: &syn::Stmt,
    name: &str,
    fallback: syn::Expr,
    references: &super::super::identifier_collection::InitializerReferences,
) -> Option<(syn::Expr, syn::Expr, syn::Expr)> {
    let syn::Stmt::Expr(syn::Expr::If(branch), _) = statement else {
        return None;
    };
    let then_value = assignment_block_value(&branch.then_branch, name, references)?;
    let else_value = match &branch.else_branch {
        Some((_, alternative)) => {
            assignment_expression_value(alternative, name, fallback, references)?
        }
        None => fallback,
    };
    Some((*branch.cond.clone(), then_value, else_value))
}

fn assignment_expression_value(
    expression: &syn::Expr,
    name: &str,
    fallback: syn::Expr,
    references: &super::super::identifier_collection::InitializerReferences,
) -> Option<syn::Expr> {
    match expression {
        syn::Expr::Block(block) => assignment_block_value(&block.block, name, references)
            .or_else(|| diverging_block_expression(&block.block)),
        syn::Expr::If(branch) => {
            let then_value = assignment_block_value(&branch.then_branch, name, references)?;
            let else_value = match &branch.else_branch {
                Some((_, alternative)) => {
                    assignment_expression_value(alternative, name, fallback.clone(), references)?
                }
                None => fallback,
            };
            let condition = branch.cond.as_ref();
            if references.expression(condition, name) {
                return None;
            }
            Some(syn::parse_quote!(if #condition { #then_value } else { #else_value }))
        }
        _ => None,
    }
}

fn assignment_block_value(
    block: &syn::Block,
    name: &str,
    references: &super::super::identifier_collection::InitializerReferences,
) -> Option<syn::Expr> {
    let assignments = block
        .stmts
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| {
            direct_named_assignment(statement, name).map(|value| (index, value))
        })
        .collect::<Vec<_>>();
    let [(assignment_index, value)] = assignments.as_slice() else {
        return None;
    };
    if references.expression(value, name) {
        return None;
    }
    if block.stmts.iter().enumerate().any(|(index, statement)| {
        index != *assignment_index && references.statement(statement, name)
    }) {
        return None;
    }
    if block.stmts.len() == 1 {
        return Some(value.clone());
    }
    let mut statements = block.stmts.clone();
    let binding = syn::Ident::new(name, proc_macro2::Span::call_site());
    let value = value.clone();
    statements[*assignment_index] = syn::parse_quote!(let #binding = #value;);
    Some(syn::parse_quote!({ #(#statements)* #binding }))
}

fn direct_named_assignment(statement: &syn::Stmt, name: &str) -> Option<syn::Expr> {
    let syn::Stmt::Expr(syn::Expr::Assign(assign), _) = statement else {
        return None;
    };
    matches!(assign.left.as_ref(), syn::Expr::Path(path)
        if path.qself.is_none() && path.path.is_ident(name))
    .then(|| assign.right.as_ref().clone())
}

fn diverging_block_expression(block: &syn::Block) -> Option<syn::Expr> {
    matches!(
        block.stmts.last()?,
        syn::Stmt::Expr(
            syn::Expr::Return(_) | syn::Expr::Break(_) | syn::Expr::Continue(_),
            _
        )
    )
    .then(|| {
        syn::Expr::Block(syn::ExprBlock {
            attrs: Vec::new(),
            label: None,
            block: block.clone(),
        })
    })
}
