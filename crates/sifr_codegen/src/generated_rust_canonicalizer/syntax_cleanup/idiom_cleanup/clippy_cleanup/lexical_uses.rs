#[derive(Default)]
struct IdentifierUseCounter {
    counts: HashMap<String, usize>,
    shadowed: HashSet<String>,
}

#[derive(Default)]
struct ClosureCaptureCollector {
    names: HashSet<String>,
}

impl Visit<'_> for ClosureCaptureCollector {
    fn visit_expr_call(&mut self, call: &syn::ExprCall) {
        if let Some(closure) = immediately_called_closure(&call.func) {
            self.visit_expr(&closure.body);
            for argument in &call.args {
                self.visit_expr(argument);
            }
        } else {
            visit::visit_expr_call(self, call);
        }
    }
    fn visit_expr_closure(&mut self, closure: &syn::ExprClosure) {
        let mut uses = IdentifierUseCounter::default();
        uses.visit_expr_closure(closure);
        self.names.extend(uses.counts.into_keys());
        visit::visit_expr_closure(self, closure);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

impl Visit<'_> for IdentifierUseCounter {
    fn visit_expr_for_loop(&mut self, loop_: &syn::ExprForLoop) {
        self.visit_expr(&loop_.expr);
        let outer = self.shadowed.clone();
        self.shadowed.extend(
            crate::generated_rust_canonicalizer::syntax_cleanup::identifier_names_in_pattern(
                &loop_.pat,
            ),
        );
        self.visit_block(&loop_.body);
        self.shadowed = outer;
    }

    fn visit_expr_while(&mut self, loop_: &syn::ExprWhile) {
        let outer = self.shadowed.clone();
        self.visit_condition(&loop_.cond);
        self.visit_block(&loop_.body);
        self.shadowed = outer;
    }

    fn visit_expr_match(&mut self, match_: &syn::ExprMatch) {
        self.visit_expr(&match_.expr);
        for arm in &match_.arms {
            let outer = self.shadowed.clone();
            self.shadowed.extend(
                crate::generated_rust_canonicalizer::syntax_cleanup::identifier_names_in_pattern(
                    &arm.pat,
                ),
            );
            self.visit_pat(&arm.pat);
            self.visit_expr(&arm.body);
            self.shadowed = outer;
        }
    }
    fn visit_block(&mut self, block: &syn::Block) {
        let outer = self.shadowed.clone();
        for statement in &block.stmts {
            self.visit_stmt(statement);
            if let syn::Stmt::Local(local) = statement {
                self.shadowed.extend(crate::generated_rust_canonicalizer::syntax_cleanup::identifier_names_in_pattern(&local.pat));
            }
        }
        self.shadowed = outer;
    }

    fn visit_expr_if(&mut self, branch: &syn::ExprIf) {
        let outer = self.shadowed.clone();
        self.visit_condition(&branch.cond);
        self.visit_block(&branch.then_branch);
        self.shadowed = outer;
        if let Some((_, alternative)) = &branch.else_branch {
            self.visit_expr(alternative);
        }
    }

    fn visit_expr_closure(&mut self, closure: &syn::ExprClosure) {
        let outer = self.shadowed.clone();
        for input in &closure.inputs {
            self.shadowed.extend(
                crate::generated_rust_canonicalizer::syntax_cleanup::identifier_names_in_pattern(
                    input,
                ),
            );
        }
        self.visit_expr(&closure.body);
        self.shadowed = outer;
    }

    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if path.qself.is_none()
            && path.path.segments.len() == 1
            && let Some(segment) = path.path.segments.first()
            && !self.shadowed.contains(&segment.ident.to_string())
        {
            *self.counts.entry(segment.ident.to_string()).or_default() += 1;
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}

    fn visit_macro(&mut self, rust_macro: &syn::Macro) {
        for name in crate::generated_rust_canonicalizer::format_capture::names(rust_macro) {
            if !self.shadowed.contains(&name) {
                *self.counts.entry(name).or_default() += 1;
            }
        }
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        }
    }
}

impl IdentifierUseCounter {
    fn visit_condition(&mut self, expression: &syn::Expr) {
        match expression {
            syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
                self.visit_condition(&binary.left);
                self.visit_condition(&binary.right);
            }
            syn::Expr::Let(let_) => {
                self.visit_expr(&let_.expr);
                self.shadowed.extend(crate::generated_rust_canonicalizer::syntax_cleanup::identifier_names_in_pattern(&let_.pat));
            }
            _ => self.visit_expr(expression),
        }
    }
}
