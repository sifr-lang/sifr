struct ScalarCallRewriter<'plans> {
    plans: &'plans HashMap<String, ScalarBorrowPlan>,
    modules: Vec<String>,
    functions: Vec<String>,
    owner: Option<String>,
    borrowed_bindings: HashSet<String>,
    optional_borrowed_bindings: HashSet<String>,
    owned_optional_bindings: HashSet<String>,
}

impl VisitMut for ScalarCallRewriter<'_> {
    fn visit_item_impl_mut(&mut self, implementation: &mut syn::ItemImpl) {
        let previous = self.owner.replace(type_owner_name(&implementation.self_ty));
        visit_mut::visit_item_impl_mut(self, implementation);
        self.owner = previous;
    }
    fn visit_item_mod_mut(&mut self, module: &mut syn::ItemMod) {
        let Some((_, items)) = &mut module.content else {
            return;
        };
        self.modules.push(module.ident.to_string());
        for item in items {
            self.visit_item_mut(item);
        }
        self.modules.pop();
    }

    fn visit_item_fn_mut(&mut self, function: &mut syn::ItemFn) {
        self.functions.push(function.sig.ident.to_string());
        self.visit_function(&function.sig, &mut function.block);
        self.functions.pop();
    }

    fn visit_impl_item_fn_mut(&mut self, function: &mut syn::ImplItemFn) {
        self.functions.push(function.sig.ident.to_string());
        self.visit_function(&function.sig, &mut function.block);
        self.functions.pop();
    }

    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let outer = self.borrowed_bindings.clone();
        let outer_optional = self.optional_borrowed_bindings.clone();
        let outer_owned_optional = self.owned_optional_bindings.clone();
        for statement in &mut block.stmts {
            self.visit_stmt_mut(statement);
            if let syn::Stmt::Local(local) = statement {
                for name in super::identifier_names_in_pattern(&local.pat) {
                    self.borrowed_bindings.remove(&name);
                    self.optional_borrowed_bindings.remove(&name);
                    self.owned_optional_bindings.remove(&name);
                }
                if let Some(name) = simple_pattern_name(&local.pat)
                    && matches!(&local.pat, syn::Pat::Type(typed) if borrowed_scalar_type(&typed.ty))
                {
                    self.borrowed_bindings.insert(name);
                } else if let Some(name) = simple_pattern_name(&local.pat)
                    && matches!(&local.pat, syn::Pat::Type(typed)
                        if borrowed_sifr_int_option_type(&typed.ty))
                {
                    self.optional_borrowed_bindings.insert(name);
                } else if let Some(name) = simple_pattern_name(&local.pat)
                    && matches!(&local.pat, syn::Pat::Type(typed)
                        if owned_sifr_int_option_type(&typed.ty))
                {
                    self.owned_optional_bindings.insert(name);
                }
            }
        }
        self.borrowed_bindings = outer;
        self.optional_borrowed_bindings = outer_optional;
        self.owned_optional_bindings = outer_owned_optional;
    }

    fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
        visit_mut::visit_expr_call_mut(self, call);
        let syn::Expr::Path(path) = call.func.as_ref() else {
            return;
        };
        if path.qself.is_some() {
            return;
        }
        let mut callee = path.path.clone();
        if let Some(first) = callee.segments.first_mut()
            && first.ident == "Self"
            && let Some(owner) = &self.owner
        {
            first.ident = syn::Ident::new(owner, first.ident.span());
        }
        let Some(plan) = call_plan(
            self.plans,
            &self.modules,
            &self.functions,
            &callee,
            call.args.len(),
        ) else {
            return;
        };
        for index in &plan.borrowed {
            let Some(argument) = call.args.get_mut(*index) else {
                continue;
            };
            rewrite_borrowed_argument(argument, &self.borrowed_bindings);
        }
        for index in &plan.optional {
            let Some(argument) = call.args.get_mut(*index) else {
                continue;
            };
            rewrite_optional_borrowed_argument(
                argument,
                &self.optional_borrowed_bindings,
                &self.owned_optional_bindings,
            );
        }
    }

    fn visit_macro_mut(&mut self, rust_macro: &mut syn::Macro) {
        let Ok(mut arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) else {
            return;
        };
        for argument in &mut arguments {
            self.visit_expr_mut(argument);
        }
        rust_macro.tokens = quote::quote!(#arguments);
    }
}

impl ScalarCallRewriter<'_> {
    fn visit_function(&mut self, signature: &syn::Signature, block: &mut syn::Block) {
        let previous = std::mem::take(&mut self.borrowed_bindings);
        let previous_optional = std::mem::take(&mut self.optional_borrowed_bindings);
        let previous_owned_optional = std::mem::take(&mut self.owned_optional_bindings);
        self.borrowed_bindings = signature
            .inputs
            .iter()
            .filter_map(|argument| {
                let syn::FnArg::Typed(parameter) = argument else {
                    return None;
                };
                borrowed_scalar_type(&parameter.ty)
                    .then(|| simple_pattern_name(&parameter.pat))
                    .flatten()
            })
            .collect();
        self.optional_borrowed_bindings = signature
            .inputs
            .iter()
            .filter_map(|argument| {
                let syn::FnArg::Typed(parameter) = argument else {
                    return None;
                };
                borrowed_sifr_int_option_type(&parameter.ty)
                    .then(|| simple_pattern_name(&parameter.pat))
                    .flatten()
            })
            .collect();
        self.owned_optional_bindings = signature
            .inputs
            .iter()
            .filter_map(|argument| {
                let syn::FnArg::Typed(parameter) = argument else {
                    return None;
                };
                owned_sifr_int_option_type(&parameter.ty)
                    .then(|| simple_pattern_name(&parameter.pat))
                    .flatten()
            })
            .collect();
        self.visit_block_mut(block);
        self.borrowed_bindings = previous;
        self.optional_borrowed_bindings = previous_optional;
        self.owned_optional_bindings = previous_owned_optional;
    }
}
