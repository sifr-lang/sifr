// A fresh local has not been referenced since its declaration. It cannot have
// an outstanding borrow or closure capture. Transfers below consume only inert
// values and remain inside the lexical block that declared the owner.
impl Rewriter<'_> {
    fn transfer_fresh_shadowed_option(
        &self,
        local: &mut syn::Local,
        fresh: &std::collections::HashSet<String>,
    ) {
        if !self.clone_is_unambiguous() {
            return;
        }
        let syn::Pat::TupleStruct(pattern) = &local.pat else {
            return;
        };
        if !pattern.path.is_ident("Some") || pattern.elems.len() != 1 {
            return;
        }
        let syn::Pat::Ident(binding) = &pattern.elems[0] else {
            return;
        };
        if binding.by_ref.is_some() || binding.subpat.is_some() {
            return;
        }
        let name = binding.ident.to_string();
        if !fresh.contains(&name)
            || !self
                .bindings
                .get(&name)
                .and_then(Option::as_ref)
                .is_some_and(|ty| {
                    self.standard_generic(ty, "Option").is_some() && self.inert_owned_type(ty)
                })
        {
            return;
        }
        let Some(init) = &mut local.init else {
            return;
        };
        if init.diverge.as_ref().is_some_and(|(_, expression)| {
            statements_reference(&[syn::Stmt::Expr(*expression.clone(), None)], &name)
        }) {
            return;
        }
        let syn::Expr::MethodCall(clone) = init.expr.as_ref() else {
            return;
        };
        if clone.method == "clone"
            && clone.args.is_empty()
            && matches!(clone.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident(&name))
        {
            init.expr = clone.receiver.clone();
        }
    }

    fn transfer_fresh_terminal_branch(
        &self,
        statement: &mut syn::Stmt,
        fresh: &std::collections::HashSet<String>,
    ) {
        if !self.clone_is_unambiguous() {
            return;
        }
        let syn::Stmt::Expr(syn::Expr::If(branch), _) = statement else {
            return;
        };
        if !matches!(
            branch.then_branch.stmts.last(),
            Some(syn::Stmt::Expr(syn::Expr::Return(_), _))
        ) {
            return;
        }
        let condition = branch.cond.to_token_stream().to_string();
        let Some((returned, prefix)) = branch.then_branch.stmts.split_last() else {
            return;
        };
        let owned = fresh
            .iter()
            .filter(|name| {
                !condition
                    .split(|character: char| !character.is_alphanumeric() && character != '_')
                    .any(|token| token == name.as_str())
                    && !statements_reference(prefix, name)
                    && sole_uncaptured_root_use(returned, name)
            })
            .cloned()
            .collect();
        super::idiom_cleanup::clean_owned_suffix(&mut branch.then_branch.stmts, owned);
    }
}

impl Rewriter<'_> {
    // A projection search compares one inert field and returns a cloned String
    // field. Borrowing each element avoids cloning the entire record on every
    // iteration; neither the predicate nor the returned value consumes it.
    fn borrow_inert_projection_search(&self, loop_: &mut syn::ExprForLoop) {
        if !self.iteration_dispatch_closed || !self.clone_is_unambiguous() {
            return;
        }
        let syn::Pat::Ident(binding) = loop_.pat.as_ref() else {
            return;
        };
        if binding.by_ref.is_some() || binding.subpat.is_some() {
            return;
        }
        let name = binding.ident.to_string();
        let Some(element) = self.iterator_element(&loop_.expr) else {
            return;
        };
        if !self.inert_owned_type(&element) {
            return;
        }
        let syn::Expr::MethodCall(cloned) = loop_.expr.as_ref() else {
            return;
        };
        if cloned.method != "cloned" || !cloned.args.is_empty() {
            return;
        }
        let syn::Expr::MethodCall(iterated) = cloned.receiver.as_ref() else {
            return;
        };
        if iterated.method != "iter" || !iterated.args.is_empty() {
            return;
        }
        let [syn::Stmt::Expr(syn::Expr::If(branch), _)] = loop_.body.stmts.as_slice() else {
            return;
        };
        if branch.else_branch.is_some() {
            return;
        }
        let syn::Expr::Binary(comparison) = branch.cond.as_ref() else {
            return;
        };
        if !matches!(comparison.op, syn::BinOp::Eq(_) | syn::BinOp::Ne(_)) {
            return;
        }
        let is_string_field = |expression: &syn::Expr| {
            matches!(expression, syn::Expr::Field(field)
                if matches!(field.base.as_ref(), syn::Expr::Path(path) if path.path.is_ident(&name))
                    && self.field_type(field).is_some_and(|ty| self.standard_named(&ty, "String")))
        };
        let other = if is_string_field(&comparison.left) {
            comparison.right.as_ref()
        } else if is_string_field(&comparison.right) {
            comparison.left.as_ref()
        } else {
            return;
        };
        if statements_reference(&[syn::Stmt::Expr(other.clone(), None)], &name) {
            return;
        }
        if !self.ty(other).is_some_and(|ty| {
            self.standard_named(unreference(&ty), "String")
                || self.standard_named(unreference(&ty), "str")
        }) {
            return;
        }
        let [syn::Stmt::Expr(syn::Expr::Return(returned), _)] = branch.then_branch.stmts.as_slice()
        else {
            return;
        };
        let Some(value) = &returned.expr else {
            return;
        };
        let syn::Expr::Call(call) = value.as_ref() else {
            return;
        };
        if !matches!(call.func.as_ref(), syn::Expr::Path(path) if path.path.is_ident("Some"))
            || call.args.len() != 1
        {
            return;
        }
        let syn::Expr::MethodCall(field_clone) = &call.args[0] else {
            return;
        };
        if field_clone.method != "clone"
            || !field_clone.args.is_empty()
            || !is_string_field(&field_clone.receiver)
        {
            return;
        }
        loop_.expr = cloned.receiver.clone();
    }
}

fn sole_uncaptured_root_use(statement: &syn::Stmt, name: &str) -> bool {
    struct Uses<'a> {
        name: &'a str,
        count: usize,
        opaque: bool,
    }
    impl<'ast> syn::visit::Visit<'ast> for Uses<'_> {
        fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
            if path.path.is_ident(self.name) {
                self.count += 1;
            }
        }
        fn visit_macro(&mut self, _: &'ast syn::Macro) {
            self.opaque = true;
        }
        fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {
            self.opaque = true;
        }
        fn visit_expr_async(&mut self, _: &'ast syn::ExprAsync) {
            self.opaque = true;
        }
        fn visit_block(&mut self, _: &'ast syn::Block) {
            self.opaque = true;
        }
        fn visit_item(&mut self, _: &'ast syn::Item) {
            self.opaque = true;
        }
    }
    let mut uses = Uses {
        name,
        count: 0,
        opaque: false,
    };
    syn::visit::Visit::visit_stmt(&mut uses, statement);
    !uses.opaque && uses.count == 1
}
