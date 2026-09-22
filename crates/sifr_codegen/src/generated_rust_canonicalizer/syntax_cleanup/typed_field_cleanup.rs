include!("typed_value_ownership.rs");
include!("typed_standard_contracts.rs");

impl Rewriter<'_> {
    fn clone_is_unambiguous(&self) -> bool {
        !self
            .ambiguous_clone_scopes
            .contains(&self.scope[..self.module_depth].join("::"))
    }

    fn scalar_shadowed(&self, name: &str) -> bool {
        self.scalar_shadows.contains(name)
            || self.scalar_shadows.contains(&format!(
                "opaque:{}::{name}",
                self.scope[..self.module_depth].join("::")
            ))
    }

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
            syn::Expr::MethodCall(call)
                if call.method == "clone"
                    && call.args.is_empty()
                    && self.clone_is_unambiguous() =>
            {
                call.receiver.as_ref()
            }
            expression => expression,
        };
        let syn::Expr::Field(field) = expression else {
            return false;
        };
        matches!(field.base.as_ref(), syn::Expr::Path(path) if path.path.get_ident().is_some())
            && self.field_type(field).is_some_and(|ty| {
                self.standard_named(&ty, "String") && !self.scalar_shadowed("String")
            })
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
            clone_fields: std::collections::HashSet<String>,
            facts: &'a Rewriter<'a>,
        }
        impl<'ast> Visit<'ast> for Counter<'_> {
            fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
                if path.path.is_ident(self.name) {
                    self.count = self.count.saturating_add(1);
                }
                visit::visit_expr_path(self, path);
            }
            fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
                if call.method == "clone"
                    && call.args.is_empty()
                    && let syn::Expr::Field(field) = call.receiver.as_ref()
                    && matches!(field.base.as_ref(), syn::Expr::Path(path) if path.path.is_ident(self.name))
                    && self
                        .facts
                        .field_type(field)
                        .is_some_and(|ty| self.facts.inert_owned_type(&ty))
                {
                    self.clone_fields
                        .insert(field.member.to_token_stream().to_string());
                }
                visit::visit_expr_method_call(self, call);
            }
            fn visit_macro(&mut self, _: &'ast syn::Macro) {
                self.count = usize::MAX;
            }
            fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {
                self.count = usize::MAX;
            }
            fn visit_expr_async(&mut self, _: &'ast syn::ExprAsync) {
                self.count = usize::MAX;
            }
            fn visit_expr_for_loop(&mut self, _: &'ast syn::ExprForLoop) {
                self.count = usize::MAX;
            }
            fn visit_expr_while(&mut self, _: &'ast syn::ExprWhile) {
                self.count = usize::MAX;
            }
            fn visit_expr_loop(&mut self, _: &'ast syn::ExprLoop) {
                self.count = usize::MAX;
            }
            fn visit_block(&mut self, _: &'ast syn::Block) {
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
                // Every root use must be a distinct inert field clone: sibling
                // borrows, repeated fields and captured uses forbid partial moves.
                let mut counter = Counter {
                    name: &name,
                    count: 0,
                    clone_fields: std::collections::HashSet::new(),
                    facts: self.facts,
                };
                counter.visit_stmt(&self.statement);
                if counter.count != counter.clone_fields.len() {
                    return;
                }
                if self
                    .facts
                    .field_type(field)
                    .is_some_and(|ty| self.facts.inert_owned_type(&ty))
                {
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
        if !self.clone_is_unambiguous() {
            return;
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
    struct Shadows(std::collections::HashSet<String>, Vec<String>);
    impl Shadows {
        fn declaration(&mut self, name: &syn::Ident) {
            if STANDARD_VALUE_NAMES.contains(&name.to_string().as_str()) {
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
                    self.imported_name(&rename.ident, &rename.rename, path);
                }
                syn::UseTree::Glob(_) => {
                    self.0
                        .extend(STANDARD_VALUE_NAMES.iter().map(|name| (*name).to_owned()));
                }
            }
        }
        fn imported_name(&mut self, source: &syn::Ident, bound: &syn::Ident, path: &[String]) {
            let mut full = path.to_vec();
            full.push(source.to_string());
            let full = full.join("::");
            let expected = match bound.to_string().as_str() {
                "String" => "std::string::String",
                "SifrInt" => "sifr_runtime::SifrInt",
                "Option" => "std::option::Option",
                "Some" => "std::option::Option::Some",
                "None" => "std::option::Option::None",
                "Result" => "std::result::Result",
                "HashMap" => "std::collections::HashMap",
                "HashSet" => "std::collections::HashSet",
                "Vec" => "std::vec::Vec",
                "vec" => "std::vec",
                "Box" => "std::boxed::Box",
                "Clone" => "std::clone::Clone",
                _ => "",
            };
            if STANDARD_VALUE_NAMES.contains(&bound.to_string().as_str()) && full != expected {
                self.0.insert(bound.to_string());
            }
        }
    }
    impl<'ast> Visit<'ast> for Shadows {
        fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
            self.declaration(&item.sig.ident);
            visit::visit_item_fn(self, item);
        }
        fn visit_item_const(&mut self, item: &'ast syn::ItemConst) {
            self.declaration(&item.ident);
            visit::visit_item_const(self, item);
        }
        fn visit_item_static(&mut self, item: &'ast syn::ItemStatic) {
            self.declaration(&item.ident);
            visit::visit_item_static(self, item);
        }
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
        fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
            self.1.push(item.ident.to_string());
            visit::visit_item_mod(self, item);
            self.1.pop();
        }
        fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
            if let Some(names) = super::standard_macros::task_local_static_names(&item.mac) {
                for name in names {
                    self.declaration(&name);
                }
                return;
            }
            for name in STANDARD_VALUE_NAMES {
                self.0
                    .insert(format!("opaque:{}::{name}", self.1.join("::")));
            }
        }
    }
    let mut shadows = Shadows::default();
    shadows.visit_file(file);
    shadows.0
}

const STANDARD_VALUE_NAMES: &[&str] = &[
    "vec", "String", "SifrInt", "Option", "Some", "None", "Result", "Vec", "Box", "HashMap",
    "HashSet", "Clone", "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8",
    "i16", "i32", "i64", "i128", "isize", "f32", "f64",
];

impl Rewriter<'_> {
    fn rewrite_inert_parent_field_clone(&self, expression: &mut syn::Expr) {
        if !self.clone_is_unambiguous() {
            return;
        }
        let syn::Expr::MethodCall(outer) = expression else {
            return;
        };
        if outer.method != "clone" || !outer.args.is_empty() {
            return;
        }
        let syn::Expr::Field(field) = outer.receiver.as_mut() else {
            return;
        };
        let syn::Expr::MethodCall(parent) = field.base.as_ref() else {
            return;
        };
        if parent.method == "clone"
            && parent.args.is_empty()
            && self
                .ty(&parent.receiver)
                .is_some_and(|ty| self.inert_owned_type(unreference(&ty)))
            && self
                .field_type(field)
                .is_some_and(|ty| self.inert_owned_type(&ty))
        {
            field.base = parent.receiver.clone();
        }
    }
}
