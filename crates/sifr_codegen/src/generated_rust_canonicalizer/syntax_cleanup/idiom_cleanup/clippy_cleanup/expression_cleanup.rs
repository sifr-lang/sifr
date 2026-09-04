pub(super) fn rewrite_clippy_expression(expression: &mut syn::Expr) {
    if rewrite_known_string_error_map(expression) {
        return;
    }
    if rewrite_overeager_cloned_filter(expression)
        || rewrite_owned_field_string_conversion(expression)
    {
        return;
    }
    if rewrite_clone_assignment(expression)
        || rewrite_single_character_pattern(expression)
        || rewrite_unnecessary_float_cast(expression)
        || rewrite_unnecessary_lazy_fallback(expression)
        || rewrite_generated_byte_identity_cast(expression)
        || rewrite_lossless_decimal_scale_cast(expression)
        || rewrite_even_length_remainder(expression)
    {
        return;
    }
    if rewrite_redundant_method_receiver_borrow(expression) {
        return;
    }
    if rewrite_known_borrowed_string_call(expression)
        || rewrite_count_without_cloning(expression)
        || rewrite_usize_len_subtraction(expression)
        || rewrite_constructor_clone(expression)
        || remove_redundant_owned_string_conversion(expression)
        || rewrite_redundant_generated_parent_clone(expression)
        || rewrite_generated_sort_comparison(expression)
    {
        return;
    }
    if rewrite_collected_query(expression) {
        return;
    }
    if rewrite_borrowed_vec_literal(expression) {
        return;
    }
    if rewrite_generated_deref_add_assign(expression) {
        return;
    }
    let syn::Expr::Binary(binary) = expression else {
        return;
    };
    rewrite_owned_comparison_constructors(binary);
    rewrite_owned_tuple_field_string_comparison(binary);
    remove_comparison_operand_clones(binary);
    if comparison_operator(&binary.op)
        && matches!(binary.left.as_ref(), syn::Expr::Reference(_))
        && matches!(binary.right.as_ref(), syn::Expr::Reference(_))
    {
        if let syn::Expr::Reference(left) = binary.left.as_ref() {
            binary.left = left.expr.clone();
        }
        if let syn::Expr::Reference(right) = binary.right.as_ref() {
            binary.right = right.expr.clone();
        }
        return;
    }
    let Some((operation, method)) = exact_integer_operation(&binary.op) else {
        return;
    };
    if !mentions_sifr_int(&binary.left)
        && !mentions_sifr_int(&binary.right)
        && !(matches!(binary.left.as_ref(), syn::Expr::Reference(_))
            && matches!(binary.right.as_ref(), syn::Expr::Reference(_)))
        && !(matches!(binary.left.as_ref(), syn::Expr::Path(path) if path.path.is_ident("self"))
            && matches!(binary.right.as_ref(), syn::Expr::Path(path) if path.path.is_ident("rhs")))
    {
        return;
    }
    let left = binary.left.clone();
    let right = binary.right.clone();
    let operation = syn::Ident::new(operation, proc_macro2::Span::call_site());
    let method = syn::Ident::new(method, proc_macro2::Span::call_site());
    *expression = syn::parse_quote!(::std::ops::#operation::#method(#left, #right));
}

fn rewrite_owned_tuple_field_string_comparison(binary: &mut syn::ExprBinary) {
    if !comparison_operator(&binary.op) {
        return;
    }
    for operand in [&mut binary.left, &mut binary.right] {
        let syn::Expr::Reference(reference) = operand.as_ref() else {
            continue;
        };
        let syn::Expr::MethodCall(clone) = reference.expr.as_ref() else {
            continue;
        };
        if clone.method == "clone"
            && clone.args.is_empty()
            && matches!(clone.receiver.as_ref(), syn::Expr::Field(field)
                if matches!(field.member, syn::Member::Unnamed(_)))
        {
            *operand = clone.receiver.clone();
        }
    }
}

fn rewrite_lossless_decimal_scale_cast(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Cast(cast) = expression else {
        return false;
    };
    if !matches!(cast.ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("i64"))
        || !matches!(cast.expr.as_ref(), syn::Expr::MethodCall(call)
            if call.method == "scale" && call.args.is_empty())
    {
        return false;
    }
    let value = cast.expr.as_ref();
    *expression = syn::parse_quote!(i64::from(#value));
    true
}

fn rewrite_generated_deref_add_assign(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Binary(binary) = expression else {
        return false;
    };
    if !matches!(binary.op, syn::BinOp::AddAssign(_))
        || !matches!(binary.left.as_ref(), syn::Expr::Unary(unary)
            if matches!(unary.op, syn::UnOp::Deref(_)))
    {
        return false;
    }
    let left = binary.left.as_ref();
    let right = binary.right.as_ref();
    *expression = syn::parse_quote!(#left = ::std::ops::Add::add(&#left, &#right));
    true
}

fn rewrite_generated_byte_identity_cast(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Cast(cast) = expression else {
        return false;
    };
    if !matches!(cast.ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("u8"))
        || !matches!(cast.expr.as_ref(), syn::Expr::Unary(unary)
            if matches!(unary.op, syn::UnOp::Deref(_))
                && matches!(unary.expr.as_ref(), syn::Expr::Path(path)
                    if path.path.get_ident().is_some_and(|name|
                        name.to_string().starts_with("sifr_generated_byte"))))
    {
        return false;
    }
    *expression = cast.expr.as_ref().clone();
    true
}

fn rewrite_even_length_remainder(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Binary(comparison) = expression else {
        return false;
    };
    if !matches!(comparison.op, syn::BinOp::Ne(_))
        || !matches!(comparison.right.as_ref(), syn::Expr::Lit(literal)
            if matches!(&literal.lit, syn::Lit::Int(value) if value.base10_digits() == "0"))
    {
        return false;
    }
    let syn::Expr::Binary(remainder) = comparison.left.as_ref() else {
        return false;
    };
    if !matches!(remainder.op, syn::BinOp::Rem(_))
        || !matches!(remainder.right.as_ref(), syn::Expr::Lit(literal)
            if matches!(&literal.lit, syn::Lit::Int(value) if value.base10_digits() == "2"))
    {
        return false;
    }
    let value = remainder.left.as_ref();
    *expression = syn::parse_quote!(!(#value).is_multiple_of(2));
    true
}

fn rewrite_redundant_generated_parent_clone(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(outer) = expression else {
        return false;
    };
    if outer.method != "clone" || !outer.args.is_empty() {
        return false;
    }
    let syn::Expr::Field(field) = outer.receiver.as_ref() else {
        return false;
    };
    let syn::Expr::MethodCall(parent_clone) = field.base.as_ref() else {
        return false;
    };
    if parent_clone.method != "clone" || !parent_clone.args.is_empty() {
        return false;
    }
    let generated_checked_tuple = matches!(parent_clone.receiver.as_ref(), syn::Expr::Path(path)
        if path.path.get_ident().is_some_and(|name|
            name.to_string().starts_with("sifr_generated_checked_value")));
    let generated_string_field = matches!(&field.member, syn::Member::Named(name)
        if matches!(name.to_string().as_str(), "kind" | "label" | "message" | "name"));
    if !generated_checked_tuple && !generated_string_field {
        return false;
    }
    let mut field = field.clone();
    field.base = parent_clone.receiver.clone();
    outer.receiver = Box::new(syn::Expr::Field(field));
    true
}

fn rewrite_generated_sort_comparison(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(call) = expression else {
        return false;
    };
    if call.method != "cmp" || call.args.len() != 1 {
        return false;
    }
    let syn::Expr::Path(receiver) = call.receiver.as_ref() else {
        return false;
    };
    let Some(receiver_name) = receiver.path.get_ident() else {
        return false;
    };
    let Some(syn::Expr::Reference(argument)) = call.args.first() else {
        return false;
    };
    if argument.mutability.is_some()
        || !matches!(argument.expr.as_ref(), syn::Expr::Path(path)
            if path.path.get_ident().is_some_and(|name|
                name.to_string().starts_with("sifr_generated_sorted_")))
        || !receiver_name.to_string().starts_with("sifr_generated_sorted_")
    {
        return false;
    }
    call.args[0] = argument.expr.as_ref().clone();
    true
}

fn rewrite_known_borrowed_string_call(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Call(call) = expression else {
        return false;
    };
    let syn::Expr::Path(path) = call.func.as_ref() else {
        return false;
    };
    if !path
        .path
        .segments
        .last()
        .is_some_and(|segment| matches!(segment.ident.to_string().as_str(), "strftime" | "strptime"))
    {
        return false;
    }
    let mut changed = false;
    for argument in &mut call.args {
        let syn::Expr::Reference(reference) = argument else {
            continue;
        };
        let syn::Expr::MethodCall(conversion) = reference.expr.as_ref() else {
            continue;
        };
        if matches!(conversion.method.to_string().as_str(), "to_owned" | "to_string")
            && conversion.args.is_empty()
            && matches!(conversion.receiver.as_ref(), syn::Expr::Lit(literal)
                if matches!(literal.lit, syn::Lit::Str(_)))
        {
            *argument = conversion.receiver.as_ref().clone();
            changed = true;
        }
    }
    changed
}

fn rewrite_count_without_cloning(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(count) = expression else {
        return false;
    };
    if count.method != "count" || !count.args.is_empty() {
        return false;
    }
    let syn::Expr::MethodCall(cloned) = count.receiver.as_ref() else {
        return false;
    };
    if !matches!(cloned.method.to_string().as_str(), "cloned" | "copied")
        || !cloned.args.is_empty()
    {
        return false;
    }
    count.receiver = cloned.receiver.clone();
    true
}

fn rewrite_usize_len_subtraction(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Binary(binary) = expression else {
        return false;
    };
    if !matches!(binary.op, syn::BinOp::Sub(_))
        || !matches!(binary.left.as_ref(), syn::Expr::MethodCall(call)
            if call.method == "len" && call.args.is_empty())
        || !matches!(binary.right.as_ref(), syn::Expr::Lit(literal)
            if matches!(&literal.lit, syn::Lit::Int(value) if value.suffix() == "usize"))
    {
        return false;
    }
    let left = binary.left.as_ref();
    let right = binary.right.as_ref();
    *expression = syn::parse_quote!((#left).saturating_sub(#right));
    true
}

fn rewrite_constructor_clone(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(clone) = expression else {
        return false;
    };
    if clone.method != "clone" || !clone.args.is_empty() {
        return false;
    }
    let syn::Expr::Call(call) = clone.receiver.as_ref() else {
        return false;
    };
    if matches!(call.func.as_ref(), syn::Expr::Path(path)
        if path.path.segments.iter().rev().nth(1).is_some_and(|segment| segment.ident == "SifrInt"))
    {
        *expression = clone.receiver.as_ref().clone();
        return true;
    }
    false
}

fn rewrite_owned_field_string_conversion(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(conversion) = expression else {
        return false;
    };
    if conversion.method != "to_string" || !conversion.args.is_empty() {
        return false;
    }
    let source = match conversion.receiver.as_ref() {
        syn::Expr::Field(field)
            if matches!(&field.member, syn::Member::Named(name)
                if matches!(name.to_string().as_str(), "kind" | "label" | "name")) =>
        {
            conversion.receiver.as_ref()
        }
        syn::Expr::MethodCall(clone)
            if clone.method == "clone"
                && clone.args.is_empty()
                && matches!(clone.receiver.as_ref(), syn::Expr::Field(field)
                    if matches!(&field.member, syn::Member::Named(name)
                        if matches!(name.to_string().as_str(), "kind" | "label" | "name"))) =>
        {
            clone.receiver.as_ref()
        }
        _ => return false,
    };
    let source = source.clone();
    *expression = syn::parse_quote!(#source.clone());
    true
}

fn remove_redundant_owned_string_conversion(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(outer) = expression else {
        return false;
    };
    if !matches!(
        outer.method.to_string().as_str(),
        "clone" | "to_owned" | "to_string"
    ) || !outer.args.is_empty()
    {
        return false;
    }
    let syn::Expr::MethodCall(inner) = outer.receiver.as_ref() else {
        return false;
    };
    if matches!(inner.method.to_string().as_str(), "to_owned" | "to_string")
        && inner.args.is_empty()
    {
        *expression = outer.receiver.as_ref().clone();
        return true;
    }
    false
}

fn rewrite_overeager_cloned_filter(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(filtered) = expression else {
        return false;
    };
    if filtered.method != "filter" || filtered.args.len() != 1 {
        return false;
    }
    let syn::Expr::MethodCall(cloned) = filtered.receiver.as_ref() else {
        return false;
    };
    if cloned.method != "cloned" || !cloned.args.is_empty() {
        return false;
    }
    let Some(syn::Expr::Closure(predicate)) = filtered.args.first_mut() else {
        return false;
    };
    let Some(binding) = predicate.inputs.first().and_then(simple_pattern_name) else {
        return false;
    };
    if predicate.inputs.len() != 1 {
        return false;
    }
    let receiver = cloned.receiver.as_ref();
    let binding = syn::Ident::new(&binding, proc_macro2::Span::call_site());
    predicate.inputs[0] = syn::parse_quote!(&#binding);
    let predicate = predicate.clone();
    *expression = syn::parse_quote!((#receiver).filter(#predicate).cloned());
    true
}

fn rewrite_known_string_error_map(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(map_error) = expression else {
        return false;
    };
    if map_error.method != "map_err"
        || !known_string_error_call(&map_error.receiver)
        || map_error.args.len() != 1
    {
        return false;
    }
    let Some(syn::Expr::Closure(closure)) = map_error.args.first_mut() else {
        return false;
    };
    let Some(name) = closure.inputs.first().and_then(simple_pattern_name) else {
        return false;
    };
    let mut counts = IdentifierUseCounter::default();
    counts.visit_expr(&closure.body);
    StringErrorConversionRewriter {
        name: &name,
        remaining: counts.counts.get(&name).copied().unwrap_or(0),
    }
    .visit_expr_mut(&mut closure.body);
    true
}

fn known_string_error_call(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Await(awaited) => known_string_error_call(&awaited.base),
        syn::Expr::MethodCall(call) => known_string_error_call(&call.receiver),
        syn::Expr::Call(call) => matches!(call.func.as_ref(), syn::Expr::Path(path)
        if {
            let rendered = path.path.to_token_stream().to_string();
            [
                "signals", "net", "tls", "http", "toml", "i18n", "encoding",
                "unicode", "url", "base64", "runtime_observability", "random",
            ].iter().any(|module|
                rendered.contains(&format!("sifr_stdlib :: {module} ::")))
                || rendered.contains("sifr_stdlib :: time :: strptime")
        }),
        syn::Expr::Paren(paren) => known_string_error_call(&paren.expr),
        _ => false,
    }
}

fn rewrite_collected_query(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(query) = expression else {
        return false;
    };
    let syn::Expr::MethodCall(collect) = query.receiver.as_ref() else {
        return false;
    };
    if collect.method != "collect" || !collect.args.is_empty() {
        return false;
    }
    if query.method == "len" && query.args.is_empty() {
        let producer = collect.receiver.as_ref();
        *expression = syn::parse_quote!((#producer).count());
        return true;
    }
    if query.method == "contains" && query.args.len() == 1 {
        let Some(syn::GenericArgument::Type(syn::Type::Path(container))) =
            collect.turbofish.as_ref().and_then(|fish| fish.args.first())
        else {
            return false;
        };
        if !container
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "HashSet")
        {
            return false;
        }
        let producer = collect.receiver.as_ref();
        let Some(argument) = query.args.first() else {
            return false;
        };
        let sought = if let syn::Expr::Reference(reference) = argument {
            reference.expr.as_ref()
        } else {
            argument
        };
        *expression = syn::parse_quote!((#producer).any(|sifr_generated_item| sifr_generated_item == #sought));
        return true;
    }
    false
}

struct StringErrorConversionRewriter<'name> {
    name: &'name str,
    remaining: usize,
}

impl VisitMut for StringErrorConversionRewriter<'_> {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        let syn::Expr::MethodCall(call) = expression else {
            return;
        };
        if call.method != "to_string"
            || !call.args.is_empty()
            || !matches!(call.receiver.as_ref(), syn::Expr::Path(path)
                if path.path.is_ident(self.name))
        {
            return;
        }
        let receiver = call.receiver.as_ref();
        *expression = if self.remaining == 0 {
            receiver.clone()
        } else {
            syn::parse_quote!(#receiver.clone())
        };
    }

    fn visit_expr_path_mut(&mut self, path: &mut syn::ExprPath) {
        if path.path.is_ident(self.name) {
            self.remaining = self.remaining.saturating_sub(1);
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn rewrite_clone_assignment(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Assign(assign) = expression else {
        return false;
    };
    let syn::Expr::MethodCall(clone) = assign.right.as_ref() else {
        return false;
    };
    if clone.method != "clone" || !clone.args.is_empty() {
        return false;
    }
    let left = assign.left.as_ref();
    let source = clone.receiver.as_ref();
    *expression = syn::parse_quote!(#left.clone_from(&#source));
    true
}

fn rewrite_single_character_pattern(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(call) = expression else {
        return false;
    };
    if !matches!(
        call.method.to_string().as_str(),
        "contains"
            | "ends_with"
            | "find"
            | "rfind"
            | "split"
            | "split_inclusive"
            | "split_terminator"
            | "starts_with"
            | "strip_prefix"
            | "strip_suffix"
            | "trim_end_matches"
            | "trim_matches"
            | "trim_start_matches"
    ) || call.args.len() != 1
    {
        return false;
    }
    let Some(syn::Expr::Lit(literal)) = call.args.first_mut() else {
        return false;
    };
    let syn::Lit::Str(text) = &literal.lit else {
        return false;
    };
    let value = text.value();
    let mut characters = value.chars();
    let Some(character) = characters.next() else {
        return false;
    };
    if characters.next().is_some() {
        return false;
    }
    literal.lit = syn::Lit::Char(syn::LitChar::new(character, text.span()));
    true
}

fn rewrite_unnecessary_float_cast(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Cast(cast) = expression else {
        return false;
    };
    if !matches!(cast.ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("f64")) {
        return false;
    }
    let syn::Expr::Lit(literal) = cast.expr.as_ref() else {
        return false;
    };
    let value = match &literal.lit {
        syn::Lit::Float(value) => value.base10_digits().to_string(),
        syn::Lit::Int(value) => value.base10_digits().to_string(),
        _ => return false,
    };
    let Ok(float) =
        syn::LitFloat::new(&format!("{value}_f64"), literal.lit.span()).base10_parse::<f64>()
    else {
        return false;
    };
    let float = syn::LitFloat::new(&format!("{float}_f64"), literal.lit.span());
    *expression = syn::parse_quote!(#float);
    true
}

fn rewrite_unnecessary_lazy_fallback(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(call) = expression else {
        return false;
    };
    if call.method != "unwrap_or_else" || call.args.len() != 1 {
        return false;
    }
    let Some(syn::Expr::Closure(closure)) = call.args.first() else {
        return false;
    };
    if !closure
        .inputs
        .iter()
        .all(|input| matches!(input, syn::Pat::Wild(_))
            || matches!(input, syn::Pat::Ident(binding) if binding.ident.to_string().starts_with('_')))
        || !eager_fallback_is_safe(&closure.body)
    {
        return false;
    }
    let fallback = closure.body.as_ref().clone();
    call.method = syn::Ident::new("unwrap_or", call.method.span());
    call.args = std::iter::once(fallback).collect();
    true
}

fn eager_fallback_is_safe(expression: &syn::Expr) -> bool {
    crate::discardability::syntax_expression_is_discardable(expression)
        || matches!(expression, syn::Expr::Path(path)
            if path.qself.is_none()
                && path.path.segments.len() == 2
                && matches!(path.path.segments[0].ident.to_string().as_str(),
                    "usize" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
                        | "i8" | "i16" | "i32" | "i64" | "i128")
                && path.path.segments[1].ident == "MAX")
}

pub(super) fn remove_discardable_expression_statements(statements: &mut Vec<syn::Stmt>) {
    statements.retain(|statement| {
        !matches!(statement,
            syn::Stmt::Expr(expression, Some(_))
                if crate::discardability::syntax_expression_is_discardable(expression))
    });
}

fn rewrite_redundant_method_receiver_borrow(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(call) = expression else {
        return false;
    };
    let reference = match call.receiver.as_ref() {
        syn::Expr::Paren(paren) => match paren.expr.as_ref() {
            syn::Expr::Reference(reference) => reference,
            _ => return false,
        },
        syn::Expr::Reference(reference) => reference,
        _ => return false,
    };
    if reference.mutability.is_some() {
        return false;
    }
    call.receiver = reference.expr.clone();
    true
}

fn rewrite_borrowed_vec_literal(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Reference(reference) = expression else {
        return false;
    };
    if reference.mutability.is_none()
        && matches!(reference.expr.as_ref(), syn::Expr::Call(call)
            if matches!(call.func.as_ref(), syn::Expr::Path(path)
                if path.path.segments.len() == 2
                    && path.path.segments[0].ident == "String"
                    && path.path.segments[1].ident == "new")
                && call.args.is_empty())
    {
        *expression = syn::parse_quote!("");
        return true;
    }
    let syn::Expr::Macro(vector) = reference.expr.as_ref() else {
        return false;
    };
    if !vector.mac.path.is_ident("vec") {
        return false;
    }
    let tokens = vector.mac.tokens.clone();
    *expression = syn::parse_quote!(&[#tokens]);
    true
}

fn rewrite_owned_comparison_constructors(binary: &mut syn::ExprBinary) {
    if !comparison_operator(&binary.op) {
        return;
    }
    if let Some(inner) = sifr_int_from_argument(&binary.left) {
        binary.left = inner;
    }
    if let Some(inner) = sifr_int_from_argument(&binary.right) {
        binary.right = inner;
    }
}

fn remove_comparison_operand_clones(binary: &mut syn::ExprBinary) {
    if !comparison_operator(&binary.op) {
        return;
    }
    for operand in [&mut binary.left, &mut binary.right] {
        if let syn::Expr::MethodCall(clone) = operand.as_ref()
            && clone.method == "clone"
            && clone.args.is_empty()
        {
            *operand = clone.receiver.clone();
        }
    }
}

pub(super) fn remove_macro_argument_clones(
    macro_name: &str,
    arguments: &mut syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>,
) {
    let start = match macro_name {
        "write" | "writeln" => 2,
        "format" | "print" | "println" | "eprint" | "eprintln" => 1,
        "assert_eq" | "assert_ne" => 0,
        _ => return,
    };
    for argument in arguments.iter_mut().skip(start) {
        MacroBorrowedFieldCloneRemover.visit_expr_mut(argument);
        if let syn::Expr::MethodCall(clone) = argument
            && clone.method == "clone"
            && clone.args.is_empty()
        {
            *argument = clone.receiver.as_ref().clone();
        }
    }
}

struct MacroBorrowedFieldCloneRemover;

impl VisitMut for MacroBorrowedFieldCloneRemover {
    fn visit_expr_field_mut(&mut self, field: &mut syn::ExprField) {
        visit_mut::visit_expr_field_mut(self, field);
        if let syn::Expr::MethodCall(clone) = field.base.as_ref()
            && clone.method == "clone"
            && clone.args.is_empty()
        {
            field.base = clone.receiver.clone();
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

struct SharedSelfBorrowRewriter;

impl VisitMut for SharedSelfBorrowRewriter {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        if let syn::Expr::Reference(reference) = expression
            && reference.mutability.is_none()
            && matches!(reference.expr.as_ref(), syn::Expr::Path(path) if path.path.is_ident("self"))
        {
            *expression = reference.expr.as_ref().clone();
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn sifr_int_from_argument(expression: &syn::Expr) -> Option<Box<syn::Expr>> {
    let syn::Expr::Call(call) = expression else {
        return None;
    };
    let syn::Expr::Path(path) = call.func.as_ref() else {
        return None;
    };
    let segments = path.path.segments.iter().collect::<Vec<_>>();
    if segments.len() < 2
        || segments[segments.len() - 2].ident != "SifrInt"
        || segments.last()?.ident != "from"
        || call.args.len() != 1
    {
        return None;
    }
    call.args.first().cloned().map(Box::new)
}

fn comparison_operator(operator: &syn::BinOp) -> bool {
    matches!(
        operator,
        syn::BinOp::Eq(_)
            | syn::BinOp::Ne(_)
            | syn::BinOp::Lt(_)
            | syn::BinOp::Le(_)
            | syn::BinOp::Gt(_)
            | syn::BinOp::Ge(_)
    )
}

fn exact_integer_operation(operator: &syn::BinOp) -> Option<(&'static str, &'static str)> {
    match operator {
        syn::BinOp::Add(_) => Some(("Add", "add")),
        syn::BinOp::Sub(_) => Some(("Sub", "sub")),
        syn::BinOp::Mul(_) => Some(("Mul", "mul")),
        syn::BinOp::BitAnd(_) => Some(("BitAnd", "bitand")),
        syn::BinOp::BitOr(_) => Some(("BitOr", "bitor")),
        syn::BinOp::BitXor(_) => Some(("BitXor", "bitxor")),
        _ => None,
    }
}

fn mentions_sifr_int(expression: &syn::Expr) -> bool {
    expression.to_token_stream().to_string().contains("SifrInt")
}
