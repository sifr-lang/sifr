pub(super) fn remove_write_only_cached_strings(block: &mut syn::Block) {
    let cached = block
        .stmts
        .iter()
        .filter_map(|statement| {
            let syn::Stmt::Local(local) = statement else {
                return None;
            };
            let name = simple_pattern_name(&local.pat)?;
            name.strip_prefix("sifr_generated_chars_")
                .map(ToString::to_string)
        })
        .collect::<HashSet<_>>();
    if cached.is_empty() {
        return;
    }
    let candidates = block
        .stmts
        .iter()
        .filter_map(|statement| {
            let syn::Stmt::Local(local) = statement else {
                return None;
            };
            let syn::Pat::Type(typed) = &local.pat else {
                return None;
            };
            let name = simple_pattern_name(&typed.pat)?;
            (cached.contains(&name) && type_is_owned_string(&typed.ty)).then_some(name)
        })
        .filter(|name| {
            let mut uses = WriteOnlyStringUse {
                name,
                read: false,
                removable_writes: true,
            };
            uses.visit_block(block);
            !uses.read && uses.removable_writes
        })
        .collect::<HashSet<_>>();
    if candidates.is_empty() {
        return;
    }
    WriteOnlyStringRemover { names: &candidates }.visit_block_mut(block);
}

struct WriteOnlyStringUse<'name> {
    name: &'name str,
    read: bool,
    removable_writes: bool,
}

impl Visit<'_> for WriteOnlyStringUse<'_> {
    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        if matches!(call.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident(self.name))
            && call.method == "push_str"
        {
            self.removable_writes &= call.args.iter().all(discardable_string_write_argument);
            for argument in &call.args {
                self.visit_expr(argument);
            }
            return;
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if path.path.is_ident(self.name) {
            self.read = true;
        }
    }

    fn visit_pat(&mut self, _pattern: &syn::Pat) {}

    fn visit_item(&mut self, _item: &syn::Item) {}
}

fn discardable_string_write_argument(expression: &syn::Expr) -> bool {
    super::super::discardable_expression::expression_is_discardable(expression)
        || matches!(expression, syn::Expr::MethodCall(call)
            if call.method == "as_str"
                && call.args.is_empty()
                && matches!(call.receiver.as_ref(), syn::Expr::Path(_)))
}

struct WriteOnlyStringRemover<'names> {
    names: &'names HashSet<String>,
}

impl VisitMut for WriteOnlyStringRemover<'_> {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        visit_mut::visit_block_mut(self, block);
        block.stmts.retain(|statement| {
            if let syn::Stmt::Local(local) = statement
                && simple_pattern_name(&local.pat).is_some_and(|name| self.names.contains(&name))
            {
                return false;
            }
            !matches!(statement, syn::Stmt::Expr(syn::Expr::MethodCall(call), _)
                if call.method == "push_str"
                    && matches!(call.receiver.as_ref(), syn::Expr::Path(path)
                        if path.path.get_ident().is_some_and(|name|
                            self.names.contains(&name.to_string()))))
        });
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}
