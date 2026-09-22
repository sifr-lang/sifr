pub(crate) fn rewrite_borrow_only_string_parameters(file: &mut syn::File) {
    // Each successful round removes owned String parameters. Solve this finite
    // signature dependency graph here rather than consuming syntax pass rounds.
    loop {
        let Some(shared_inputs) = super::super::typed_expression_cleanup::shared_string_call_inputs(file) else {
            return;
        };
        let mut rewriter = BorrowOnlyStringParameterRewriter {
            shared_inputs,
            ambiguous_clones: super::ambiguous_clone_scopes(file),
            retained_abis: super::super::borrowed_scalar_parameters::callable_value_abi_keys(file),
            scope: Vec::new(), owner: None, trait_implementation: false, changes: 0,
        };
        rewriter.visit_file_mut(file);
        if rewriter.changes == 0 { return; }
    }
}

struct BorrowOnlyStringParameterRewriter {
    shared_inputs: HashMap<String, Vec<bool>>,
    ambiguous_clones: HashSet<String>,
    changes: usize,
    retained_abis: HashSet<String>,
    scope: Vec<String>,
    owner: Option<String>,
    trait_implementation: bool,
}

impl BorrowOnlyStringParameterRewriter {
    fn retains(&self, signature: &syn::Signature) -> bool {
        let mut path = self.scope.clone();
        let kind = if let Some(owner) = &self.owner { path.push(owner.clone()); "method" } else { "function" };
        path.push(signature.ident.to_string());
        let count = signature.inputs.iter().filter(|input| matches!(input, syn::FnArg::Typed(_))).count();
        self.trait_implementation || self.retained_abis.contains(&format!("{kind}:{}#{count}", path.join("::")))
    }
}

impl VisitMut for BorrowOnlyStringParameterRewriter {
    fn visit_item_mod_mut(&mut self, module: &mut syn::ItemMod) {
        self.scope.push(module.ident.to_string());
        visit_mut::visit_item_mod_mut(self, module);
        self.scope.pop();
    }
    fn visit_item_impl_mut(&mut self, implementation: &mut syn::ItemImpl) {
        let owner = match implementation.self_ty.as_ref() {
            syn::Type::Path(path) => Some(path.path.segments.iter().map(|part| part.ident.to_string()).collect::<Vec<_>>().join("::")),
            _ => None,
        };
        let previous_owner = std::mem::replace(&mut self.owner, owner);
        let previous_trait = std::mem::replace(&mut self.trait_implementation, implementation.trait_.is_some());
        visit_mut::visit_item_impl_mut(self, implementation);
        self.owner = previous_owner;
        self.trait_implementation = previous_trait;
    }
    fn visit_item_fn_mut(&mut self, function: &mut syn::ItemFn) {
        if !self.retains(&function.sig) { self.rewrite_signature(&mut function.sig, &mut function.block); }
        self.scope.push(function.sig.ident.to_string());
        visit_mut::visit_item_fn_mut(self, function);
        self.scope.pop();
    }

    fn visit_impl_item_fn_mut(&mut self, function: &mut syn::ImplItemFn) {
        if !self.retains(&function.sig) { self.rewrite_signature(&mut function.sig, &mut function.block); }
        visit_mut::visit_impl_item_fn_mut(self, function);
    }
}

impl BorrowOnlyStringParameterRewriter {
fn rewrite_signature(&mut self, signature: &mut syn::Signature, block: &mut syn::Block) {
    let mut context = StringCallContext::new(&self.shared_inputs, &self.scope, signature, block);
    context.clone_unambiguous = !(0..=self.scope.len())
        .any(|depth| self.ambiguous_clones.contains(&self.scope[..depth].join("::")));
    let mut copied_parameters = HashSet::new();
    for input in &mut signature.inputs {
        let syn::FnArg::Typed(parameter) = input else {
            continue;
        };
        if !matches!(parameter.ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("String")) {
            continue;
        }
        let Some(name) = simple_pattern_name(&parameter.pat) else {
            continue;
        };
        let mut uses = BorrowOnlyStringUses {
            name: &name,
            context: &context,
            seen: false,
            unsupported: false,
        };
        uses.visit_block(block);
        if uses.seen && !uses.unsupported {
            *parameter.ty = syn::parse_quote!(&str);
            copied_parameters.insert(name);
            self.changes += 1;
        }
    }
    if !copied_parameters.is_empty() { StringCopiesToOwned(&copied_parameters).visit_block_mut(block); }
}

}

struct StringCopiesToOwned<'a>(&'a HashSet<String>);
impl VisitMut for StringCopiesToOwned<'_> {
    fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
        visit_mut::visit_expr_method_call_mut(self, call);
        if call.method == "clone" && call.args.is_empty()
            && matches!(call.receiver.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name| self.0.contains(&name.to_string()))) {
            call.method = syn::Ident::new("to_string", call.method.span());
        }
    }
    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
    fn visit_macro_mut(&mut self, rust_macro: &mut syn::Macro) {
        if let Ok(mut arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated) {
            for argument in &mut arguments { self.visit_expr_mut(argument); }
            rust_macro.tokens = quote::quote!(#arguments);
        }
    }
}

struct BorrowOnlyStringUses<'name> {
    name: &'name str,
    context: &'name StringCallContext<'name>,
    seen: bool,
    unsupported: bool,
}

impl Visit<'_> for BorrowOnlyStringUses<'_> {
    fn visit_block(&mut self, block: &syn::Block) {
        for (index, statement) in block.stmts.iter().enumerate() {
            if let syn::Stmt::Local(local) = statement
                && let syn::Pat::Ident(alias) = &local.pat
                && alias.subpat.is_none() && alias.by_ref.is_none()
                && let Some(init) = &local.init
                && let syn::Expr::MethodCall(copy) = init.expr.as_ref()
                && copy.method == "clone" && copy.args.is_empty()
                && expression_is_binding(&copy.receiver, self.name)
            {
                let name = alias.ident.to_string();
                let mut uses = BorrowOnlyStringUses { name: &name, context: self.context, seen: false, unsupported: false };
                for later in &block.stmts[index + 1..] { uses.visit_stmt(later); }
                if uses.seen && !uses.unsupported { self.seen = true; continue; }
            }
            self.visit_stmt(statement);
        }
    }

    fn visit_expr_call(&mut self, call: &syn::ExprCall) {
        let inputs = self.context.inputs(call);
        self.visit_expr(&call.func);
        for (index, argument) in call.args.iter().enumerate() {
            if inputs.and_then(|inputs| inputs.get(index)).copied().unwrap_or(false)
                && (expression_is_binding(argument, self.name) || matches!(argument, syn::Expr::Reference(reference)
                    if reference.mutability.is_none() && expression_is_binding(&reference.expr, self.name))) {
                self.seen = true;
            } else {
                self.visit_expr(argument);
            }
        }
    }

    fn visit_expr_reference(&mut self, reference: &syn::ExprReference) {
        visit::visit_expr_reference(self, reference);
    }

    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        if expression_is_binding(&call.receiver, self.name) {
            self.seen = true;
            if call.method == "clone" && call.args.is_empty() && self.context.clone_unambiguous {
                return;
            }
            if !matches!(
                    call.method.to_string().as_str(),
                    "as_str" | "len" | "is_empty" | "chars" | "bytes" | "trim" | "trim_start" | "trim_end" | "contains" | "starts_with" | "ends_with" | "find" | "rfind" | "split" | "splitn" | "rsplitn" | "replace" | "replacen" | "to_lowercase" | "to_uppercase" | "is_ascii"
                )
            {
                self.unsupported = true;
            }
            for argument in &call.args {
                self.visit_expr(argument);
            }
            return;
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_binary(&mut self, binary: &syn::ExprBinary) {
        if expression_is_binding(&binary.left, self.name)
            || expression_is_binding(&binary.right, self.name)
        {
            self.seen = true;
            let other = if expression_is_binding(&binary.left, self.name) {
                &binary.right
            } else {
                &binary.left
            };
            self.visit_expr(other);
            return;
        }
        visit::visit_expr_binary(self, binary);
    }

    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if path.path.is_ident(self.name) {
            self.seen = true;
            self.unsupported = true;
        }
    }

    fn visit_item(&mut self, _item: &syn::Item) {}

    fn visit_pat_ident(&mut self, binding: &syn::PatIdent) {
        if binding.ident == self.name { self.unsupported = true; }
        visit::visit_pat_ident(self, binding);
    }

    fn visit_macro(&mut self, rust_macro: &syn::Macro) {
        if !rust_macro.path.get_ident().is_some_and(|name| matches!(name.to_string().as_str(),
            "assert" | "assert_eq" | "assert_ne" | "debug_assert" | "debug_assert_eq"
            | "debug_assert_ne" | "print" | "println" | "eprint" | "eprintln"
            | "format" | "format_args" | "write" | "writeln" | "vec")) {
            self.unsupported = true;
            return;
        }
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        } else {
            self.unsupported = true;
        }
    }
}


fn expression_is_binding(expression: &syn::Expr, name: &str) -> bool {
    matches!(expression, syn::Expr::Path(path) if path.path.is_ident(name))
}
