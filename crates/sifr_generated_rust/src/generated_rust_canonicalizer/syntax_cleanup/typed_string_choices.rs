// A copied string may be selected lazily only across a plain bool read.
// Calls, operators, macros and captures in the condition keep the snapshot eager.
impl Rewriter<'_> {
    fn fold_inert_string_choices(&mut self, block: &mut syn::Block) {
        let outer = self.bindings.clone();
        let mut index = 0;
        while index < block.stmts.len() {
            if let syn::Stmt::Local(local) = &block.stmts[index] {
                if let Some(choice) = block.stmts.get(index + 1)
                    .and_then(|following| self.inert_string_choice(local, following)) {
                    if let syn::Stmt::Local(local) = &mut block.stmts[index]
                        && let Some(init) = &mut local.init {
                        *init.expr = choice;
                    }
                    block.stmts.remove(index + 1);
                }
                if let syn::Stmt::Local(local) = &block.stmts[index] {
                    let ty = local.init.as_ref().and_then(|init| self.ty(&init.expr));
                    self.bind(&local.pat, ty);
                }
            }
            index += 1;
        }
        self.bindings = outer;
    }

    fn inert_string_choice(&self, local: &syn::Local, following: &syn::Stmt) -> Option<syn::Expr> {
        if !local.attrs.is_empty() || !self.clone_is_unambiguous() { return None; }
        let syn::Pat::Type(typed) = &local.pat else { return None };
        if !self.standard_named(&typed.ty, "String") { return None; }
        let syn::Pat::Ident(binding) = typed.pat.as_ref() else { return None };
        if binding.by_ref.is_some() || binding.subpat.is_some() { return None; }
        let init = local.init.as_ref().filter(|init| init.diverge.is_none())?;
        let syn::Expr::MethodCall(copy) = init.expr.as_ref() else { return None };
        if !copy.args.is_empty() || !matches!(copy.method.to_string().as_str(), "clone" | "to_string" | "to_owned")
            || !matches!(copy.receiver.as_ref(), syn::Expr::Path(path) if path.path.get_ident().is_some())
            || !self.ty(&copy.receiver).is_some_and(|ty| self.standard_named(unreference(&ty), "String")
                || self.standard_named(unreference(&ty), "str")) { return None; }
        let syn::Stmt::Expr(syn::Expr::If(branch), _) = following else { return None };
        if !branch.attrs.is_empty() || branch.else_branch.is_some()
            || !matches!(branch.cond.as_ref(), syn::Expr::Path(path) if path.path.get_ident().is_some())
            || !self.ty(&branch.cond).is_some_and(|ty| self.standard_named(&ty, "bool")) { return None; }
        let [syn::Stmt::Expr(syn::Expr::Assign(assign), Some(_))] = branch.then_branch.stmts.as_slice() else { return None };
        if !assign.attrs.is_empty()
            || !matches!(assign.left.as_ref(), syn::Expr::Path(path) if path.path.is_ident(&binding.ident))
            || !matches!(assign.right.as_ref(), syn::Expr::Path(path) if path.path.get_ident().is_some() && !path.path.is_ident(&binding.ident))
            || !self.ty(&assign.right).is_some_and(|ty| self.standard_named(&ty, "String")) { return None; }
        let condition = &branch.cond;
        let value = &assign.right;
        let default = &init.expr;
        Some(syn::parse_quote!(if #condition { #value } else { #default }))
    }
}
