use std::collections::{HashMap, HashSet};

use quote::quote;
use syn::punctuated::Punctuated;
use syn::visit_mut::{self, VisitMut};

mod assignment_cleanup;
mod borrowed_string_arguments;
mod clippy_cleanup;
mod lint_cleanup;
mod residual_cleanup;
mod result_control_cleanup;
mod structured_control_cleanup;

use assignment_cleanup::{expression_uses_identifier, fold_assignment_conditionals};
use borrowed_string_arguments::{
    collect_borrowed_string_params, collect_owned_string_returns,
    remove_returned_string_conversion, rewrite_borrowed_string_literal_arguments,
};
pub(super) use borrowed_string_arguments::{
    collect_project_borrowed_string_params, rewrite_project_borrowed_string_literals,
};
use clippy_cleanup::{
    remove_discardable_expression_statements, remove_last_use_clones,
    remove_last_use_closure_input_clones, remove_needless_collected_length_bindings,
    replace_unused_underscore_bindings, rewrite_clippy_expression,
};

pub(super) fn rewrite_owned_string_clones(signature: &syn::Signature, body: &mut syn::Block) {
    clippy_cleanup::rewrite_owned_string_clones(signature, body);
}

pub(super) fn remove_last_use_parameter_clones(signature: &syn::Signature, body: &mut syn::Block) {
    clippy_cleanup::remove_last_use_parameter_clones(signature, body);
}
use lint_cleanup::{
    flatten_infallible_result_scaffolding, fold_delayed_initializations, fold_initial_assignments,
    fold_literal_result_bindings, fold_tail_bindings, fold_vec_push_sequences,
    group_long_float_literal, group_long_integer_literal, rewrite_assert_comparison,
    rewrite_empty_vec, rewrite_identity_constructor_closure, rewrite_literal_result_fallback,
    rewrite_option_expression, rewrite_option_match_with_if_let,
    rewrite_result_match_with_let_else, rewrite_single_element_exclusive_range,
    rewrite_single_value_format, rewrite_unwrap_or_default, terminate_known_unit_macro_tail,
};
use residual_cleanup::{
    remove_explicit_unit_tail, remove_redundant_iterator_into_iter, rewrite_static_format_to_string,
};
use result_control_cleanup::{rewrite_discarded_result_matches, rewrite_result_identity_match};
use structured_control_cleanup::{
    collapse_else_if, collapse_identical_if_else_branches, collapse_nested_if,
    factor_shared_if_prefix, factor_shared_if_suffix, factor_tuple_struct_or_pattern,
    flatten_or_pattern, invert_negative_condition_with_else, remove_single_expression_block,
};

pub(super) fn canonicalize_idioms(file: &mut syn::File, mutating_methods: &HashSet<String>) {
    let borrowed_string_params = collect_borrowed_string_params(file);
    let owned_string_returns = collect_owned_string_returns(file);
    IdiomCleanup {
        mutating_methods,
        borrowed_string_params: &borrowed_string_params,
        owned_string_returns: &owned_string_returns,
    }
    .visit_file_mut(file);
}

struct IdiomCleanup<'methods> {
    mutating_methods: &'methods HashSet<String>,
    borrowed_string_params: &'methods HashMap<String, Vec<bool>>,
    owned_string_returns: &'methods HashSet<String>,
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
                | "write"
                | "writeln"
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
        flatten_nested_format_argument(&name, &mut arguments);
        clippy_cleanup::remove_macro_argument_clones(&name, &mut arguments);
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
        fold_tail_bindings(&mut block.stmts);
        remove_needless_collected_length_bindings(&mut block.stmts);
        remove_last_use_clones(&mut block.stmts);
        replace_unused_underscore_bindings(&mut block.stmts);
        remove_discardable_expression_statements(&mut block.stmts);
        remove_redundant_else_blocks(&mut block.stmts);
        rewrite_discarded_result_matches(&mut block.stmts);
        rewrite_identity_error_propagation(&mut block.stmts);
        remove_uninhabited_match_semicolons(&mut block.stmts);
        remove_explicit_unit_tail(&mut block.stmts);
        terminate_known_unit_macro_tail(&mut block.stmts);
    }

    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        rewrite_result_match_with_let_else(local);
        visit_mut::visit_local_mut(self, local);
        rewrite_copy_local_cloned(local);
        rewrite_option_let_else_with_question_mark(local);
        rewrite_result_match_with_let_else(local);
        clippy_cleanup::add_complex_local_type_expectation(local);
    }

    fn visit_expr_for_loop_mut(&mut self, for_loop: &mut syn::ExprForLoop) {
        visit_mut::visit_expr_for_loop_mut(self, for_loop);
        clippy_cleanup::remove_unnecessary_owned_iteration(for_loop);
    }

    fn visit_expr_closure_mut(&mut self, closure: &mut syn::ExprClosure) {
        visit_mut::visit_expr_closure_mut(self, closure);
        remove_last_use_closure_input_clones(closure);
    }

    fn visit_pat_mut(&mut self, pattern: &mut syn::Pat) {
        visit_mut::visit_pat_mut(self, pattern);
        flatten_or_pattern(pattern);
        if let Some(factored) = factor_tuple_struct_or_pattern(pattern) {
            *pattern = factored;
        }
    }

    fn visit_expr_if_mut(&mut self, branch: &mut syn::ExprIf) {
        visit_mut::visit_expr_if_mut(self, branch);
        clippy_cleanup::remove_last_use_if_let_clones(branch);
    }

    fn visit_expr_match_mut(&mut self, match_: &mut syn::ExprMatch) {
        visit_mut::visit_expr_match_mut(self, match_);
        clippy_cleanup::remove_owned_generated_error_arm_clones(match_);
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
        rewrite_borrowed_string_literal_arguments(expression, self.borrowed_string_params);
        remove_returned_string_conversion(expression, self.owned_string_returns);
        rewrite_identity_constructor_closure(expression);
        rewrite_immediate_async_closure(expression);
        rewrite_result_identity_match(expression);
        rewrite_literal_result_fallback(expression);
        make_map_or_default_lazy(expression);
        rewrite_identity_map_or(expression);
        make_constant_unwrap_default_eager(expression);
        rewrite_unwrap_or_default(expression);
        remove_known_identity_conversion(expression);
        remove_redundant_iterator_into_iter(expression);
        rewrite_single_element_exclusive_range(expression);
        flatten_generated_format(expression);
        rewrite_static_format_to_string(expression);
        rewrite_single_value_format(expression);
        rewrite_option_match_with_if_let(expression);
        rewrite_option_expression(expression);
        collapse_nested_if(expression);
        collapse_else_if(expression);
        collapse_identical_if_else_branches(expression);
        factor_shared_if_prefix(expression);
        factor_shared_if_suffix(expression);
        invert_negative_condition_with_else(expression);
        remove_single_expression_block(expression);
        rewrite_clippy_expression(expression);
    }
}

fn rewrite_copy_local_cloned(local: &mut syn::Local) {
    let syn::Pat::Type(typed) = &local.pat else {
        return;
    };
    if !type_is_option_or_vector_of_copy(&typed.ty) {
        return;
    }
    let Some(init) = &mut local.init else {
        return;
    };
    CopyIteratorRewriter.visit_expr_mut(&mut init.expr);
}

fn type_is_option_or_vector_of_copy(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    if !matches!(segment.ident.to_string().as_str(), "Option" | "Vec") {
        return false;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    matches!(arguments.args.first(), Some(syn::GenericArgument::Type(syn::Type::Path(inner)))
        if inner.path.segments.last().is_some_and(|part|
            matches!(part.ident.to_string().as_str(),
                "bool" | "char" | "f32" | "f64" | "i8" | "i16" | "i32" | "i64"
                    | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
                    | "usize")))
}

struct CopyIteratorRewriter;

impl VisitMut for CopyIteratorRewriter {
    fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
        visit_mut::visit_expr_method_call_mut(self, call);
        if call.method == "cloned" && call.args.is_empty() {
            call.method = syn::Ident::new("copied", call.method.span());
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn flatten_nested_format_argument(
    macro_name: &str,
    arguments: &mut Punctuated<syn::Expr, syn::Token![,]>,
) {
    let format_index = match macro_name {
        "write" | "writeln" => 1,
        "format" | "print" | "println" | "eprint" | "eprintln" => 0,
        _ => return,
    };
    let values = arguments.iter().cloned().collect::<Vec<_>>();
    if values.len() != format_index + 2 {
        return;
    }
    let syn::Expr::Lit(outer) = &values[format_index] else {
        return;
    };
    let syn::Lit::Str(outer_format) = &outer.lit else {
        return;
    };
    if outer_format.value() != "{}" {
        return;
    }
    let Some((inner_format, inner_values)) = parsed_format_macro(&values[format_index + 1]) else {
        return;
    };
    let mut flattened = values[..format_index].to_vec();
    flattened.push(syn::Expr::Lit(syn::ExprLit {
        attrs: Vec::new(),
        lit: syn::Lit::Str(syn::LitStr::new(&inner_format, outer_format.span())),
    }));
    flattened.extend(inner_values);
    *arguments = flattened.into_iter().collect();
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

fn remove_redundant_else_blocks(statements: &mut Vec<syn::Stmt>) {
    let mut index = 0;
    while index < statements.len() {
        let syn::Stmt::Expr(syn::Expr::If(branch), _) = &mut statements[index] else {
            index += 1;
            continue;
        };
        if matches!(&branch.else_branch,
            Some((_, alternative))
                if matches!(alternative.as_ref(), syn::Expr::Block(block) if block.block.stmts.is_empty()))
        {
            branch.else_branch = None;
            index += 1;
            continue;
        }
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
    if closure.inputs.len() > 1
        || !crate::discardability::syntax_expression_is_discardable(&closure.body)
    {
        return;
    }
    if closure.inputs.iter().any(|input| match input {
        syn::Pat::Wild(_) => false,
        syn::Pat::Ident(binding) => {
            expression_uses_identifier(&closure.body, &binding.ident.to_string())
        }
        _ => true,
    }) {
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
