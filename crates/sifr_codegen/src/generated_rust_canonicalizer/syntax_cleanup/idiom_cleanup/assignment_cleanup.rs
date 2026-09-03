use std::collections::HashSet;

use syn::visit::{self, Visit};

use super::super::mutability_cleanup::statements_mutate_name;

pub(super) fn fold_assignment_conditionals(
    statements: &mut Vec<syn::Stmt>,
    mutating_methods: &HashSet<String>,
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
        if !expression_is_replaceable_initializer(&init_value, &name)
            || expression_uses_identifier(&init_value, &name)
        {
            index += 1;
            continue;
        }
        let Some((condition, then_value, else_value)) =
            assignment_if_values(&statements[index + 1], &name, init_value)
        else {
            index += 1;
            continue;
        };
        if expression_uses_identifier(&condition, &name) {
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

fn expression_is_replaceable_initializer(expression: &syn::Expr, binding: &str) -> bool {
    expression_is_pure_initializer(expression)
        || generated_local_builder_is_pure(expression)
        || (binding.starts_with("sifr_generated_chars_") && is_character_collection(expression))
}

fn generated_local_builder_is_pure(expression: &syn::Expr) -> bool {
    let syn::Expr::Block(block) = expression else {
        return false;
    };
    let Some(syn::Stmt::Expr(syn::Expr::Path(tail), None)) = block.block.stmts.last() else {
        return false;
    };
    let Some(name) = tail.path.get_ident().map(ToString::to_string) else {
        return false;
    };
    if !name.starts_with("sifr_generated_") {
        return false;
    }
    block.block.stmts[..block.block.stmts.len() - 1]
        .iter()
        .all(|statement| generated_builder_statement_is_pure(statement, &name))
}

fn generated_builder_statement_is_pure(statement: &syn::Stmt, name: &str) -> bool {
    match statement {
        syn::Stmt::Local(local) => {
            simple_binding_name(&local.pat).as_deref() == Some(name)
                && local.init.as_ref().is_some_and(|init| {
                    init.diverge.is_none() && generated_builder_initializer_is_pure(&init.expr)
                })
        }
        syn::Stmt::Expr(syn::Expr::MethodCall(call), Some(_)) => {
            matches!(call.method.to_string().as_str(), "push" | "push_str")
                && matches!(call.receiver.as_ref(), syn::Expr::Path(path)
                    if path.qself.is_none() && path.path.is_ident(name))
                && call.args.iter().all(expression_is_pure_initializer)
        }
        _ => false,
    }
}

fn generated_builder_initializer_is_pure(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Call(call) => {
            matches!(call.func.as_ref(), syn::Expr::Path(path)
                if path.path.segments.last().is_some_and(|segment|
                    matches!(segment.ident.to_string().as_str(), "new" | "with_capacity")))
                && call.args.iter().all(expression_is_pure_initializer)
        }
        syn::Expr::Macro(expression_macro) => {
            expression_macro.mac.path.is_ident("vec") && expression_macro.mac.tokens.is_empty()
        }
        _ => false,
    }
}

fn simple_binding_name(pattern: &syn::Pat) -> Option<String> {
    match pattern {
        syn::Pat::Ident(binding) if binding.subpat.is_none() => Some(binding.ident.to_string()),
        syn::Pat::Type(typed) => simple_binding_name(&typed.pat),
        syn::Pat::Paren(paren) => simple_binding_name(&paren.pat),
        _ => None,
    }
}

fn is_character_collection(expression: &syn::Expr) -> bool {
    let syn::Expr::MethodCall(collect) = expression else {
        return false;
    };
    if collect.method != "collect" || !collect.args.is_empty() {
        return false;
    }
    let syn::Expr::MethodCall(chars) = collect.receiver.as_ref() else {
        return false;
    };
    chars.method == "chars"
        && chars.args.is_empty()
        && expression_is_pure_initializer(&chars.receiver)
}

pub(super) fn expression_is_pure_initializer(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Lit(_) | syn::Expr::Path(_) => true,
        syn::Expr::Paren(paren) => expression_is_pure_initializer(&paren.expr),
        syn::Expr::Reference(reference) => expression_is_pure_initializer(&reference.expr),
        syn::Expr::Unary(unary) => expression_is_pure_initializer(&unary.expr),
        syn::Expr::MethodCall(call)
            if call.args.is_empty()
                && matches!(
                    call.method.to_string().as_str(),
                    "as_str" | "len" | "to_string" | "to_owned"
                ) =>
        {
            expression_is_pure_initializer(&call.receiver)
        }
        syn::Expr::Call(call)
            if call.args.is_empty()
                && matches!(call.func.as_ref(), syn::Expr::Path(path)
                    if path.path.segments.last().is_some_and(|segment| segment.ident == "new")) =>
        {
            true
        }
        syn::Expr::Call(call)
            if matches!(call.func.as_ref(), syn::Expr::Path(path)
                if path.path.segments.last().is_some_and(|segment|
                    matches!(segment.ident.to_string().as_str(), "from" | "from_i64"))
                    && path.path.segments.iter().rev().nth(1).is_some_and(|segment| segment.ident == "SifrInt")) =>
        {
            call.args.iter().all(expression_is_pure_initializer)
        }
        _ => false,
    }
}

pub(super) fn expression_uses_identifier(expression: &syn::Expr, name: &str) -> bool {
    struct IdentifierUse<'name> {
        name: &'name str,
        found: bool,
    }
    impl<'ast> Visit<'ast> for IdentifierUse<'_> {
        fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
            if path.qself.is_none() && path.path.is_ident(self.name) {
                self.found = true;
            } else {
                visit::visit_expr_path(self, path);
            }
        }
    }
    let mut use_ = IdentifierUse { name, found: false };
    use_.visit_expr(expression);
    use_.found
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
) -> Option<(syn::Expr, syn::Expr, syn::Expr)> {
    let syn::Stmt::Expr(syn::Expr::If(branch), _) = statement else {
        return None;
    };
    let then_value = assignment_block_value(&branch.then_branch, name)?;
    let else_value = match &branch.else_branch {
        Some((_, alternative)) => assignment_expression_value(alternative, name, fallback)?,
        None => fallback,
    };
    Some((*branch.cond.clone(), then_value, else_value))
}

fn assignment_expression_value(
    expression: &syn::Expr,
    name: &str,
    fallback: syn::Expr,
) -> Option<syn::Expr> {
    match expression {
        syn::Expr::Block(block) => assignment_block_value(&block.block, name)
            .or_else(|| diverging_block_expression(&block.block)),
        syn::Expr::If(branch) => {
            let then_value = assignment_block_value(&branch.then_branch, name)?;
            let else_value = match &branch.else_branch {
                Some((_, alternative)) => {
                    assignment_expression_value(alternative, name, fallback.clone())?
                }
                None => fallback,
            };
            let condition = branch.cond.as_ref();
            Some(syn::parse_quote!(if #condition { #then_value } else { #else_value }))
        }
        _ => None,
    }
}

fn assignment_block_value(block: &syn::Block, name: &str) -> Option<syn::Expr> {
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
    if expression_uses_identifier(value, name) {
        return None;
    }
    if block.stmts.iter().enumerate().any(|(index, statement)| {
        index != *assignment_index && statement_references_name(statement, name)
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

fn statement_references_name(statement: &syn::Stmt, name: &str) -> bool {
    struct Reference<'name> {
        name: &'name str,
        found: bool,
    }
    impl<'ast> Visit<'ast> for Reference<'_> {
        fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
            if path.qself.is_none() && path.path.is_ident(self.name) {
                self.found = true;
            } else {
                visit::visit_expr_path(self, path);
            }
        }
    }
    let mut reference = Reference { name, found: false };
    reference.visit_stmt(statement);
    reference.found
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
