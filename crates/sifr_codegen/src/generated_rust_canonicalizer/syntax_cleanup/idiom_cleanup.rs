use std::collections::HashSet;

use quote::quote;
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

mod lint_cleanup;
mod result_control_cleanup;
mod structured_control_cleanup;

use super::mutability_cleanup::statements_mutate_name;
use lint_cleanup::{
    flatten_infallible_result_scaffolding, fold_delayed_initializations, fold_initial_assignments,
    fold_literal_result_bindings, fold_vec_push_sequences, group_long_float_literal,
    group_long_integer_literal, rewrite_assert_comparison, rewrite_empty_vec,
    rewrite_literal_result_fallback, rewrite_option_expression, rewrite_result_match_with_let_else,
    rewrite_single_element_exclusive_range, rewrite_single_value_format, rewrite_unwrap_or_default,
};
use result_control_cleanup::{rewrite_discarded_result_matches, rewrite_result_identity_match};
use structured_control_cleanup::{
    collapse_else_if, collapse_identical_if_else_branches, collapse_nested_if,
    factor_tuple_struct_or_pattern, flatten_or_pattern, invert_negative_condition_with_else,
    remove_single_expression_block,
};

pub(super) fn canonicalize_idioms(file: &mut syn::File, mutating_methods: &HashSet<String>) {
    IdiomCleanup { mutating_methods }.visit_file_mut(file);
}

struct IdiomCleanup<'methods> {
    mutating_methods: &'methods HashSet<String>,
}

impl VisitMut for IdiomCleanup<'_> {
    fn visit_macro_mut(&mut self, rust_macro: &mut syn::Macro) {
        let Some(name) = rust_macro
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
        else {
            return;
        };
        if !matches!(
            name.as_str(),
            "assert"
                | "assert_eq"
                | "assert_ne"
                | "vec"
                | "print"
                | "println"
                | "eprint"
                | "eprintln"
                | "format"
                | "format_args"
        ) {
            return;
        }
        let Ok(mut arguments) =
            rust_macro.parse_body_with(Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated)
        else {
            return;
        };
        for argument in &mut arguments {
            self.visit_expr_mut(argument);
        }
        rust_macro.tokens = quote!(#arguments);
        if name == "assert" {
            rewrite_assert_comparison(rust_macro);
        }
    }

    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        rewrite_discarded_result_matches(&mut block.stmts);
        visit_mut::visit_block_mut(self, block);
        move_scoped_items_before_statements(&mut block.stmts);
        flatten_infallible_result_scaffolding(&mut block.stmts);
        fold_delayed_initializations(&mut block.stmts, self.mutating_methods);
        fold_initial_assignments(&mut block.stmts);
        fold_literal_result_bindings(&mut block.stmts);
        fold_assignment_conditionals(&mut block.stmts, self.mutating_methods);
        fold_vec_push_sequences(&mut block.stmts);
        remove_redundant_else_blocks(&mut block.stmts);
        rewrite_discarded_result_matches(&mut block.stmts);
        rewrite_identity_error_propagation(&mut block.stmts);
        remove_uninhabited_match_semicolons(&mut block.stmts);
    }

    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        rewrite_result_match_with_let_else(local);
        visit_mut::visit_local_mut(self, local);
        rewrite_option_let_else_with_question_mark(local);
        rewrite_result_match_with_let_else(local);
    }

    fn visit_pat_mut(&mut self, pattern: &mut syn::Pat) {
        visit_mut::visit_pat_mut(self, pattern);
        flatten_or_pattern(pattern);
        if let Some(factored) = factor_tuple_struct_or_pattern(pattern) {
            *pattern = factored;
        }
    }

    fn visit_lit_float_mut(&mut self, literal: &mut syn::LitFloat) {
        group_long_float_literal(literal);
    }

    fn visit_lit_int_mut(&mut self, literal: &mut syn::LitInt) {
        group_long_integer_literal(literal);
    }

    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        remove_expression_parentheses(expression);
        rewrite_empty_string_construction(expression);
        rewrite_empty_vec(expression);
        remove_repeated_to_string(expression);
        rewrite_empty_string_comparison(expression);
        rewrite_redundant_borrowed_method_closure(expression);
        rewrite_redundant_method_closure(expression);
        rewrite_immediate_async_closure(expression);
        rewrite_result_identity_match(expression);
        rewrite_literal_result_fallback(expression);
        make_map_or_default_lazy(expression);
        rewrite_identity_map_or(expression);
        make_constant_unwrap_default_eager(expression);
        rewrite_unwrap_or_default(expression);
        remove_known_identity_conversion(expression);
        rewrite_single_element_exclusive_range(expression);
        flatten_generated_format(expression);
        rewrite_single_value_format(expression);
        rewrite_option_expression(expression);
        collapse_nested_if(expression);
        collapse_else_if(expression);
        collapse_identical_if_else_branches(expression);
        invert_negative_condition_with_else(expression);
        remove_single_expression_block(expression);
    }
}

fn move_scoped_items_before_statements(statements: &mut Vec<syn::Stmt>) {
    if !statements
        .iter()
        .any(|statement| matches!(statement, syn::Stmt::Item(_)))
    {
        return;
    }
    let mut items = Vec::new();
    let mut executable = Vec::new();
    for statement in statements.drain(..) {
        if matches!(statement, syn::Stmt::Item(_)) {
            items.push(statement);
        } else {
            executable.push(statement);
        }
    }
    items.extend(executable);
    *statements = items;
}

fn rewrite_identity_error_propagation(statements: &mut [syn::Stmt]) {
    for statement in statements {
        let syn::Stmt::Expr(syn::Expr::If(branch), _) = statement else {
            continue;
        };
        if branch.else_branch.is_some() {
            continue;
        }
        let syn::Expr::Let(condition) = branch.cond.as_ref() else {
            continue;
        };
        let syn::Pat::TupleStruct(pattern) = condition.pat.as_ref() else {
            continue;
        };
        let Some(syn::Pat::Ident(binding)) = pattern.elems.first() else {
            continue;
        };
        if pattern.elems.len() != 1
            || pattern
                .path
                .segments
                .last()
                .is_none_or(|segment| segment.ident != "Err")
        {
            continue;
        }
        let [syn::Stmt::Expr(syn::Expr::Return(return_), _)] = branch.then_branch.stmts.as_slice()
        else {
            continue;
        };
        let Some(syn::Expr::Call(error)) = return_.expr.as_deref() else {
            continue;
        };
        if !matches!(error.func.as_ref(), syn::Expr::Path(path) if path.path.is_ident("Err"))
            || error.args.len() != 1
            || !matches!(error.args.first(), Some(syn::Expr::Path(path))
                if path.qself.is_none() && path.path.is_ident(&binding.ident))
        {
            continue;
        }
        let tested = condition.expr.clone();
        *statement = syn::parse_quote!((#tested)?;);
    }
}

fn fold_assignment_conditionals(
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
        let initializer_may_have_effects = !expression_is_pure_initializer(&init_value);
        let Some((condition, then_value, else_value)) =
            assignment_if_values(&statements[index + 1], &name, init_value)
        else {
            index += 1;
            continue;
        };
        if initializer_may_have_effects
            || expression_uses_identifier(&condition, &name)
            || expression_uses_identifier(&then_value, &name)
            || expression_uses_identifier(&else_value, &name)
        {
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

fn expression_uses_identifier(expression: &syn::Expr, name: &str) -> bool {
    struct IdentifierUse<'name> {
        name: &'name str,
        found: bool,
    }

    impl<'ast> Visit<'ast> for IdentifierUse<'_> {
        fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
            if path.qself.is_none() && path.path.is_ident(self.name) {
                self.found = true;
                return;
            }
            visit::visit_expr_path(self, path);
        }
    }

    let mut use_ = IdentifierUse { name, found: false };
    use_.visit_expr(expression);
    use_.found
}

fn expression_is_pure_initializer(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Lit(_) | syn::Expr::Path(_) => true,
        syn::Expr::Paren(paren) => expression_is_pure_initializer(&paren.expr),
        syn::Expr::Reference(reference) => expression_is_pure_initializer(&reference.expr),
        syn::Expr::Unary(unary) => expression_is_pure_initializer(&unary.expr),
        syn::Expr::MethodCall(call)
            if call.args.is_empty()
                && matches!(
                    call.method.to_string().as_str(),
                    "len" | "to_string" | "to_owned"
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
    let then_value = sole_assignment_value(&branch.then_branch, name)?;
    let else_value = if let Some((_, else_expression)) = &branch.else_branch {
        let syn::Expr::Block(else_block) = else_expression.as_ref() else {
            return None;
        };
        sole_assignment_value(&else_block.block, name)
            .or_else(|| diverging_block_expression(&else_block.block))?
    } else {
        fallback
    };
    Some((*branch.cond.clone(), then_value, else_value))
}

fn diverging_block_expression(block: &syn::Block) -> Option<syn::Expr> {
    let last = block.stmts.last()?;
    if !matches!(
        last,
        syn::Stmt::Expr(
            syn::Expr::Return(_) | syn::Expr::Break(_) | syn::Expr::Continue(_),
            _
        )
    ) {
        return None;
    }
    Some(syn::Expr::Block(syn::ExprBlock {
        attrs: Vec::new(),
        label: None,
        block: block.clone(),
    }))
}

fn sole_assignment_value(block: &syn::Block, name: &str) -> Option<syn::Expr> {
    let (last, prefix) = block.stmts.split_last()?;
    let syn::Stmt::Expr(syn::Expr::Assign(assign), _) = last else {
        return None;
    };
    if !matches!(assign.left.as_ref(), syn::Expr::Path(path) if path.qself.is_none() && path.path.is_ident(name))
    {
        return None;
    }
    let value = assign.right.as_ref();
    if prefix.is_empty() {
        Some(value.clone())
    } else {
        Some(syn::parse_quote!({ #(#prefix)* #value }))
    }
}

fn remove_redundant_else_blocks(statements: &mut Vec<syn::Stmt>) {
    let mut index = 0;
    while index < statements.len() {
        let syn::Stmt::Expr(syn::Expr::If(branch), _) = &mut statements[index] else {
            index += 1;
            continue;
        };
        if !block_always_diverges(&branch.then_branch) {
            index += 1;
            continue;
        }
        let Some((else_token, else_expression)) = branch.else_branch.take() else {
            index += 1;
            continue;
        };
        let syn::Expr::Block(else_block) = *else_expression else {
            branch.else_branch = Some((else_token, else_expression));
            index += 1;
            continue;
        };
        for statement in else_block.block.stmts.into_iter().rev() {
            statements.insert(index + 1, statement);
        }
        index += 1;
    }
}

fn block_always_diverges(block: &syn::Block) -> bool {
    block.stmts.last().is_some_and(|statement| {
        matches!(
            statement,
            syn::Stmt::Expr(
                syn::Expr::Return(_) | syn::Expr::Break(_) | syn::Expr::Continue(_),
                _
            )
        )
    })
}

fn remove_uninhabited_match_semicolons(statements: &mut [syn::Stmt]) {
    for statement in statements {
        if let syn::Stmt::Expr(syn::Expr::Match(match_), semi @ Some(_)) = statement
            && match_.arms.is_empty()
        {
            *semi = None;
        }
    }
}

fn rewrite_option_let_else_with_question_mark(local: &mut syn::Local) {
    let Some(init) = &mut local.init else {
        return;
    };
    let Some((_, diverge)) = &init.diverge else {
        return;
    };
    let syn::Pat::TupleStruct(pattern) = &local.pat else {
        return;
    };
    if pattern
        .path
        .segments
        .last()
        .is_none_or(|segment| segment.ident != "Some")
        || pattern.elems.len() != 1
        || !returns_none(diverge)
    {
        return;
    }
    let Some(inner) = pattern.elems.first().cloned() else {
        return;
    };
    local.pat = inner;
    let tested = init.expr.clone();
    *init.expr = syn::parse_quote!((#tested)?);
    init.diverge = None;
}

fn returns_none(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Block(block) => {
            matches!(block.block.stmts.as_slice(), [syn::Stmt::Expr(inner, _)] if returns_none(inner))
        }
        syn::Expr::Return(return_) => {
            matches!(return_.expr.as_deref(), Some(syn::Expr::Path(path)) if path.path.is_ident("None"))
        }
        syn::Expr::Group(group) => returns_none(&group.expr),
        syn::Expr::Paren(paren) => returns_none(&paren.expr),
        _ => false,
    }
}

fn remove_expression_parentheses(expression: &mut syn::Expr) {
    if let syn::Expr::Paren(paren) = expression {
        *expression = *paren.expr.clone();
    }
}

fn rewrite_empty_string_construction(expression: &mut syn::Expr) {
    let syn::Expr::MethodCall(call) = expression else {
        return;
    };
    if !call.args.is_empty()
        || !matches!(call.method.to_string().as_str(), "to_string" | "to_owned")
    {
        return;
    }
    if matches!(call.receiver.as_ref(), syn::Expr::Lit(literal) if matches!(&literal.lit, syn::Lit::Str(value) if value.value().is_empty()))
    {
        *expression = syn::parse_quote!(String::new());
    }
}

fn remove_repeated_to_string(expression: &mut syn::Expr) {
    let syn::Expr::MethodCall(outer) = expression else {
        return;
    };
    if outer.method != "to_string" || !outer.args.is_empty() {
        return;
    }
    let syn::Expr::MethodCall(inner) = outer.receiver.as_ref() else {
        return;
    };
    if inner.method == "to_string" && inner.args.is_empty() {
        *expression = syn::Expr::MethodCall(inner.clone());
    }
}

fn rewrite_empty_string_comparison(expression: &mut syn::Expr) {
    let syn::Expr::Binary(binary) = expression else {
        return;
    };
    let negated = match binary.op {
        syn::BinOp::Eq(_) => false,
        syn::BinOp::Ne(_) => true,
        _ => return,
    };
    let tested = if is_empty_string(&binary.left) {
        binary.right.clone()
    } else if is_empty_string(&binary.right) {
        binary.left.clone()
    } else {
        return;
    };
    *expression = if negated {
        syn::parse_quote!(!(#tested).is_empty())
    } else {
        syn::parse_quote!((#tested).is_empty())
    };
}

fn is_empty_string(expression: &syn::Expr) -> bool {
    matches!(expression, syn::Expr::Lit(literal) if matches!(&literal.lit, syn::Lit::Str(value) if value.value().is_empty()))
        || matches!(expression, syn::Expr::Call(call)
            if call.args.is_empty()
                && matches!(call.func.as_ref(), syn::Expr::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "new")))
}

fn rewrite_redundant_method_closure(expression: &mut syn::Expr) {
    let syn::Expr::Closure(closure) = expression else {
        return;
    };
    let mut inputs = closure.inputs.iter();
    let Some(syn::Pat::Ident(input)) = inputs.next() else {
        return;
    };
    if inputs.next().is_some() {
        return;
    }
    let syn::Expr::MethodCall(call) = closure.body.as_ref() else {
        return;
    };
    if !call.args.is_empty()
        || !matches!(call.receiver.as_ref(), syn::Expr::Path(path) if path.qself.is_none() && path.path.is_ident(&input.ident))
    {
        return;
    }
    if call.method == "into_sifr_int" {
        *expression = syn::parse_quote!(::sifr_runtime::interop::SifrIntBridge::into_sifr_int);
    }
}

fn rewrite_redundant_borrowed_method_closure(expression: &mut syn::Expr) {
    let syn::Expr::MethodCall(map) = expression else {
        return;
    };
    if map.method != "map" || map.args.len() != 1 {
        return;
    }
    let Some(borrowing_call) = tail_method_call(&map.receiver) else {
        return;
    };
    if !matches!(
        borrowing_call.method.to_string().as_str(),
        "as_deref" | "as_ref" | "first" | "get" | "iter" | "last"
    ) {
        return;
    }
    let Some(syn::Expr::Closure(closure)) = map.args.first() else {
        return;
    };
    let mut inputs = closure.inputs.iter();
    let Some(syn::Pat::Ident(input)) = inputs.next() else {
        return;
    };
    if inputs.next().is_some() {
        return;
    }
    let syn::Expr::MethodCall(call) = closure.body.as_ref() else {
        return;
    };
    if call.method != "to_string"
        || !call.args.is_empty()
        || !matches!(call.receiver.as_ref(), syn::Expr::Path(path)
            if path.qself.is_none() && path.path.is_ident(&input.ident))
    {
        return;
    }
    map.args.clear();
    map.args
        .push(syn::parse_quote!(::std::string::ToString::to_string));
}

fn tail_method_call(expression: &syn::Expr) -> Option<&syn::ExprMethodCall> {
    match expression {
        syn::Expr::MethodCall(call) => Some(call),
        syn::Expr::Block(block) => match block.block.stmts.last()? {
            syn::Stmt::Expr(tail, None) => tail_method_call(tail),
            _ => None,
        },
        syn::Expr::Paren(paren) => tail_method_call(&paren.expr),
        _ => None,
    }
}

fn rewrite_immediate_async_closure(expression: &mut syn::Expr) {
    let syn::Expr::Call(call) = expression else {
        return;
    };
    if !call.args.is_empty() {
        return;
    }
    let syn::Expr::Closure(closure) = call.func.as_ref() else {
        return;
    };
    if closure.asyncness.is_none() || !closure.inputs.is_empty() {
        return;
    }
    let body = closure.body.clone();
    *expression = syn::parse_quote!(async { #body });
}

fn make_map_or_default_lazy(expression: &mut syn::Expr) {
    let syn::Expr::MethodCall(call) = expression else {
        return;
    };
    if call.method != "map_or" || call.args.len() != 2 {
        return;
    }
    let mut args = std::mem::take(&mut call.args).into_iter();
    let Some(default) = args.next() else {
        return;
    };
    let Some(mapper) = args.next() else {
        return;
    };
    if !expression_may_have_effects(&default) {
        call.args.push(default);
        call.args.push(mapper);
        return;
    }
    call.method = syn::Ident::new("map_or_else", call.method.span());
    call.args.push(syn::parse_quote!(|| #default));
    call.args.push(mapper);
}

fn rewrite_identity_map_or(expression: &mut syn::Expr) {
    let syn::Expr::MethodCall(call) = expression else {
        return;
    };
    if !matches!(call.method.to_string().as_str(), "map_or" | "map_or_else") || call.args.len() != 2
    {
        return;
    }
    let Some(mapper) = call.args.last() else {
        return;
    };
    let syn::Expr::Closure(closure) = mapper else {
        return;
    };
    let mut inputs = closure.inputs.iter();
    let Some(syn::Pat::Ident(input)) = inputs.next() else {
        return;
    };
    if inputs.next().is_some()
        || !matches!(closure.body.as_ref(), syn::Expr::Path(path)
            if path.qself.is_none() && path.path.is_ident(&input.ident))
    {
        return;
    }
    let replacement = if call.method == "map_or" {
        "unwrap_or"
    } else {
        "unwrap_or_else"
    };
    call.method = syn::Ident::new(replacement, call.method.span());
    call.args.pop();
}

fn make_constant_unwrap_default_eager(expression: &mut syn::Expr) {
    let syn::Expr::MethodCall(call) = expression else {
        return;
    };
    if call.method != "unwrap_or_else" || call.args.len() != 1 {
        return;
    }
    let Some(syn::Expr::Closure(closure)) = call.args.first() else {
        return;
    };
    if closure.inputs.len() > 1 || !expression_is_pure_initializer(&closure.body) {
        return;
    }
    let default = closure.body.as_ref().clone();
    call.method = syn::Ident::new("unwrap_or", call.method.span());
    call.args.clear();
    call.args.push(default);
}

fn expression_may_have_effects(expression: &syn::Expr) -> bool {
    matches!(
        expression,
        syn::Expr::Call(_)
            | syn::Expr::MethodCall(_)
            | syn::Expr::Macro(_)
            | syn::Expr::Await(_)
            | syn::Expr::Block(_)
    )
}

fn remove_known_identity_conversion(expression: &mut syn::Expr) {
    let syn::Expr::MethodCall(call) = expression else {
        return;
    };
    if call.method == "into"
        && call.args.is_empty()
        && matches!(call.receiver.as_ref(), syn::Expr::Path(path)
            if path.path.segments.last().is_some_and(|segment| segment.ident.to_string().contains("finally_err")))
    {
        *expression = *call.receiver.clone();
    }
}

fn flatten_generated_format(expression: &mut syn::Expr) {
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
    let Ok(arguments) = expression_macro
        .mac
        .parse_body_with(Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated)
    else {
        return;
    };
    let Some(syn::Expr::Lit(format_expression)) = arguments.first() else {
        return;
    };
    let syn::Lit::Str(format_literal) = &format_expression.lit else {
        return;
    };
    let values = arguments.iter().skip(1).cloned().collect::<Vec<_>>();
    if format_literal.value() != "{}".repeat(values.len()) {
        return;
    }
    let mut format = String::new();
    let mut flattened = Vec::new();
    let mut changed = false;
    for value in values {
        if let Some((inner_format, inner_values)) = parsed_format_macro(&value) {
            format.push_str(&inner_format);
            flattened.extend(inner_values);
            changed = true;
        } else if let syn::Expr::Lit(literal) = &value
            && let syn::Lit::Str(text) = &literal.lit
        {
            format.push_str(&text.value().replace('{', "{{").replace('}', "}}"));
            changed = true;
        } else {
            format.push_str("{}");
            flattened.push(value);
        }
    }
    if !changed {
        return;
    }
    let format = syn::LitStr::new(&format, format_literal.span());
    expression_macro.mac.tokens = quote!(#format #(, #flattened)*);
}

fn parsed_format_macro(expression: &syn::Expr) -> Option<(String, Vec<syn::Expr>)> {
    let syn::Expr::Macro(expression_macro) = expression else {
        return None;
    };
    if expression_macro.mac.path.segments.last()?.ident != "format" {
        return None;
    }
    let arguments = expression_macro
        .mac
        .parse_body_with(Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated)
        .ok()?;
    let syn::Expr::Lit(format_expression) = arguments.first()? else {
        return None;
    };
    let syn::Lit::Str(format_literal) = &format_expression.lit else {
        return None;
    };
    Some((
        format_literal.value(),
        arguments.iter().skip(1).cloned().collect(),
    ))
}
