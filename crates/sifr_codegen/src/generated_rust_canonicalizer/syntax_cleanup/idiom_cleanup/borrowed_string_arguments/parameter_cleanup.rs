pub(crate) fn rewrite_borrow_only_string_parameters(file: &mut syn::File) {
    BorrowOnlyStringParameterRewriter.visit_file_mut(file);
}

struct BorrowOnlyStringParameterRewriter;

impl VisitMut for BorrowOnlyStringParameterRewriter {
    fn visit_item_fn_mut(&mut self, function: &mut syn::ItemFn) {
        rewrite_string_signature(&mut function.sig, &mut function.block);
        visit_mut::visit_item_fn_mut(self, function);
    }

    fn visit_impl_item_fn_mut(&mut self, function: &mut syn::ImplItemFn) {
        rewrite_string_signature(&mut function.sig, &mut function.block);
        visit_mut::visit_impl_item_fn_mut(self, function);
    }
}

fn rewrite_string_signature(signature: &mut syn::Signature, block: &mut syn::Block) {
    let mut borrowed = HashSet::new();
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
            borrowed.insert(name);
        }
    }
    BorrowedStringParameterBodyRewriter { borrowed }.visit_block_mut(block);
}

struct BorrowOnlyStringUses<'name> {
    name: &'name str,
    seen: bool,
    unsupported: bool,
}

impl Visit<'_> for BorrowOnlyStringUses<'_> {
    fn visit_expr_reference(&mut self, reference: &syn::ExprReference) {
        if expression_is_binding(&reference.expr, self.name) && reference.mutability.is_none() {
            self.seen = true;
            return;
        }
        visit::visit_expr_reference(self, reference);
    }

    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        if expression_is_binding(&call.receiver, self.name) {
            self.seen = true;
            if call.method.to_string().starts_with("into_")
                || matches!(
                    call.method.to_string().as_str(),
                    "clear" | "drain" | "push" | "push_str"
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

struct BorrowedStringParameterBodyRewriter {
    borrowed: HashSet<String>,
}

impl VisitMut for BorrowedStringParameterBodyRewriter {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let outer = self.borrowed.clone();
        for statement in &mut block.stmts {
            self.visit_stmt_mut(statement);
            if let syn::Stmt::Local(local) = statement
                && let Some(name) = simple_pattern_name(&local.pat)
            {
                self.borrowed.remove(&name);
            }
        }
        self.borrowed = outer;
    }

    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        if let syn::Expr::Reference(reference) = expression
            && reference.mutability.is_none()
            && expression_is_any_binding(&reference.expr, &self.borrowed)
        {
            *expression = reference.expr.as_ref().clone();
            return;
        }
        if let syn::Expr::MethodCall(call) = expression
            && call.method == "as_str"
            && call.args.is_empty()
            && expression_is_any_binding(&call.receiver, &self.borrowed)
        {
            *expression = call.receiver.as_ref().clone();
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn expression_is_binding(expression: &syn::Expr, name: &str) -> bool {
    matches!(expression, syn::Expr::Path(path) if path.path.is_ident(name))
}

fn expression_is_any_binding(expression: &syn::Expr, names: &HashSet<String>) -> bool {
    matches!(expression, syn::Expr::Path(path)
        if path.path.get_ident().is_some_and(|name| names.contains(&name.to_string())))
}
