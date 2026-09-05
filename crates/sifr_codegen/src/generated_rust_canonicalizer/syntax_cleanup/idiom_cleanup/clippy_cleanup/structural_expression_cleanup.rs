fn rewrite_boolean_if_expression(expression: &mut syn::Expr) -> bool {
    let syn::Expr::If(branch) = expression else {
        return false;
    };
    let Some(then_value) = block_boolean(&branch.then_branch) else {
        return false;
    };
    let Some((_, alternative)) = &branch.else_branch else {
        return false;
    };
    let syn::Expr::Block(alternative) = alternative.as_ref() else {
        return false;
    };
    let Some(else_value) = block_boolean(&alternative.block) else {
        return false;
    };
    if then_value == else_value {
        return false;
    }
    let mut let_finder = LetExpressionFinder { found: false };
    let_finder.visit_expr(&branch.cond);
    if let_finder.found {
        return false;
    }
    let condition = branch.cond.clone();
    *expression = if then_value {
        *condition
    } else {
        syn::parse_quote!(!(#condition))
    };
    true
}

struct LetExpressionFinder {
    found: bool,
}

impl Visit<'_> for LetExpressionFinder {
    fn visit_expr_let(&mut self, _expression: &syn::ExprLet) {
        self.found = true;
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

pub(super) fn rewrite_created_boolean_conditionals(statements: &mut [syn::Stmt]) {
    for statement in statements {
        let syn::Stmt::Local(local) = statement else {
            continue;
        };
        if let Some(init) = &mut local.init {
            rewrite_boolean_if_expression(&mut init.expr);
        }
    }
}

fn block_boolean(block: &syn::Block) -> Option<bool> {
    let [syn::Stmt::Expr(syn::Expr::Lit(literal), None)] = block.stmts.as_slice() else {
        return None;
    };
    let syn::Lit::Bool(value) = &literal.lit else {
        return None;
    };
    Some(value.value)
}

fn rewrite_owned_literal_comparison(binary: &mut syn::ExprBinary) {
    if !comparison_operator(&binary.op) {
        return;
    }
    if !matches!(binary.op, syn::BinOp::Eq(_) | syn::BinOp::Ne(_)) {
        if let Some(literal) = owned_string_literal(&binary.left) {
            let right = binary.right.clone();
            binary.left = literal;
            *binary.right = syn::parse_quote!((#right).as_str());
        } else if let Some(literal) = owned_string_literal(&binary.right) {
            let left = binary.left.clone();
            *binary.left = syn::parse_quote!((#left).as_str());
            binary.right = literal;
        }
        return;
    }
    for operand in [&mut binary.left, &mut binary.right] {
        if let Some(literal) = owned_string_literal(operand) {
            *operand = literal;
        }
    }
}

fn owned_string_literal(expression: &syn::Expr) -> Option<Box<syn::Expr>> {
    let syn::Expr::MethodCall(conversion) = expression else {
        return None;
    };
    if !matches!(
        conversion.method.to_string().as_str(),
        "to_string" | "to_owned"
    ) || !conversion.args.is_empty()
        || !matches!(conversion.receiver.as_ref(), syn::Expr::Lit(literal)
            if matches!(literal.lit, syn::Lit::Str(_)))
    {
        return None;
    }
    Some(conversion.receiver.clone())
}

fn rewrite_entry_default(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(call) = expression else {
        return false;
    };
    if call.method != "or_insert" || call.args.len() != 1 {
        return false;
    }
    let Some(syn::Expr::Call(default)) = call.args.first() else {
        return false;
    };
    if !default.args.is_empty()
        || !matches!(default.func.as_ref(), syn::Expr::Path(path)
            if path.path.segments.last().is_some_and(|segment| segment.ident == "new"))
    {
        return false;
    }
    call.method = syn::Ident::new("or_default", call.method.span());
    call.args.clear();
    true
}

fn rewrite_integer_float_power(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(call) = expression else {
        return false;
    };
    if call.method != "powf" || call.args.len() != 1 {
        return false;
    }
    let Some(syn::Expr::Lit(argument)) = call.args.first_mut() else {
        return false;
    };
    let (digits, span) = match &argument.lit {
        syn::Lit::Float(value) => (value.base10_digits(), value.span()),
        syn::Lit::Int(value) if value.suffix() == "f64" => (value.base10_digits(), value.span()),
        _ => return false,
    };
    let Ok(parsed) = digits.parse::<f64>() else {
        return false;
    };
    if parsed.fract() != 0.0 || parsed < f64::from(i32::MIN) || parsed > f64::from(i32::MAX) {
        return false;
    }
    let exponent = syn::LitInt::new(&format!("{parsed:.0}"), span);
    argument.lit = syn::Lit::Int(exponent);
    call.method = syn::Ident::new("powi", call.method.span());
    true
}

fn rewrite_generated_usize_increment(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Binary(binary) = expression else {
        return false;
    };
    if !matches!(binary.op, syn::BinOp::AddAssign(_))
        || !matches!(binary.left.as_ref(), syn::Expr::Path(path)
            if path.path.get_ident().is_some_and(|name|
                name.to_string().starts_with("sifr_generated_count")))
        || !matches!(binary.right.as_ref(), syn::Expr::Lit(literal)
            if matches!(&literal.lit, syn::Lit::Int(value)
                if value.base10_digits() == "1"))
    {
        return false;
    }
    let left = binary.left.as_ref();
    *expression = syn::parse_quote!(#left = (#left).saturating_add(1usize));
    true
}

fn rewrite_clone_then_to_vec(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(to_vec) = expression else {
        return false;
    };
    if to_vec.method != "to_vec" || !to_vec.args.is_empty() {
        return false;
    }
    let syn::Expr::MethodCall(clone) = to_vec.receiver.as_ref() else {
        return false;
    };
    if clone.method != "clone" || !clone.args.is_empty() {
        return false;
    }
    *expression = syn::Expr::MethodCall(clone.clone());
    true
}

fn rewrite_generated_owned_to_vec(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(to_vec) = expression else {
        return false;
    };
    if to_vec.method != "to_vec" || !to_vec.args.is_empty() {
        return false;
    }
    let syn::Expr::Block(block) = to_vec.receiver.as_ref() else {
        return false;
    };
    let Some(syn::Stmt::Expr(syn::Expr::Path(tail), None)) = block.block.stmts.last() else {
        return false;
    };
    let Some(name) = tail.path.get_ident() else {
        return false;
    };
    if !name.to_string().starts_with("sifr_generated_v")
        || !block.block.stmts.iter().any(|statement| {
            matches!(statement,
            syn::Stmt::Local(local) if simple_pattern_name(&local.pat).as_deref()
                == Some(name.to_string().as_str()))
        })
    {
        return false;
    }
    *expression = to_vec.receiver.as_ref().clone();
    true
}

fn rewrite_bigdecimal_operation(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Binary(binary) = expression else {
        return false;
    };
    let (trait_name, method_name) = match binary.op {
        syn::BinOp::Add(_) => ("Add", "add"),
        syn::BinOp::Sub(_) => ("Sub", "sub"),
        syn::BinOp::Mul(_) => ("Mul", "mul"),
        syn::BinOp::Div(_) => ("Div", "div"),
        _ => return false,
    };
    let rendered = binary.to_token_stream().to_string();
    if !rendered.contains("BigDecimal") && !rendered.contains("sifr_generated_bigdecimal") {
        return false;
    }
    let left = binary.left.as_ref();
    let right = binary.right.as_ref();
    let trait_name = syn::Ident::new(trait_name, proc_macro2::Span::call_site());
    let method_name = syn::Ident::new(method_name, proc_macro2::Span::call_site());
    *expression = syn::parse_quote!(::std::ops::#trait_name::#method_name(#left, #right));
    true
}

fn rewrite_nonminimal_option_test(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Unary(negation) = expression else {
        return false;
    };
    if !matches!(negation.op, syn::UnOp::Not(_)) {
        return false;
    }
    let syn::Expr::MethodCall(call) = negation.expr.as_ref() else {
        return false;
    };
    if call.method != "is_some" || !call.args.is_empty() {
        return false;
    }
    let mut call = call.clone();
    call.method = syn::Ident::new("is_none", call.method.span());
    *expression = syn::Expr::MethodCall(call);
    true
}

fn rewrite_nan_self_comparison(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Binary(binary) = expression else {
        return false;
    };
    if !matches!(binary.op, syn::BinOp::Ne(_))
        || binary.left.to_token_stream().to_string() != binary.right.to_token_stream().to_string()
        || !matches!(binary.left.as_ref(), syn::Expr::Path(_))
    {
        return false;
    }
    let value = binary.left.as_ref();
    *expression = syn::parse_quote!((#value).is_nan());
    true
}

fn rewrite_nonnegative_unsigned_cast(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Cast(cast) = expression else {
        return false;
    };
    if !matches!(cast.ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("u32")) {
        return false;
    }
    let branch = match cast.expr.as_ref() {
        syn::Expr::If(branch) => branch,
        syn::Expr::Paren(paren) => {
            let syn::Expr::If(branch) = paren.expr.as_ref() else {
                return false;
            };
            branch
        }
        _ => return false,
    };
    if branch.else_branch.is_none()
        || !matches!(branch.then_branch.stmts.as_slice(),
            [syn::Stmt::Expr(syn::Expr::Lit(literal), _)]
                if matches!(&literal.lit, syn::Lit::Int(value) if value.base10_digits() == "0"))
    {
        return false;
    }
    let mut branch = branch.clone();
    if let Some(syn::Stmt::Local(local)) = branch.then_branch.stmts.first_mut() {
        suffix_generated_scale_binding(local);
    }
    let value = syn::Expr::If(branch);
    *expression = syn::parse_quote!((#value).cast_unsigned());
    true
}

pub(super) fn suffix_generated_scale_binding(local: &mut syn::Local) {
    let Some(name) = simple_pattern_name(&local.pat) else {
        return;
    };
    if !name.starts_with("sifr_generated_scale") {
        return;
    }
    let Some(init) = &mut local.init else {
        return;
    };
    let syn::Expr::Lit(literal) = init.expr.as_mut() else {
        return;
    };
    let syn::Lit::Int(value) = &literal.lit else {
        return;
    };
    if value.suffix().is_empty() {
        literal.lit = syn::Lit::Int(syn::LitInt::new(
            &format!("{}_i32", value.base10_digits()),
            value.span(),
        ));
    }
}

pub(super) fn suffix_generated_count_binding(local: &mut syn::Local) {
    let Some(name) = simple_pattern_name(&local.pat) else {
        return;
    };
    if !name.starts_with("sifr_generated_count") {
        return;
    }
    let Some(init) = &mut local.init else {
        return;
    };
    let syn::Expr::Lit(literal) = init.expr.as_mut() else {
        return;
    };
    let syn::Lit::Int(value) = &literal.lit else {
        return;
    };
    if value.suffix().is_empty() {
        literal.lit = syn::Lit::Int(syn::LitInt::new(
            &format!("{}usize", value.base10_digits()),
            value.span(),
        ));
    }
}

fn remove_generated_owned_block_reference(binary: &mut syn::ExprBinary) {
    if !matches!(binary.op, syn::BinOp::Eq(_) | syn::BinOp::Ne(_)) {
        return;
    }
    let syn::Expr::Reference(reference) = binary.left.as_ref() else {
        return;
    };
    let syn::Expr::Block(block) = reference.expr.as_ref() else {
        return;
    };
    let rendered = block.to_token_stream().to_string();
    if !rendered.contains("sifr_generated_slice_src") {
        return;
    }
    binary.left = reference.expr.clone();
}

fn rewrite_usize_zero_ordering(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Binary(binary) = expression else {
        return false;
    };
    if !matches!(binary.op, syn::BinOp::Ge(_))
        || !matches!(binary.left.as_ref(), syn::Expr::Lit(literal)
            if matches!(&literal.lit, syn::Lit::Int(value)
                if value.base10_digits() == "0" && value.suffix() == "usize"))
    {
        return false;
    }
    let right = binary.right.clone();
    binary.left = right;
    *binary.right = syn::parse_quote!(0usize);
    binary.op = syn::BinOp::Eq(syn::Token![==](proc_macro2::Span::call_site()));
    true
}

fn rewrite_borrow_only_cloned_map(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(mapped) = expression else {
        return false;
    };
    if mapped.method != "map" || mapped.args.len() != 1 {
        return false;
    }
    let syn::Expr::MethodCall(cloned) = mapped.receiver.as_ref() else {
        return false;
    };
    if cloned.method != "cloned" || !cloned.args.is_empty() {
        return false;
    }
    let Some(syn::Expr::Closure(mapper)) = mapped.args.first_mut() else {
        return false;
    };
    let Some(binding) = mapper.inputs.first().and_then(simple_pattern_name) else {
        return false;
    };
    if mapper.inputs.len() != 1 {
        return false;
    }
    let mut uses = BorrowOnlyLoopBindingUse {
        binding: &binding,
        owned_use: false,
    };
    uses.visit_expr(&mapper.body);
    if uses.owned_use {
        return false;
    }
    mapped.receiver = cloned.receiver.clone();
    LoopBindingUseRewriter {
        owned: &HashSet::new(),
        borrowed: &HashSet::from([binding]),
    }
    .visit_expr_mut(&mut mapper.body);
    true
}

pub(super) fn rewrite_redundant_literal_guards(match_: &mut syn::ExprMatch) {
    for arm in &mut match_.arms {
        let syn::Pat::Guard(guard) = &arm.pat else {
            continue;
        };
        let syn::Pat::Ident(binding) = guard.pat.as_ref() else {
            continue;
        };
        if binding.by_ref.is_some() || binding.mutability.is_some() || binding.subpat.is_some() {
            continue;
        }
        let syn::Expr::Binary(comparison) = guard.guard.as_ref() else {
            continue;
        };
        if !matches!(comparison.op, syn::BinOp::Eq(_)) {
            continue;
        }
        let literal = if matches!(comparison.left.as_ref(), syn::Expr::Path(path)
            if path.path.is_ident(&binding.ident))
        {
            comparison.right.as_ref()
        } else if matches!(comparison.right.as_ref(), syn::Expr::Path(path)
            if path.path.is_ident(&binding.ident))
        {
            comparison.left.as_ref()
        } else {
            continue;
        };
        let syn::Expr::Lit(literal) = literal else {
            continue;
        };
        let mut uses = IdentifierUseCounter::default();
        uses.visit_expr(&arm.body);
        if uses.counts.contains_key(&binding.ident.to_string()) {
            continue;
        }
        arm.pat = syn::Pat::Lit(syn::PatLit {
            attrs: Vec::new(),
            lit: literal.lit.clone(),
        });
    }
}

pub(super) fn rewrite_literal_print(
    name: &str,
    arguments: &mut syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>,
) {
    if !matches!(name, "print" | "println") || arguments.len() != 2 {
        return;
    }
    let Some(syn::Expr::Lit(format)) = arguments.first() else {
        return;
    };
    if !matches!(&format.lit, syn::Lit::Str(value) if value.value() == "{}") {
        return;
    }
    let Some(syn::Expr::Lit(value)) = arguments.iter().nth(1) else {
        return;
    };
    let rendered = match &value.lit {
        syn::Lit::Bool(value) => value.value.to_string(),
        _ => return,
    };
    arguments.clear();
    let rendered = syn::LitStr::new(&rendered, proc_macro2::Span::call_site());
    arguments.push(syn::parse_quote!(#rendered));
}

pub(super) fn remove_vacuous_literal_assertions(statements: &mut Vec<syn::Stmt>) {
    statements.retain(|statement| {
        let syn::Stmt::Macro(statement) = statement else {
            return true;
        };
        if !statement.mac.path.is_ident("assert_eq") {
            return true;
        }
        let Ok(arguments) = statement.mac.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) else {
            return true;
        };
        let mut values = arguments.iter();
        let (Some(syn::Expr::Lit(left)), Some(syn::Expr::Lit(right)), None) =
            (values.next(), values.next(), values.next())
        else {
            return true;
        };
        !matches!((&left.lit, &right.lit),
            (syn::Lit::Str(left), syn::Lit::Str(right)) if left.value() == right.value()
        ) && !matches!((&left.lit, &right.lit),
            (syn::Lit::Bool(left), syn::Lit::Bool(right)) if left.value == right.value
        ) && !matches!((&left.lit, &right.lit),
            (syn::Lit::Int(left), syn::Lit::Int(right)) if left.base10_digits() == right.base10_digits()
        )
    });
}

pub(super) fn rewrite_single_iteration_while_else(statements: &mut Vec<syn::Stmt>) {
    let mut index = 0;
    while index + 2 < statements.len() {
        let Some(flag) = false_bool_local(&statements[index]) else {
            index += 1;
            continue;
        };
        let syn::Stmt::Expr(syn::Expr::While(while_), _) = &statements[index + 1] else {
            index += 1;
            continue;
        };
        let syn::Stmt::Expr(syn::Expr::If(otherwise), _) = &statements[index + 2] else {
            index += 1;
            continue;
        };
        if otherwise.else_branch.is_some()
            || !matches!(otherwise.cond.as_ref(), syn::Expr::Unary(negation)
                if matches!(negation.op, syn::UnOp::Not(_))
                    && matches!(negation.expr.as_ref(), syn::Expr::Path(path)
                        if path.path.is_ident(&flag)))
            || while_.body.stmts.len() < 2
        {
            index += 1;
            continue;
        }
        let body_len = while_.body.stmts.len();
        if !sets_bool_name(&while_.body.stmts[body_len - 2], &flag, true)
            || !matches!(&while_.body.stmts[body_len - 1],
                syn::Stmt::Expr(syn::Expr::Break(break_), _)
                    if break_.expr.is_none() && break_.label.is_none())
        {
            index += 1;
            continue;
        }
        let condition = while_.cond.as_ref();
        let body = &while_.body.stmts[..body_len - 2];
        let alternative = &otherwise.then_branch.stmts;
        statements[index] =
            syn::parse_quote!(if #condition { #(#body)* } else { #(#alternative)* });
        statements.remove(index + 2);
        statements.remove(index + 1);
    }
}

fn false_bool_local(statement: &syn::Stmt) -> Option<String> {
    let syn::Stmt::Local(local) = statement else {
        return None;
    };
    let name = simple_pattern_name(&local.pat)?;
    (name.starts_with("sifr_generated_broke")
        && local.init.as_ref().is_some_and(|init| {
            matches!(init.expr.as_ref(), syn::Expr::Lit(literal)
                if matches!(&literal.lit, syn::Lit::Bool(value) if !value.value))
        }))
    .then_some(name)
}

fn sets_bool_name(statement: &syn::Stmt, name: &str, expected: bool) -> bool {
    matches!(statement, syn::Stmt::Expr(syn::Expr::Assign(assignment), _)
        if matches!(assignment.left.as_ref(), syn::Expr::Path(path) if path.path.is_ident(name))
            && matches!(assignment.right.as_ref(), syn::Expr::Lit(literal)
                if matches!(&literal.lit, syn::Lit::Bool(value) if value.value == expected)))
}

pub(super) fn rewrite_array_compatible_generated_vecs(statements: &mut [syn::Stmt]) {
    for index in 0..statements.len() {
        let candidate = match &statements[index] {
            syn::Stmt::Local(local) => simple_pattern_name(&local.pat).and_then(|name| {
                let init = local.init.as_ref()?;
                matches!(init.expr.as_ref(), syn::Expr::Macro(rust_macro)
                    if rust_macro.mac.path.is_ident("vec"))
                .then_some(name)
            }),
            _ => None,
        };
        let Some(name) = candidate.filter(|name| name.starts_with("sifr_generated_vals")) else {
            continue;
        };
        let mut uses = ArrayCompatibleUse {
            name: &name,
            valid: true,
        };
        for statement in &statements[index + 1..] {
            uses.visit_stmt(statement);
        }
        if !uses.valid {
            continue;
        }
        let syn::Stmt::Local(local) = &mut statements[index] else {
            continue;
        };
        let Some(init) = &mut local.init else {
            continue;
        };
        let syn::Expr::Macro(rust_macro) = init.expr.as_ref() else {
            continue;
        };
        let Ok(elements) = rust_macro.mac.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) else {
            continue;
        };
        *init.expr = syn::Expr::Array(syn::ExprArray {
            attrs: Vec::new(),
            bracket_token: Default::default(),
            elems: elements,
        });
    }
}

struct ArrayCompatibleUse<'name> {
    name: &'name str,
    valid: bool,
}

impl Visit<'_> for ArrayCompatibleUse<'_> {
    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        if matches!(call.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident(self.name))
        {
            if !matches!(
                call.method.to_string().as_str(),
                "as_slice" | "contains" | "first" | "get" | "is_empty" | "iter" | "last" | "len"
            ) {
                self.valid = false;
            }
            for argument in &call.args {
                self.visit_expr(argument);
            }
            return;
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if path.path.is_ident(self.name) {
            self.valid = false;
        }
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

#[cfg(test)]
mod item12_tests {
    use super::*;

    #[test]
    fn item12_integer_float_power_uses_powi() {
        let mut expression: syn::Expr = syn::parse_quote!(2_f64.powf(10_f64));
        assert!(rewrite_integer_float_power(&mut expression));
        assert_eq!(
            expression.to_token_stream().to_string(),
            "2_f64 . powi (10)"
        );
    }
}
