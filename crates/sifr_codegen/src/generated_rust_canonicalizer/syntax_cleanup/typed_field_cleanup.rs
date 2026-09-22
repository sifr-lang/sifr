impl Rewriter<'_> {
    fn discardable_unused_string_field(&self, local: &syn::Local, remaining: &[syn::Stmt]) -> bool {
        if super::identifier_names_in_pattern(&local.pat)
            .iter()
            .any(|name| statements_reference(remaining, name))
        {
            return false;
        }
        let Some(init) = &local.init else {
            return false;
        };
        if init.diverge.is_some() {
            return false;
        }
        let expression = match init.expr.as_ref() {
            syn::Expr::MethodCall(call) if call.method == "clone" && call.args.is_empty() => {
                call.receiver.as_ref()
            }
            expression => expression,
        };
        let syn::Expr::Field(field) = expression else {
            return false;
        };
        matches!(field.base.as_ref(), syn::Expr::Path(path) if path.path.get_ident().is_some())
            && self
                .field_type(field)
                .is_some_and(|ty| named(&ty, "String") && !self.scalar_shadows.contains("String"))
    }

    fn remove_terminal_owned_field_clones(
        &self,
        statement: &mut syn::Stmt,
        remaining: &[syn::Stmt],
        owned: &std::collections::HashSet<String>,
    ) {
        use syn::visit::{self, Visit};
        struct Counter<'a> {
            name: &'a str,
            count: usize,
        }
        impl<'ast> Visit<'ast> for Counter<'_> {
            fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
                if path.path.is_ident(self.name) {
                    self.count = self.count.saturating_add(1);
                }
                visit::visit_expr_path(self, path);
            }
            fn visit_macro(&mut self, _: &'ast syn::Macro) {
                self.count = usize::MAX;
            }
        }
        struct Fields<'a, 'facts> {
            facts: &'a Rewriter<'facts>,
            remaining: &'a [syn::Stmt],
            owned: &'a std::collections::HashSet<String>,
            statement: syn::Stmt,
        }
        impl VisitMut for Fields<'_, '_> {
            fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
                visit_mut::visit_expr_mut(self, expression);
                let syn::Expr::MethodCall(clone) = expression else {
                    return;
                };
                if clone.method != "clone" || !clone.args.is_empty() {
                    return;
                }
                let syn::Expr::Field(field) = clone.receiver.as_ref() else {
                    return;
                };
                let syn::Expr::Path(path) = field.base.as_ref() else {
                    return;
                };
                let Some(name) = path.path.get_ident().map(ToString::to_string) else {
                    return;
                };
                if !self.owned.contains(&name) || statements_reference(self.remaining, &name) {
                    return;
                }
                // One occurrence forbids sibling borrows, and only a stored
                // String or exact integer can use the standard owned Clone.
                let mut counter = Counter {
                    name: &name,
                    count: 0,
                };
                counter.visit_stmt(&self.statement);
                if counter.count != 1 {
                    return;
                }
                if self.facts.field_type(field).is_some_and(|ty| {
                    (named(&ty, "String") && !self.facts.scalar_shadows.contains("String"))
                        || (named(&ty, "SifrInt") && !self.facts.scalar_shadows.contains("SifrInt"))
                }) {
                    *expression = clone.receiver.as_ref().clone();
                }
            }
            fn visit_expr_closure_mut(&mut self, _: &mut syn::ExprClosure) {}
            fn visit_expr_async_mut(&mut self, _: &mut syn::ExprAsync) {}
            fn visit_expr_for_loop_mut(&mut self, _: &mut syn::ExprForLoop) {}
            fn visit_expr_while_mut(&mut self, _: &mut syn::ExprWhile) {}
            fn visit_expr_loop_mut(&mut self, _: &mut syn::ExprLoop) {}
            fn visit_block_mut(&mut self, _: &mut syn::Block) {}
            fn visit_item_mut(&mut self, _: &mut syn::Item) {}
        }
        let original = statement.clone();
        Fields {
            facts: self,
            remaining,
            owned,
            statement: original,
        }
        .visit_stmt_mut(statement);
    }
}

fn collect_scalar_shadows(file: &syn::File) -> std::collections::HashSet<String> {
    use syn::visit::{self, Visit};
    #[derive(Default)]
    struct Shadows(std::collections::HashSet<String>);
    impl Shadows {
        fn declaration(&mut self, name: &syn::Ident) {
            if matches!(name.to_string().as_str(), "String" | "SifrInt") {
                self.0.insert(name.to_string());
            }
        }
        fn import(&mut self, tree: &syn::UseTree, path: &mut Vec<String>) {
            match tree {
                syn::UseTree::Path(part) => {
                    path.push(part.ident.to_string());
                    self.import(&part.tree, path);
                    path.pop();
                }
                syn::UseTree::Group(group) => {
                    for item in &group.items {
                        self.import(item, path);
                    }
                }
                syn::UseTree::Name(name) => self.imported_name(&name.ident, &name.ident, path),
                syn::UseTree::Rename(rename) => {
                    self.imported_name(&rename.ident, &rename.rename, path)
                }
                syn::UseTree::Glob(_) => {
                    self.0.extend(["String".to_owned(), "SifrInt".to_owned()]);
                }
            }
        }
        fn imported_name(&mut self, source: &syn::Ident, bound: &syn::Ident, path: &[String]) {
            let mut full = path.to_vec();
            full.push(source.to_string());
            let full = full.join("::");
            if (bound == "String" && full != "std::string::String")
                || (bound == "SifrInt" && full != "sifr_runtime::SifrInt")
            {
                self.0.insert(bound.to_string());
            }
        }
    }
    impl<'ast> Visit<'ast> for Shadows {
        fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
            self.declaration(&item.ident);
            visit::visit_item_struct(self, item);
        }
        fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
            self.declaration(&item.ident);
            visit::visit_item_enum(self, item);
        }
        fn visit_item_union(&mut self, item: &'ast syn::ItemUnion) {
            self.declaration(&item.ident);
            visit::visit_item_union(self, item);
        }
        fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
            self.declaration(&item.ident);
            visit::visit_item_type(self, item);
        }
        fn visit_type_param(&mut self, item: &'ast syn::TypeParam) {
            self.declaration(&item.ident);
            visit::visit_type_param(self, item);
        }
        fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
            self.import(&item.tree, &mut Vec::new());
        }
        fn visit_item_macro(&mut self, _: &'ast syn::ItemMacro) {
            self.0.extend(["String".to_owned(), "SifrInt".to_owned()]);
        }
    }
    let mut shadows = Shadows::default();
    shadows.visit_file(file);
    shadows.0
}
