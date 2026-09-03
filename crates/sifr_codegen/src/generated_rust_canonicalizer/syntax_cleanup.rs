use quote::ToTokens;
use std::collections::HashSet;
use syn::visit_mut::{self, VisitMut};

use super::local_name_cleanup::{
    disambiguate_similar_local_names, disambiguate_similar_names_across_nested_scopes,
    disambiguate_similar_parameter_names,
};

mod dead_assignment_cleanup;
mod identifier_collection;
mod identity_conversion_cleanup;
mod idiom_cleanup;
mod let_else_cleanup;
mod liveness;
mod mutability_cleanup;
mod typed_fallback_cleanup;

use dead_assignment_cleanup::remove_dead_generated_assignments;
pub(super) use identifier_collection::{
    expression_has_control_carrier, statement_identifier_names,
};
use identifier_collection::{
    identifier_names_in_expr, identifier_names_in_pattern, referenced_identifier_names_in_expr,
};
use liveness::{references_after_statements, update_references_crossing_statement};
use mutability_cleanup::remove_unneeded_mutability;
use mutability_cleanup::{
    collect_local_method_facts, collect_mutating_method_names, remove_unneeded_parameter_mutability,
};

pub(super) fn canonicalize_syntax(file: &mut syn::File) {
    identity_conversion_cleanup::remove_known_sifr_int_identity_conversions(file);
    let mutating_methods = collect_mutating_method_names(file);
    let local_method_facts = collect_local_method_facts(file);
    CanonicalSyntaxRewriter {
        mutating_methods: &mutating_methods,
        local_method_facts: &local_method_facts,
    }
    .visit_file_mut(file);
    idiom_cleanup::canonicalize_idioms(file, &mutating_methods);
    typed_fallback_cleanup::canonicalize_typed_fallbacks(file);
}

struct CanonicalSyntaxRewriter<'methods> {
    mutating_methods: &'methods HashSet<String>,
    local_method_facts: &'methods mutability_cleanup::LocalMethodFacts,
}

impl VisitMut for CanonicalSyntaxRewriter<'_> {
    fn visit_item_fn_mut(&mut self, function: &mut syn::ItemFn) {
        remove_explicit_unit_return(&mut function.sig);
        disambiguate_similar_parameter_names(&mut function.sig, &mut function.block);
        remove_unneeded_parameter_mutability(
            &mut function.sig,
            &function.block,
            self.mutating_methods,
            self.local_method_facts,
        );
        visit_mut::visit_item_fn_mut(self, function);
        disambiguate_similar_names_across_nested_scopes(&function.sig, &mut function.block);
        remove_dead_generated_assignments(&mut function.block);
        normalize_tail_position(&mut function.block.stmts);
        terminate_unit_tail(&function.sig, &mut function.block.stmts);
    }

    fn visit_impl_item_fn_mut(&mut self, method: &mut syn::ImplItemFn) {
        remove_explicit_unit_return(&mut method.sig);
        disambiguate_similar_parameter_names(&mut method.sig, &mut method.block);
        remove_unneeded_parameter_mutability(
            &mut method.sig,
            &method.block,
            self.mutating_methods,
            self.local_method_facts,
        );
        visit_mut::visit_impl_item_fn_mut(self, method);
        disambiguate_similar_names_across_nested_scopes(&method.sig, &mut method.block);
        remove_dead_generated_assignments(&mut method.block);
        normalize_tail_position(&mut method.block.stmts);
        terminate_unit_tail(&method.sig, &mut method.block.stmts);
    }

    fn visit_arm_mut(&mut self, arm: &mut syn::Arm) {
        visit_mut::visit_arm_mut(self, arm);
        let used = identifier_names_in_expr(&arm.body);
        suppress_unused_pattern_bindings(&mut arm.pat, &used);
    }

    fn visit_expr_closure_mut(&mut self, closure: &mut syn::ExprClosure) {
        visit_mut::visit_expr_closure_mut(self, closure);
        let used = identifier_names_in_expr(&closure.body);
        for input in &mut closure.inputs {
            suppress_unused_pattern_bindings(input, &used);
        }
    }

    fn visit_expr_for_loop_mut(&mut self, for_loop: &mut syn::ExprForLoop) {
        visit_mut::visit_expr_for_loop_mut(self, for_loop);
        let used = for_loop
            .body
            .stmts
            .iter()
            .flat_map(statement_identifier_names)
            .collect();
        suppress_unused_pattern_bindings(&mut for_loop.pat, &used);
    }

    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        visit_mut::visit_block_mut(self, block);
        disambiguate_similar_local_names(&mut block.stmts);
        remove_redundant_local_types(&mut block.stmts);
        remove_unused_bindings(&mut block.stmts);
        let_else_cleanup::remove_unused_bindings(&mut block.stmts);
        replace_empty_conditionals_with_condition_evaluation(&mut block.stmts);
        remove_unneeded_mutability(
            &mut block.stmts,
            self.mutating_methods,
            self.local_method_facts,
        );
    }

    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        rewrite_single_pattern_match(expression);
        if let syn::Expr::If(if_) = expression {
            let mut used_after = references_after_statements(&if_.then_branch.stmts);
            suppress_unused_condition_bindings(&mut if_.cond, &mut used_after);
        }
        if let syn::Expr::If(if_) = expression
            && let syn::Expr::Let(let_) = if_.cond.as_mut()
        {
            if is_wildcard_result_pattern(&let_.pat, "Err") {
                let tested = &let_.expr;
                *if_.cond = syn::parse_quote!((#tested).is_err());
            } else if is_wildcard_result_pattern(&let_.pat, "Ok") {
                let tested = &let_.expr;
                *if_.cond = syn::parse_quote!((#tested).is_ok());
            } else if is_wildcard_option_pattern(&let_.pat, "Some") {
                let tested = &let_.expr;
                *if_.cond = syn::parse_quote!((#tested).is_some());
            } else if is_none_pattern(&let_.pat) {
                let tested = &let_.expr;
                *if_.cond = syn::parse_quote!((#tested).is_none());
            }
        }
        if let syn::Expr::Call(call) = expression
            && call.args.is_empty()
            && let syn::Expr::Paren(paren) = call.func.as_ref()
            && let syn::Expr::Closure(closure) = paren.expr.as_ref()
            && closure.inputs.is_empty()
            && closure.asyncness.is_none()
            && !expression_has_control_carrier(&closure.body)
        {
            *expression = closure.body.as_ref().clone();
        }
        if let syn::Expr::Closure(closure) = expression
            && let syn::Expr::Block(block) = closure.body.as_mut()
        {
            normalize_tail_position(&mut block.block.stmts);
        }
    }
}

fn suppress_unused_condition_bindings(condition: &mut syn::Expr, used_after: &mut HashSet<String>) {
    match condition {
        syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
            suppress_unused_condition_bindings(&mut binary.right, used_after);
            suppress_unused_condition_bindings(&mut binary.left, used_after);
        }
        syn::Expr::Let(let_) => {
            suppress_unused_pattern_bindings(&mut let_.pat, used_after);
            used_after.extend(referenced_identifier_names_in_expr(&let_.expr));
        }
        syn::Expr::Paren(paren) => {
            suppress_unused_condition_bindings(&mut paren.expr, used_after);
        }
        syn::Expr::Group(group) => {
            suppress_unused_condition_bindings(&mut group.expr, used_after);
        }
        _ => {
            used_after.extend(referenced_identifier_names_in_expr(condition));
        }
    }
}

fn remove_redundant_local_types(statements: &mut [syn::Stmt]) {
    for statement in statements {
        let syn::Stmt::Local(local) = statement else {
            continue;
        };
        let Some(init) = &local.init else {
            continue;
        };
        if init.diverge.is_some() || !initializer_has_self_describing_type(&init.expr) {
            continue;
        }
        let syn::Pat::Type(typed) = &local.pat else {
            continue;
        };
        if matches!(typed.pat.as_ref(), syn::Pat::Ident(binding) if binding.subpat.is_none()) {
            local.pat = typed.pat.as_ref().clone();
        }
    }
}

fn initializer_has_self_describing_type(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Await(await_) => matches!(await_.base.as_ref(), syn::Expr::MethodCall(_)),
        syn::Expr::Paren(paren) => initializer_has_self_describing_type(&paren.expr),
        _ => false,
    }
}

fn rewrite_single_pattern_match(expression: &mut syn::Expr) {
    let syn::Expr::Match(match_) = expression else {
        return;
    };
    if match_.arms.len() != 2
        || !matches!(match_.arms[1].pat, syn::Pat::Wild(_))
        || match_
            .arms
            .iter()
            .any(|arm| matches!(arm.pat, syn::Pat::Guard(_)))
    {
        return;
    }
    let mut arms = std::mem::take(&mut match_.arms);
    let Some(fallback) = arms.pop() else {
        return;
    };
    let Some(selected) = arms.pop() else {
        return;
    };
    let matched = std::mem::replace(&mut match_.expr, Box::new(syn::parse_quote!(())));
    let condition = syn::Expr::Let(syn::ExprLet {
        attrs: Vec::new(),
        let_token: syn::token::Let::default(),
        pat: Box::new(selected.pat),
        eq_token: syn::token::Eq::default(),
        expr: matched,
    });
    *expression = syn::Expr::If(syn::ExprIf {
        attrs: Vec::new(),
        if_token: syn::token::If::default(),
        cond: Box::new(condition),
        then_branch: expression_into_block(selected.body),
        else_branch: Some((
            syn::token::Else::default(),
            Box::new(syn::Expr::Block(syn::ExprBlock {
                attrs: Vec::new(),
                label: None,
                block: expression_into_block(fallback.body),
            })),
        )),
    });
}

pub(super) fn expression_into_block(expression: Box<syn::Expr>) -> syn::Block {
    if let syn::Expr::Block(block) = *expression {
        block.block
    } else {
        syn::Block {
            brace_token: syn::token::Brace::default(),
            stmts: vec![syn::Stmt::Expr(*expression, None)],
        }
    }
}

fn remove_explicit_unit_return(signature: &mut syn::Signature) {
    let syn::ReturnType::Type(_, ty) = &signature.output else {
        return;
    };
    if matches!(ty.as_ref(), syn::Type::Tuple(tuple) if tuple.elems.is_empty()) {
        signature.output = syn::ReturnType::Default;
    }
}

fn terminate_unit_tail(signature: &syn::Signature, statements: &mut [syn::Stmt]) {
    if !matches!(signature.output, syn::ReturnType::Default) {
        return;
    }
    let Some(syn::Stmt::Expr(expression, semi @ None)) = statements.last_mut() else {
        return;
    };
    if !matches!(
        expression,
        syn::Expr::If(_) | syn::Expr::Match(_) | syn::Expr::Block(_)
    ) {
        *semi = Some(syn::token::Semi::default());
    }
}

fn normalize_tail_position(statements: &mut Vec<syn::Stmt>) {
    if let Some(syn::Stmt::Expr(syn::Expr::Return(return_), _)) = statements.last_mut() {
        if let Some(value) = return_.expr.take() {
            if let Some(tail) = statements.last_mut() {
                *tail = syn::Stmt::Expr(*value, None);
            }
        } else {
            statements.pop();
            return;
        }
    }
    fold_tail_option_if_let(statements);
    expand_tail_option_match(statements);
    let Some(tail) = statements.last_mut() else {
        return;
    };
    if let syn::Stmt::Expr(expression, _) = tail {
        normalize_tail_expression(expression);
    }
    if matches!(tail, syn::Stmt::Expr(expression, _) if expression_is_literal_unit(expression)) {
        statements.pop();
    }
}

fn expand_tail_option_match(statements: &mut Vec<syn::Stmt>) {
    let Some(syn::Stmt::Expr(syn::Expr::Match(match_), None)) = statements.last() else {
        return;
    };
    if match_.arms.len() != 2 {
        return;
    }
    let selected = &match_.arms[0];
    let fallback = &match_.arms[1];
    let syn::Pat::TupleStruct(selected_pattern) = &selected.pat else {
        return;
    };
    if selected_pattern
        .path
        .segments
        .last()
        .is_none_or(|segment| segment.ident != "Some")
        || fallback.pat.to_token_stream().to_string() != "None"
    {
        return;
    }

    let tail = statements.pop();
    let Some(syn::Stmt::Expr(syn::Expr::Match(mut match_), None)) = tail else {
        return;
    };
    let fallback = match_.arms.pop();
    let selected = match_.arms.pop();
    let (Some(fallback), Some(selected)) = (fallback, selected) else {
        return;
    };
    let fallback_return = syn::Expr::Return(syn::ExprReturn {
        attrs: Vec::new(),
        return_token: syn::token::Return::default(),
        expr: Some(fallback.body),
    });
    statements.push(syn::Stmt::Local(syn::Local {
        attrs: Vec::new(),
        let_token: syn::token::Let::default(),
        modifiers: syn::LocalModifiers::default(),
        pat: selected.pat,
        init: Some(syn::LocalInit {
            eq_token: syn::token::Eq::default(),
            expr: match_.expr,
            diverge: Some((
                syn::token::Else::default(),
                Box::new(syn::Expr::Block(syn::ExprBlock {
                    attrs: Vec::new(),
                    label: None,
                    block: expression_into_block(Box::new(fallback_return)),
                })),
            )),
        }),
        semi_token: syn::token::Semi::default(),
    }));
    match *selected.body {
        syn::Expr::Block(block) => statements.extend(block.block.stmts),
        expression => statements.push(syn::Stmt::Expr(expression, None)),
    }
}

fn fold_tail_option_if_let(statements: &mut Vec<syn::Stmt>) {
    if statements.len() < 2 {
        return;
    }
    let fallback_index = statements.len() - 1;
    let if_index = fallback_index - 1;
    let syn::Stmt::Expr(_, None) = &statements[fallback_index] else {
        return;
    };
    let syn::Stmt::Expr(syn::Expr::If(if_), _) = &statements[if_index] else {
        return;
    };
    if if_.else_branch.is_some() {
        return;
    }
    let syn::Expr::Let(let_) = if_.cond.as_ref() else {
        return;
    };
    let syn::Pat::TupleStruct(pattern) = let_.pat.as_ref() else {
        return;
    };
    if pattern
        .path
        .segments
        .last()
        .is_none_or(|segment| segment.ident != "Some")
        || !if_.then_branch.stmts.last().is_some_and(|statement| {
            matches!(statement, syn::Stmt::Expr(syn::Expr::Return(return_), _) if return_.expr.is_some())
        })
    {
        return;
    }

    let Some(syn::Stmt::Expr(fallback, None)) = statements.pop() else {
        return;
    };
    let Some(syn::Stmt::Expr(syn::Expr::If(branch), _)) = statements.pop() else {
        return;
    };
    let syn::Expr::Let(condition) = *branch.cond else {
        return;
    };
    let mut selected = branch.then_branch;
    normalize_tail_position(&mut selected.stmts);
    let selected_expression = syn::Expr::Block(syn::ExprBlock {
        attrs: Vec::new(),
        label: None,
        block: selected,
    });
    statements.push(syn::Stmt::Expr(
        syn::Expr::Match(syn::ExprMatch {
            attrs: Vec::new(),
            match_token: syn::token::Match::default(),
            expr: condition.expr,
            brace_token: syn::token::Brace::default(),
            arms: vec![
                syn::Arm {
                    attrs: Vec::new(),
                    pat: *condition.pat,
                    fat_arrow_token: syn::token::FatArrow::default(),
                    body: Box::new(selected_expression),
                    comma: Some(syn::token::Comma::default()),
                },
                syn::Arm {
                    attrs: Vec::new(),
                    pat: syn::parse_quote!(None),
                    fat_arrow_token: syn::token::FatArrow::default(),
                    body: Box::new(fallback),
                    comma: Some(syn::token::Comma::default()),
                },
            ],
        }),
        None,
    ));
}

fn normalize_tail_expression(expression: &mut syn::Expr) {
    match expression {
        syn::Expr::Block(block) => normalize_tail_position(&mut block.block.stmts),
        syn::Expr::If(if_) => {
            let Some((_, else_expression)) = &mut if_.else_branch else {
                return;
            };
            normalize_tail_position(&mut if_.then_branch.stmts);
            normalize_tail_expression(else_expression);
        }
        syn::Expr::Match(match_) => {
            for arm in &mut match_.arms {
                if let syn::Expr::Return(return_) = arm.body.as_mut()
                    && let Some(value) = return_.expr.take()
                {
                    arm.body = value;
                } else {
                    normalize_tail_expression(&mut arm.body);
                }
            }
        }
        _ => {}
    }
}

fn remove_unused_bindings(statements: &mut Vec<syn::Stmt>) {
    let mut referenced_later = HashSet::new();
    let mut retained = Vec::with_capacity(statements.len());
    for mut statement in statements.drain(..).rev() {
        let mut keep = true;
        let mut replacement = None;
        if let syn::Stmt::Local(local) = &mut statement
            && local
                .init
                .as_ref()
                .is_some_and(|init| init.diverge.is_none())
        {
            let binding = simple_binding_name(&local.pat);
            let unused = binding
                .as_ref()
                .is_some_and(|name| !referenced_later.contains(name));
            let disposable_unit = disposable_typed_unit_binding(&local.pat, &referenced_later);
            if unused || disposable_unit {
                replacement = if disposable_unit {
                    local
                        .init
                        .take()
                        .map(|init| syn::Stmt::Expr(*init.expr, Some(local.semi_token)))
                } else {
                    unused.then(|| discarded_match_error_check(local)).flatten()
                };
                if replacement.is_none() {
                    if let Some(init) = local.init.take() {
                        if expression_is_literal_unit(&init.expr)
                            || expression_is_discardable(&init.expr)
                        {
                            keep = false;
                        } else {
                            suppress_unused_pattern_bindings(&mut local.pat, &referenced_later);
                            local.init = Some(init);
                        }
                    }
                }
            } else if binding.is_none() {
                suppress_unused_pattern_bindings(&mut local.pat, &referenced_later);
            }
        }
        if let Some(replacement) = replacement {
            statement = replacement;
        }
        if keep {
            update_references_crossing_statement(&statement, &mut referenced_later);
            retained.push(statement);
        }
    }
    retained.reverse();
    *statements = retained;
}

fn replace_empty_conditionals_with_condition_evaluation(statements: &mut [syn::Stmt]) {
    for statement in statements {
        let syn::Stmt::Expr(syn::Expr::If(if_), _) = statement else {
            continue;
        };
        if !if_.then_branch.stmts.is_empty() || if_.else_branch.is_some() {
            continue;
        }
        let condition = std::mem::replace(&mut if_.cond, Box::new(syn::parse_quote!(false)));
        let evaluated = match *condition {
            syn::Expr::Let(let_) => *let_.expr,
            expression => expression,
        };
        *statement = syn::parse_quote!(let _ = #evaluated;);
    }
}

fn discarded_match_error_check(local: &syn::Local) -> Option<syn::Stmt> {
    let Some(init) = &local.init else {
        return None;
    };
    let syn::Expr::Match(match_) = init.expr.as_ref() else {
        return None;
    };
    if match_.arms.len() != 2 {
        return None;
    }
    let success = &match_.arms[0];
    let failure = &match_.arms[1];
    let syn::Pat::TupleStruct(success_pattern) = &success.pat else {
        return None;
    };
    if success_pattern
        .path
        .segments
        .last()
        .is_none_or(|segment| segment.ident != "Ok")
        || !matches!(success.body.as_ref(), syn::Expr::Path(_))
        || !is_wildcard_result_pattern(&failure.pat, "Err")
        || !expression_always_returns(&failure.body)
    {
        return None;
    }
    let matched = &match_.expr;
    let condition: syn::Expr = syn::parse_quote!((#matched).is_err());
    Some(syn::Stmt::Expr(
        syn::Expr::If(syn::ExprIf {
            attrs: Vec::new(),
            if_token: syn::token::If::default(),
            cond: Box::new(condition),
            then_branch: expression_into_block(failure.body.clone()),
            else_branch: None,
        }),
        Some(syn::token::Semi::default()),
    ))
}

pub(super) fn is_wildcard_result_pattern(pattern: &syn::Pat, variant: &str) -> bool {
    let syn::Pat::TupleStruct(tuple) = pattern else {
        return false;
    };
    tuple
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == variant)
        && matches!(tuple.elems.first(), Some(syn::Pat::Wild(_)))
}

fn is_wildcard_option_pattern(pattern: &syn::Pat, variant: &str) -> bool {
    is_wildcard_result_pattern(pattern, variant)
}

fn is_none_pattern(pattern: &syn::Pat) -> bool {
    matches!(pattern,
        syn::Pat::Path(path) if path.path.is_ident("None")
    ) || matches!(pattern,
        syn::Pat::Ident(binding)
            if binding.ident == "None" && binding.subpat.is_none()
    )
}

fn expression_always_returns(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Return(_) => true,
        syn::Expr::Block(block) => {
            block.block.stmts.last().is_some_and(|statement| {
                matches!(statement, syn::Stmt::Expr(syn::Expr::Return(_), _))
            })
        }
        _ => false,
    }
}

pub(super) fn suppress_unused_pattern_bindings(pattern: &mut syn::Pat, used: &HashSet<String>) {
    let mut reserved = used.clone();
    reserved.extend(identifier_names_in_pattern(pattern));
    suppress_unused_pattern_bindings_inner(pattern, used, &mut reserved);
}

fn suppress_unused_pattern_bindings_inner(
    pattern: &mut syn::Pat,
    used: &HashSet<String>,
    reserved: &mut HashSet<String>,
) {
    match pattern {
        syn::Pat::Ident(binding) => {
            let original = binding.ident.to_string();
            if !used.contains(&original)
                && !original.starts_with('_')
                && !matches!(original.as_str(), "None" | "true" | "false")
            {
                let stable = original
                    .strip_prefix("sifr_generated_")
                    .unwrap_or(&original);
                let base = format!("_{stable}");
                let mut candidate = base.clone();
                let mut suffix = 2_usize;
                while reserved.contains(&candidate) {
                    candidate = format!("{base}_{suffix}");
                    suffix += 1;
                }
                reserved.insert(candidate.clone());
                binding.ident = syn::Ident::new(&candidate, binding.ident.span());
            }
            if let Some((_, subpattern)) = &mut binding.subpat {
                suppress_unused_pattern_bindings_inner(subpattern, used, reserved);
            }
        }
        syn::Pat::Tuple(tuple) => {
            for element in &mut tuple.elems {
                suppress_unused_pattern_bindings_inner(element, used, reserved);
            }
        }
        syn::Pat::TupleStruct(tuple) => {
            for element in &mut tuple.elems {
                suppress_unused_pattern_bindings_inner(element, used, reserved);
            }
        }
        syn::Pat::Struct(struct_) => {
            for field in &mut struct_.fields {
                suppress_unused_pattern_bindings_inner(&mut field.pat, used, reserved);
                if field.colon_token.is_none()
                    && matches!(field.pat.as_ref(), syn::Pat::Ident(binding)
                        if matches!(&field.member, syn::Member::Named(member) if member != &binding.ident))
                {
                    field.colon_token = Some(syn::token::Colon::default());
                }
            }
        }
        syn::Pat::Slice(slice) => {
            for element in &mut slice.elems {
                suppress_unused_pattern_bindings_inner(element, used, reserved);
            }
        }
        syn::Pat::Reference(reference) => {
            suppress_unused_pattern_bindings_inner(&mut reference.pat, used, reserved);
        }
        syn::Pat::Type(typed) => {
            suppress_unused_pattern_bindings_inner(&mut typed.pat, used, reserved);
        }
        syn::Pat::Paren(paren) => {
            suppress_unused_pattern_bindings_inner(&mut paren.pat, used, reserved);
        }
        _ => {}
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

fn expression_is_discardable(expression: &syn::Expr) -> bool {
    crate::discardability::syntax_expression_is_discardable(expression)
}

fn expression_is_literal_unit(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Tuple(tuple) => tuple.elems.is_empty(),
        syn::Expr::Paren(paren) => expression_is_literal_unit(&paren.expr),
        _ => false,
    }
}

fn disposable_typed_unit_binding(pattern: &syn::Pat, referenced_later: &HashSet<String>) -> bool {
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
