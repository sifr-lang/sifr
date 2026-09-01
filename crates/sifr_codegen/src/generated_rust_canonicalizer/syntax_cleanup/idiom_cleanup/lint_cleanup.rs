use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};

pub(super) fn rewrite_result_match_with_let_else(local: &mut syn::Local) {
    let Some(init) = &mut local.init else {
        return;
    };
    if init.diverge.is_some() {
        return;
    }
    let syn::Expr::Match(match_) = init.expr.as_ref() else {
        return;
    };
    if match_.arms.len() != 2
        || match_
            .arms
            .iter()
            .any(|arm| matches!(arm.pat, syn::Pat::Guard(_)))
    {
        return;
    }
    let mut selected = false;
    let mut rejected = None;
    for arm in &match_.arms {
        let syn::Pat::TupleStruct(pattern) = &arm.pat else {
            return;
        };
        let Some(variant) = pattern.path.segments.last() else {
            return;
        };
        if variant.ident == "Ok" && pattern.elems.len() == 1 {
            let syn::Pat::Ident(binding) = &pattern.elems[0] else {
                return;
            };
            if !matches!(arm.body.as_ref(), syn::Expr::Path(path)
                if path.qself.is_none() && path.path.is_ident(&binding.ident))
            {
                return;
            }
            selected = true;
        } else if variant.ident == "Err"
            && pattern.elems.len() == 1
            && matches!(pattern.elems.first(), Some(syn::Pat::Wild(_)))
            && expression_always_diverges(&arm.body)
        {
            rejected = Some(arm.body.as_ref().clone());
        } else {
            return;
        }
    }
    if !selected {
        return;
    }
    let Some(rejected) = rejected else {
        return;
    };
    let binding = local.pat.clone();
    let tested = match_.expr.clone();
    local.pat = syn::parse_quote!(Ok(#binding));
    *init.expr = *tested;
    init.diverge = Some((
        syn::token::Else::default(),
        Box::new(syn::parse_quote!({ #rejected })),
    ));
}

fn expression_always_diverges(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Return(_) | syn::Expr::Break(_) | syn::Expr::Continue(_) => true,
        syn::Expr::Block(block) => block.block.stmts.last().is_some_and(|statement| {
            matches!(
                statement,
                syn::Stmt::Expr(
                    syn::Expr::Return(_) | syn::Expr::Break(_) | syn::Expr::Continue(_),
                    _
                )
            )
        }),
        _ => false,
    }
}

pub(super) fn rewrite_unwrap_or_default(expression: &mut syn::Expr) {
    let syn::Expr::MethodCall(call) = expression else {
        return;
    };
    if call.method != "unwrap_or" || call.args.len() != 1 {
        return;
    }
    let Some(syn::Expr::Call(default)) = call.args.first() else {
        return;
    };
    if !default.args.is_empty()
        || !matches!(default.func.as_ref(), syn::Expr::Path(path)
            if path.path.segments.last().is_some_and(|segment|
                matches!(segment.ident.to_string().as_str(), "default" | "new")))
    {
        return;
    }
    call.method = syn::Ident::new("unwrap_or_default", call.method.span());
    call.args.clear();
}

pub(super) fn rewrite_single_element_exclusive_range(expression: &mut syn::Expr) {
    let syn::Expr::Range(range) = expression else {
        return;
    };
    if !matches!(range.limits, syn::RangeLimits::HalfOpen(_)) {
        return;
    }
    let (Some(start), Some(end)) = (&range.start, &range.end) else {
        return;
    };
    let syn::Expr::Binary(binary) = end.as_ref() else {
        return;
    };
    if !matches!(binary.op, syn::BinOp::Add(_))
        || binary.left.to_token_stream().to_string() != start.to_token_stream().to_string()
        || !matches!(binary.right.as_ref(), syn::Expr::Lit(literal)
            if matches!(&literal.lit, syn::Lit::Int(value)
                if value.base10_parse::<u8>().is_ok_and(|value| value == 1)))
    {
        return;
    }
    range.end = Some(binary.left.clone());
    range.limits = syn::RangeLimits::Closed(syn::token::DotDotEq::default());
}

pub(super) fn flatten_infallible_result_scaffolding(statements: &mut Vec<syn::Stmt>) {
    let mut index = 0;
    while index + 1 < statements.len() {
        let Some((name, body)) = infallible_result_block(&statements[index]) else {
            index += 1;
            continue;
        };
        if !is_impossible_error_check(&statements[index + 1], &name) {
            index += 1;
            continue;
        }
        statements.splice(index..=index + 1, body);
    }
}

fn infallible_result_block(statement: &syn::Stmt) -> Option<(String, Vec<syn::Stmt>)> {
    let syn::Stmt::Local(local) = statement else {
        return None;
    };
    let name = simple_binding_name(&local.pat)?;
    let syn::Expr::Block(block) = local.init.as_ref()?.expr.as_ref() else {
        return None;
    };
    let mut statements = block.block.stmts.clone();
    let syn::Stmt::Expr(tail, None) = statements.last()? else {
        return None;
    };
    if !is_unit_ok(tail) {
        return None;
    }
    statements.pop();
    Some((name, statements))
}

fn is_unit_ok(expression: &syn::Expr) -> bool {
    let syn::Expr::Call(call) = expression else {
        return false;
    };
    matches!(call.func.as_ref(), syn::Expr::Path(path) if path.path.is_ident("Ok"))
        && matches!(call.args.first(), Some(syn::Expr::Tuple(tuple)) if tuple.elems.is_empty())
        && call.args.len() == 1
}

fn is_impossible_error_check(statement: &syn::Stmt, name: &str) -> bool {
    let syn::Stmt::Expr(syn::Expr::If(branch), _) = statement else {
        return false;
    };
    if branch.else_branch.is_some() {
        return false;
    }
    match branch.cond.as_ref() {
        syn::Expr::MethodCall(call) => {
            call.method == "is_err"
                && call.args.is_empty()
                && matches!(call.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident(name))
        }
        syn::Expr::Let(let_) => {
            matches!(let_.pat.as_ref(), syn::Pat::TupleStruct(tuple)
                if tuple.path.segments.last().is_some_and(|segment| segment.ident == "Err"))
                && matches!(let_.expr.as_ref(), syn::Expr::Path(path) if path.path.is_ident(name))
        }
        _ => false,
    }
}

pub(super) fn fold_initial_assignments(statements: &mut Vec<syn::Stmt>) {
    let mut index = 0;
    while index + 1 < statements.len() {
        let Some(name) = initializable_local_name(&statements[index]) else {
            index += 1;
            continue;
        };
        let syn::Stmt::Expr(syn::Expr::Assign(assignment), Some(_)) = &statements[index + 1] else {
            index += 1;
            continue;
        };
        if !matches!(assignment.left.as_ref(), syn::Expr::Path(path) if path.path.is_ident(&name))
            || expression_references_name(&assignment.right, &name)
        {
            index += 1;
            continue;
        }
        let replacement = assignment.right.as_ref().clone();
        let syn::Stmt::Local(local) = &mut statements[index] else {
            index += 1;
            continue;
        };
        if let Some(init) = &mut local.init {
            *init.expr = replacement;
        }
        statements.remove(index + 1);
    }
}

pub(super) fn fold_delayed_initializations(statements: &mut Vec<syn::Stmt>) {
    let mut declaration_index = 0;
    while declaration_index < statements.len() {
        let Some((name, mut pattern, attrs)) =
            movable_default_declaration(&statements[declaration_index])
        else {
            declaration_index += 1;
            continue;
        };
        let mut assignment_index = declaration_index + 1;
        while assignment_index < statements.len()
            && !statement_references_name(&statements[assignment_index], &name)
        {
            assignment_index += 1;
        }
        if assignment_index == statements.len() {
            declaration_index += 1;
            continue;
        }
        let Some(value) = direct_assignment_value(&statements[assignment_index], &name) else {
            declaration_index += 1;
            continue;
        };
        remove_pattern_mutability(&mut pattern);
        statements[assignment_index] = syn::parse_quote!(#(#attrs)* let #pattern = #value;);
        statements.remove(declaration_index);
    }
}

fn movable_default_declaration(
    statement: &syn::Stmt,
) -> Option<(String, syn::Pat, Vec<syn::Attribute>)> {
    let syn::Stmt::Local(local) = statement else {
        return None;
    };
    let name = simple_binding_name(&local.pat)?;
    is_discardable_initializer(local.init.as_ref()?.expr.as_ref())
        .then(|| (name, local.pat.clone(), local.attrs.clone()))
}

fn direct_assignment_value(statement: &syn::Stmt, name: &str) -> Option<syn::Expr> {
    let syn::Stmt::Expr(syn::Expr::Assign(assignment), Some(_)) = statement else {
        return None;
    };
    if !matches!(assignment.left.as_ref(), syn::Expr::Path(path) if path.path.is_ident(name))
        || expression_references_name(&assignment.right, name)
    {
        return None;
    }
    Some(assignment.right.as_ref().clone())
}

fn statement_references_name(statement: &syn::Stmt, name: &str) -> bool {
    let mut collector = NamedReferenceCollector { name, found: false };
    collector.visit_stmt(statement);
    collector.found
}

fn remove_pattern_mutability(pattern: &mut syn::Pat) {
    match pattern {
        syn::Pat::Ident(binding) => binding.mutability = None,
        syn::Pat::Type(typed) => remove_pattern_mutability(&mut typed.pat),
        syn::Pat::Paren(paren) => remove_pattern_mutability(&mut paren.pat),
        _ => {}
    }
}

fn initializable_local_name(statement: &syn::Stmt) -> Option<String> {
    let syn::Stmt::Local(local) = statement else {
        return None;
    };
    let name = simple_binding_name(&local.pat)?;
    is_discardable_initializer(local.init.as_ref()?.expr.as_ref()).then_some(name)
}

fn is_discardable_initializer(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Lit(_) | syn::Expr::Path(_) => true,
        syn::Expr::Tuple(tuple) => tuple.elems.is_empty(),
        syn::Expr::Macro(expression_macro) => {
            expression_macro.mac.path.is_ident("vec") && expression_macro.mac.tokens.is_empty()
        }
        syn::Expr::Call(call) if call.args.is_empty() => matches!(call.func.as_ref(),
            syn::Expr::Path(path)
                if path.path.segments.last().is_some_and(|segment|
                    matches!(segment.ident.to_string().as_str(), "new" | "default"))),
        _ => false,
    }
}

fn expression_references_name(expression: &syn::Expr, name: &str) -> bool {
    let mut collector = NamedReferenceCollector { name, found: false };
    collector.visit_expr(expression);
    collector.found
}

struct NamedReferenceCollector<'name> {
    name: &'name str,
    found: bool,
}

impl<'ast> Visit<'ast> for NamedReferenceCollector<'_> {
    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        if path.qself.is_none() && path.path.is_ident(self.name) {
            self.found = true;
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        if macro_tokens_reference_name(rust_macro.tokens.clone(), self.name) {
            self.found = true;
        }
        visit::visit_macro(self, rust_macro);
    }
}

fn macro_tokens_reference_name(tokens: proc_macro2::TokenStream, name: &str) -> bool {
    tokens.into_iter().any(|token| match token {
        proc_macro2::TokenTree::Ident(identifier) => identifier == name,
        proc_macro2::TokenTree::Group(group) => macro_tokens_reference_name(group.stream(), name),
        _ => false,
    })
}

pub(super) fn rewrite_assert_comparison(rust_macro: &mut syn::Macro) {
    let Ok(arguments) =
        rust_macro.parse_body_with(Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated)
    else {
        return;
    };
    let expressions = arguments.iter().collect::<Vec<_>>();
    let [syn::Expr::Binary(comparison)] = expressions.as_slice() else {
        return;
    };
    let macro_name = match comparison.op {
        syn::BinOp::Eq(_) => "assert_eq",
        syn::BinOp::Ne(_) => "assert_ne",
        _ => return,
    };
    let Ok(path) = syn::parse_str(macro_name) else {
        return;
    };
    let left = comparison.left.as_ref();
    let right = comparison.right.as_ref();
    rust_macro.path = path;
    rust_macro.tokens = quote!(#left, #right);
}

pub(super) fn group_long_integer_literal(literal: &mut syn::LitInt) {
    let rendered = literal.to_string();
    let digits = literal.base10_digits();
    if rendered.starts_with("0x")
        || rendered.starts_with("0o")
        || rendered.starts_with("0b")
        || digits.contains('_')
        || digits.len() < 6
    {
        return;
    }
    let mut grouped = group_integral_digits(digits);
    if !literal.suffix().is_empty() {
        grouped.push('_');
        grouped.push_str(literal.suffix());
    }
    *literal = syn::LitInt::new(&grouped, literal.span());
}

pub(super) fn rewrite_empty_vec(expression: &mut syn::Expr) {
    let syn::Expr::Macro(vec_macro) = expression else {
        return;
    };
    if vec_macro.mac.path.is_ident("vec") && vec_macro.mac.tokens.is_empty() {
        *expression = syn::parse_quote!(Vec::new());
    }
}

pub(super) fn group_long_float_literal(literal: &mut syn::LitFloat) {
    let digits = literal.base10_digits();
    if digits.contains('e') || digits.contains('E') || digits.contains('_') {
        return;
    }
    let (integral, fractional) = digits.split_once('.').unwrap_or((digits, ""));
    if integral.len() < 6 && fractional.len() < 6 {
        return;
    }
    let integral = if integral.len() >= 6 {
        group_integral_digits(integral)
    } else {
        integral.to_string()
    };
    let fractional = fractional
        .as_bytes()
        .chunks(3)
        .map(|chunk| {
            chunk
                .iter()
                .map(|byte| char::from(*byte))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("_");
    let mut grouped = if fractional.is_empty() {
        integral
    } else {
        format!("{integral}.{fractional}")
    };
    if !literal.suffix().is_empty() {
        grouped.push('_');
        grouped.push_str(literal.suffix());
    }
    *literal = syn::LitFloat::new(&grouped, literal.span());
}

fn group_integral_digits(digits: &str) -> String {
    let first = digits.len() % 3;
    let mut grouped = String::new();
    if first > 0 {
        grouped.push_str(&digits[..first]);
    }
    for chunk in digits.as_bytes()[first..].chunks(3) {
        if !grouped.is_empty() {
            grouped.push('_');
        }
        grouped.extend(chunk.iter().map(|byte| char::from(*byte)));
    }
    grouped
}

pub(super) fn rewrite_literal_result_fallback(expression: &mut syn::Expr) {
    let syn::Expr::MethodCall(call) = expression else {
        return;
    };
    if !matches!(
        call.method.to_string().as_str(),
        "unwrap_or" | "unwrap_or_else"
    ) {
        return;
    }
    let syn::Expr::Call(ok) = call.receiver.as_ref() else {
        return;
    };
    if !matches!(ok.func.as_ref(), syn::Expr::Path(path) if path.path.is_ident("Ok"))
        || ok.args.len() != 1
    {
        return;
    }
    if let Some(value) = ok.args.first().cloned() {
        *expression = value;
    }
}

pub(super) fn fold_literal_result_bindings(statements: &mut Vec<syn::Stmt>) {
    let mut index = 0;
    while index + 1 < statements.len() {
        let Some((name, value)) = literal_ok_binding(&statements[index]) else {
            index += 1;
            continue;
        };
        let syn::Stmt::Expr(syn::Expr::MethodCall(fallback), semicolon) =
            &mut statements[index + 1]
        else {
            index += 1;
            continue;
        };
        if !matches!(
            fallback.method.to_string().as_str(),
            "unwrap_or" | "unwrap_or_else"
        ) || !matches!(fallback.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident(&name))
        {
            index += 1;
            continue;
        }
        statements[index + 1] = syn::Stmt::Expr(value, *semicolon);
        statements.remove(index);
    }
}

fn literal_ok_binding(statement: &syn::Stmt) -> Option<(String, syn::Expr)> {
    let syn::Stmt::Local(local) = statement else {
        return None;
    };
    let name = simple_binding_name(&local.pat)?;
    let syn::Expr::Call(ok) = local.init.as_ref()?.expr.as_ref() else {
        return None;
    };
    if !matches!(ok.func.as_ref(), syn::Expr::Path(path) if path.path.is_ident("Ok"))
        || ok.args.len() != 1
    {
        return None;
    }
    Some((name, ok.args.first()?.clone()))
}

pub(super) fn rewrite_single_value_format(expression: &mut syn::Expr) {
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
    let value = if format_literal.value() == "{}" && arguments.len() == 2 {
        arguments.iter().nth(1).cloned()
    } else if arguments.len() == 1 {
        format_capture(&format_literal.value()).and_then(|name| syn::parse_str(&name).ok())
    } else {
        None
    };
    let Some(value) = value else {
        return;
    };
    *expression = syn::parse_quote!((#value).to_string());
}

fn format_capture(format: &str) -> Option<String> {
    let name = format.strip_prefix('{')?.strip_suffix('}')?;
    (!name.is_empty()
        && name
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric()))
    .then(|| name.to_string())
}

pub(super) fn rewrite_option_expression(expression: &mut syn::Expr) {
    let syn::Expr::If(branch) = expression else {
        return;
    };
    let syn::Expr::Let(condition) = branch.cond.as_ref() else {
        return;
    };
    let Some(binding) = some_binding_name(&condition.pat) else {
        return;
    };
    let [syn::Stmt::Expr(selected, None)] = branch.then_branch.stmts.as_slice() else {
        return;
    };
    let Some((_, alternative)) = &branch.else_branch else {
        return;
    };
    let fallback = match alternative.as_ref() {
        syn::Expr::Block(alternative) => {
            let [syn::Stmt::Expr(fallback, None)] = alternative.block.stmts.as_slice() else {
                return;
            };
            fallback.clone()
        }
        fallback => fallback.clone(),
    };
    let tested = condition.expr.as_ref();
    let selected = selected.clone();
    if super::super::expression_has_control_carrier(&selected)
        || super::super::expression_has_control_carrier(&fallback)
    {
        return;
    }
    *expression = if expression_is_false(&fallback) {
        syn::parse_quote!((#tested).is_some_and(|#binding| #selected))
    } else if is_trivially_pure(&fallback) {
        syn::parse_quote!((#tested).map_or(#fallback, |#binding| #selected))
    } else {
        syn::parse_quote!((#tested).map_or_else(|| #fallback, |#binding| #selected))
    };
}

fn expression_is_false(expression: &syn::Expr) -> bool {
    matches!(expression, syn::Expr::Lit(literal)
        if matches!(&literal.lit, syn::Lit::Bool(value) if !value.value))
}

fn is_trivially_pure(expression: &syn::Expr) -> bool {
    matches!(
        expression,
        syn::Expr::Lit(_) | syn::Expr::Path(_) | syn::Expr::Reference(_)
    )
}

fn some_binding_name(pattern: &syn::Pat) -> Option<syn::Ident> {
    let syn::Pat::TupleStruct(tuple) = pattern else {
        return None;
    };
    if tuple.path.segments.last()?.ident != "Some" || tuple.elems.len() != 1 {
        return None;
    }
    let syn::Pat::Ident(binding) = tuple.elems.first()? else {
        return None;
    };
    binding.subpat.is_none().then(|| binding.ident.clone())
}

pub(super) fn fold_vec_push_sequences(statements: &mut Vec<syn::Stmt>) {
    let mut index = 0;
    while index < statements.len() {
        let Some(name) = empty_vec_binding_name(&statements[index]) else {
            index += 1;
            continue;
        };
        let values = statements[index + 1..]
            .iter()
            .map_while(|statement| pushed_value(statement, &name))
            .collect::<Vec<_>>();
        if values.is_empty() {
            index += 1;
            continue;
        }
        let syn::Stmt::Local(local) = &mut statements[index] else {
            index += 1;
            continue;
        };
        if let Some(init) = &mut local.init {
            *init.expr = syn::parse_quote!(vec![#(#values),*]);
        }
        statements.drain(index + 1..index + 1 + values.len());
        index += 1;
    }
}

fn empty_vec_binding_name(statement: &syn::Stmt) -> Option<String> {
    let syn::Stmt::Local(local) = statement else {
        return None;
    };
    let name = simple_binding_name(&local.pat)?;
    let expression = local.init.as_ref()?.expr.as_ref();
    let is_empty = match expression {
        syn::Expr::Macro(vec_macro) => {
            vec_macro.mac.path.is_ident("vec") && vec_macro.mac.tokens.is_empty()
        }
        syn::Expr::Call(call) => {
            call.args.is_empty()
                && matches!(call.func.as_ref(), syn::Expr::Path(path)
                    if path.path.segments.len() == 2
                        && path.path.segments[0].ident == "Vec"
                        && path.path.segments[1].ident == "new")
        }
        _ => false,
    };
    is_empty.then_some(name)
}

fn pushed_value(statement: &syn::Stmt, name: &str) -> Option<syn::Expr> {
    let syn::Stmt::Expr(syn::Expr::MethodCall(call), Some(_)) = statement else {
        return None;
    };
    if call.method != "push"
        || !matches!(call.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident(name))
        || call.args.len() != 1
    {
        return None;
    }
    call.args.first().cloned()
}

fn simple_binding_name(pattern: &syn::Pat) -> Option<String> {
    match pattern {
        syn::Pat::Ident(binding) if binding.subpat.is_none() => Some(binding.ident.to_string()),
        syn::Pat::Type(typed) => simple_binding_name(&typed.pat),
        syn::Pat::Paren(paren) => simple_binding_name(&paren.pat),
        _ => None,
    }
}
