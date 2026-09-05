struct DoubleReferenceCloneFromRewriter {
    active: HashSet<String>,
}

impl VisitMut for DoubleReferenceCloneFromRewriter {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let outer = self.active.clone();
        for statement in &mut block.stmts {
            self.visit_stmt_mut(statement);
            if let syn::Stmt::Local(local) = statement {
                for name in pattern_binding_names(&local.pat) {
                    self.active.remove(&name);
                }
                if local.init.as_ref().is_some_and(|init| {
                    matches!(init.expr.as_ref(), syn::Expr::MethodCall(call)
                        if matches!(call.method.to_string().as_str(), "as_ref" | "as_deref")
                            && call.args.is_empty())
                }) {
                    self.active.extend(pattern_binding_names(&local.pat));
                }
            }
        }
        self.active = outer;
    }

    fn visit_expr_if_mut(&mut self, branch: &mut syn::ExprIf) {
        self.visit_expr_mut(&mut branch.cond);
        let outer = self.active.clone();
        if let syn::Expr::Let(let_) = branch.cond.as_ref() {
            for name in pattern_binding_names(&let_.pat) {
                self.active.remove(&name);
            }
            if matches!(let_.expr.as_ref(), syn::Expr::MethodCall(call)
                if matches!(call.method.to_string().as_str(), "as_ref" | "as_deref")
                    && call.args.is_empty())
            {
                self.active.extend(pattern_binding_names(&let_.pat));
            }
        }
        self.visit_block_mut(&mut branch.then_branch);
        self.active = outer.clone();
        if let Some((_, alternative)) = &mut branch.else_branch {
            self.visit_expr_mut(alternative);
        }
        self.active = outer;
    }

    fn visit_expr_match_mut(&mut self, match_: &mut syn::ExprMatch) {
        self.visit_expr_mut(&mut match_.expr);
        let outer = self.active.clone();
        for arm in &mut match_.arms {
            self.active = outer.clone();
            for name in pattern_binding_names(&arm.pat) {
                self.active.remove(&name);
            }
            if let syn::Pat::Guard(guard) = &mut arm.pat {
                self.visit_expr_mut(&mut guard.guard);
            }
            self.visit_expr_mut(&mut arm.body);
        }
        self.active = outer;
    }

    fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
        visit_mut::visit_expr_method_call_mut(self, call);
        if call.method != "clone_from" || call.args.len() != 1 {
            return;
        }
        let Some(syn::Expr::Reference(reference)) = call.args.first() else {
            return;
        };
        if reference.mutability.is_none() {
            if matches!(reference.expr.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name|
                    self.active.contains(&name.to_string())))
            {
                call.args[0] = reference.expr.as_ref().clone();
            } else if let syn::Expr::Unary(dereference) = reference.expr.as_ref()
                && matches!(dereference.op, syn::UnOp::Deref(_))
                && matches!(dereference.expr.as_ref(), syn::Expr::Path(path)
                    if path.path.get_ident().is_some_and(|name|
                        self.active.contains(&name.to_string())))
            {
                call.args[0] = dereference.expr.as_ref().clone();
            }
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}
