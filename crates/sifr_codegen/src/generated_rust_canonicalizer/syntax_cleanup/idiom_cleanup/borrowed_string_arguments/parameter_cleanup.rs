pub(crate) fn rewrite_borrow_only_string_parameters(file: &mut syn::File) {
    BorrowOnlyStringParameterRewriter {
        retained_abis: super::super::borrowed_scalar_parameters::callable_value_abi_keys(file),
        scope: Vec::new(), owner: None, trait_implementation: false,
    }.visit_file_mut(file);
}

struct BorrowOnlyStringParameterRewriter {
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
        if !self.retains(&function.sig) { rewrite_string_signature(&mut function.sig, &function.block); }
        self.scope.push(function.sig.ident.to_string());
        visit_mut::visit_item_fn_mut(self, function);
        self.scope.pop();
    }

    fn visit_impl_item_fn_mut(&mut self, function: &mut syn::ImplItemFn) {
        if !self.retains(&function.sig) { rewrite_string_signature(&mut function.sig, &function.block); }
        visit_mut::visit_impl_item_fn_mut(self, function);
    }
}

fn rewrite_string_signature(signature: &mut syn::Signature, block: &syn::Block) {
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
            seen: false,
            unsupported: false,
        };
        uses.visit_block(block);
        if uses.seen && !uses.unsupported {
            *parameter.ty = syn::parse_quote!(&str);
        }
    }
}

struct BorrowOnlyStringUses<'name> {
    name: &'name str,
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
                let mut uses = BorrowOnlyStringUses { name: &name, seen: false, unsupported: false };
                for later in &block.stmts[index + 1..] { uses.visit_stmt(later); }
                if uses.seen && !uses.unsupported { self.seen = true; continue; }
            }
            self.visit_stmt(statement);
        }
    }

    fn visit_expr_reference(&mut self, reference: &syn::ExprReference) {
        visit::visit_expr_reference(self, reference);
    }

    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        if expression_is_binding(&call.receiver, self.name) {
            self.seen = true;
            if !matches!(
                    call.method.to_string().as_str(),
                    "len" | "is_empty" | "chars" | "bytes" | "trim" | "trim_start" | "trim_end" | "contains" | "starts_with" | "ends_with" | "find" | "rfind" | "split" | "splitn" | "rsplitn" | "replace" | "replacen" | "to_lowercase" | "to_uppercase" | "is_ascii"
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
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        }
    }
}


fn expression_is_binding(expression: &syn::Expr, name: &str) -> bool {
    matches!(expression, syn::Expr::Path(path) if path.path.is_ident(name))
}
