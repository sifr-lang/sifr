struct BorrowOnlyLoopBindingUse<'binding> {
    binding: &'binding str,
    owned_use: bool,
}

struct BindingShadowCollector<'binding> {
    binding: &'binding str,
    found: bool,
}

impl Visit<'_> for BindingShadowCollector<'_> {
    fn visit_expr_let(&mut self, let_: &syn::ExprLet) {
        self.visit_expr(&let_.expr);
        if pattern_contains_name(&let_.pat, self.binding) {
            self.found = true;
        }
    }

    fn visit_local(&mut self, local: &syn::Local) {
        if pattern_contains_name(&local.pat, self.binding) {
            self.found = true;
        }
        visit::visit_local(self, local);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

impl Visit<'_> for BorrowOnlyLoopBindingUse<'_> {
    fn visit_expr_if(&mut self, branch: &syn::ExprIf) {
        if let syn::Expr::Let(let_) = branch.cond.as_ref() {
            self.visit_expr(&let_.expr);
            if !pattern_contains_name(&let_.pat, self.binding) {
                self.visit_block(&branch.then_branch);
            }
            if let Some((_, alternative)) = &branch.else_branch {
                self.visit_expr(alternative);
            }
            return;
        }
        visit::visit_expr_if(self, branch);
    }

    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        if matches!(call.receiver.as_ref(), syn::Expr::Path(path)
            if path.path.is_ident(self.binding))
        {
            for argument in &call.args {
                self.visit_expr(argument);
            }
            return;
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_binary(&mut self, binary: &syn::ExprBinary) {
        if comparison_operator(&binary.op) {
            return;
        }
        visit::visit_expr_binary(self, binary);
    }

    fn visit_expr_reference(&mut self, reference: &syn::ExprReference) {
        if matches!(reference.expr.as_ref(), syn::Expr::Path(path)
            if path.path.is_ident(self.binding))
        {
            return;
        }
        visit::visit_expr_reference(self, reference);
    }

    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if path.qself.is_none() && path.path.is_ident(self.binding) {
            self.owned_use = true;
            return;
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_macro(&mut self, rust_macro: &syn::Macro) {
        if rust_macro.path.get_ident().is_some_and(|name| {
            matches!(
                name.to_string().as_str(),
                "println" | "print" | "format" | "eprintln" | "eprint" | "format_args"
            )
        }) && let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                if !matches!(argument, syn::Expr::Path(path) if path.path.is_ident(self.binding)) {
                    self.visit_expr(argument);
                }
            }
            return;
        }
        if rust_macro
            .tokens
            .to_string()
            .split_whitespace()
            .any(|token| token == self.binding)
        {
            self.owned_use = true;
            return;
        }
        visit::visit_macro(self, rust_macro);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

fn pattern_contains_name(pattern: &syn::Pat, expected: &str) -> bool {
    match pattern {
        syn::Pat::Ident(binding) => binding.ident == expected,
        syn::Pat::Paren(paren) => pattern_contains_name(&paren.pat, expected),
        syn::Pat::Reference(reference) => pattern_contains_name(&reference.pat, expected),
        syn::Pat::Type(typed) => pattern_contains_name(&typed.pat, expected),
        syn::Pat::Tuple(tuple) => tuple
            .elems
            .iter()
            .any(|element| pattern_contains_name(element, expected)),
        syn::Pat::TupleStruct(tuple) => tuple
            .elems
            .iter()
            .any(|element| pattern_contains_name(element, expected)),
        _ => false,
    }
}
