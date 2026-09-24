fn rewrite_vec_copy_extend(expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(extend) = expression else {
        return false;
    };
    if extend.method != "extend" || extend.args.len() != 1 {
        return false;
    }
    let Some(syn::Expr::MethodCall(copied)) = extend.args.first() else {
        return false;
    };
    if copied.method != "copied" || !copied.args.is_empty() {
        return false;
    }
    let syn::Expr::MethodCall(iterated) = copied.receiver.as_ref() else {
        return false;
    };
    if iterated.method != "iter" || !iterated.args.is_empty() {
        return false;
    }
    let syn::Expr::Macro(vector) = iterated.receiver.as_ref() else {
        return false;
    };
    if !vector.mac.path.is_ident("vec") {
        return false;
    }
    let elements = vector.mac.tokens.clone();
    extend.args[0] = syn::parse_quote!([#elements]);
    true
}

fn rewrite_negated_sifr_int_literal(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Unary(negation) = expression else {
        return false;
    };
    if !matches!(negation.op, syn::UnOp::Neg(_)) {
        return false;
    }
    let syn::Expr::Call(call) = negation.expr.as_ref() else {
        return false;
    };
    if call.args.len() != 1
        || !matches!(call.func.as_ref(), syn::Expr::Path(path)
            if path.path.segments.len() == 2
                && path.path.segments[0].ident == "SifrInt"
                && path.path.segments[1].ident == "from_i64")
    {
        return false;
    }
    let Some(syn::Expr::Lit(literal)) = call.args.first() else {
        return false;
    };
    let syn::Lit::Int(value) = &literal.lit else {
        return false;
    };
    let value = value.clone();
    *expression = syn::parse_quote!(SifrInt::from_i64(-#value));
    true
}

fn rewrite_double_negated_is_empty(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Unary(outer) = expression else {
        return false;
    };
    if !matches!(outer.op, syn::UnOp::Not(_)) {
        return false;
    }
    let syn::Expr::Unary(inner) = outer.expr.as_ref() else {
        return false;
    };
    if !matches!(inner.op, syn::UnOp::Not(_))
        || !matches!(inner.expr.as_ref(), syn::Expr::MethodCall(call)
            if call.method == "is_empty" && call.args.is_empty())
    {
        return false;
    }
    *expression = inner.expr.as_ref().clone();
    true
}




fn rewrite_borrowed_callback_arguments(expression: &mut syn::Expr) -> bool {
    let syn::Expr::Call(call) = expression else {
        return false;
    };
    let syn::Expr::Path(function) = call.func.as_ref() else {
        return false;
    };
    if !function.path.segments.last().is_some_and(|segment| {
        matches!(
            segment.ident.to_string().as_str(),
            "dropwhile" | "takewhile" | "filterfalse" | "starmap"
        )
    }) {
        return false;
    }
    let Some(syn::Expr::Closure(closure)) = call.args.first_mut() else {
        return false;
    };
    let bindings = closure
        .inputs
        .iter()
        .filter_map(simple_pattern_name)
        .collect::<HashSet<_>>();
    if bindings.is_empty() {
        return false;
    }
    let mut rewriter = BorrowedCallbackArgumentRewriter {
        bindings: &bindings,
        changed: false,
    };
    rewriter.visit_expr_mut(&mut closure.body);
    rewriter.changed
}

struct BorrowedCallbackArgumentRewriter<'bindings> {
    bindings: &'bindings HashSet<String>,
    changed: bool,
}

impl VisitMut for BorrowedCallbackArgumentRewriter<'_> {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        let syn::Expr::Reference(reference) = expression else {
            return;
        };
        if reference.mutability.is_none()
            && matches!(reference.expr.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name|
                    self.bindings.contains(&name.to_string())))
        {
            *expression = reference.expr.as_ref().clone();
            self.changed = true;
        }
    }

    fn visit_expr_closure_mut(&mut self, _closure: &mut syn::ExprClosure) {}
}
