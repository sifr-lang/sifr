//! Import demand differs from conservative support retention: only unqualified
//! references that resolve outside the consumer's lexical scope need an import.
use proc_macro2::{TokenStream, TokenTree};
use std::collections::HashSet;
use syn::parse::Parser;
use syn::visit::{self, Visit};

pub(crate) fn rust_source_unqualified_item_names(
    source: &str,
    candidates: &HashSet<String>,
) -> Result<HashSet<String>, String> {
    let file = syn::parse_file(source)
        .map_err(|error| format!("failed to parse generated import consumer: {error}"))?;
    let mut collector = ImportReferences {
        candidates,
        references: HashSet::new(),
        bindings: vec![Bindings::items(file.items.iter())],
        item_bindings: vec![Bindings::items(file.items.iter())],
        type_path: false,
    };
    collector.visit_file(&file);
    Ok(collector.references)
}

#[derive(Clone, Default)]
struct Bindings {
    values: HashSet<String>,
    types: HashSet<String>,
}

impl Bindings {
    fn items<'a>(items: impl Iterator<Item = &'a syn::Item>) -> Self {
        let mut bindings = Self::default();
        for item in items {
            match item {
                syn::Item::Fn(item) => {
                    bindings.values.insert(item.sig.ident.to_string());
                }
                syn::Item::Const(item) => {
                    bindings.values.insert(item.ident.to_string());
                }
                syn::Item::Static(item) => {
                    bindings.values.insert(item.ident.to_string());
                }
                syn::Item::Struct(item) => {
                    bindings.types.insert(item.ident.to_string());
                    bindings.values.insert(item.ident.to_string());
                }
                syn::Item::Enum(item) => {
                    bindings.types.insert(item.ident.to_string());
                }
                syn::Item::Type(item) => {
                    bindings.types.insert(item.ident.to_string());
                }
                syn::Item::Trait(item) => {
                    bindings.types.insert(item.ident.to_string());
                }
                syn::Item::Union(item) => {
                    bindings.types.insert(item.ident.to_string());
                }
                syn::Item::Mod(item) => {
                    bindings.types.insert(item.ident.to_string());
                }
                syn::Item::Use(item) => bindings.imports(&item.tree),
                _ => {}
            }
        }
        bindings
    }

    fn imports(&mut self, tree: &syn::UseTree) {
        let name = match tree {
            syn::UseTree::Name(name) => Some(&name.ident),
            syn::UseTree::Rename(rename) => Some(&rename.rename),
            syn::UseTree::Path(path) => {
                self.imports(&path.tree);
                None
            }
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    self.imports(item);
                }
                None
            }
            syn::UseTree::Glob(_) => None,
        };
        if let Some(name) = name {
            self.values.insert(name.to_string());
            self.types.insert(name.to_string());
        }
    }

    fn pattern(&mut self, pattern: &syn::Pat) {
        struct Names<'a>(&'a mut HashSet<String>);
        impl<'ast> Visit<'ast> for Names<'_> {
            fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
                self.0.insert(pattern.ident.to_string());
                visit::visit_pat_ident(self, pattern);
            }
        }
        Names(&mut self.values).visit_pat(pattern);
    }

    fn generics(&mut self, generics: &syn::Generics) {
        for parameter in &generics.params {
            match parameter {
                syn::GenericParam::Type(ty) => {
                    self.types.insert(ty.ident.to_string());
                }
                syn::GenericParam::Const(value) => {
                    self.values.insert(value.ident.to_string());
                }
                syn::GenericParam::Lifetime(_) => {}
            }
        }
    }
}

struct ImportReferences<'a> {
    candidates: &'a HashSet<String>,
    references: HashSet<String>,
    bindings: Vec<Bindings>,
    item_bindings: Vec<Bindings>,
    type_path: bool,
}

impl ImportReferences<'_> {
    fn reference(&mut self, name: &str, type_path: bool) {
        let bound = self.bindings.iter().rev().any(|scope| {
            if type_path {
                scope.types.contains(name)
            } else {
                scope.values.contains(name)
            }
        });
        if self.candidates.contains(name) && !bound {
            self.references.insert(name.to_string());
        }
    }

    fn function(&mut self, signature: &syn::Signature, body: Option<&syn::Block>) {
        let mut scope = Bindings::default();
        scope.generics(&signature.generics);
        for input in &signature.inputs {
            if let syn::FnArg::Typed(input) = input {
                scope.pattern(&input.pat);
            }
        }
        self.bindings.push(scope);
        self.visit_signature(signature);
        if let Some(body) = body {
            self.visit_block(body);
        }
        self.bindings.pop();
    }

    fn condition(&mut self, expression: &syn::Expr) {
        match expression {
            syn::Expr::Let(binding) => {
                self.visit_expr(&binding.expr);
                self.visit_pat(&binding.pat);
                let mut scope = Bindings::default();
                scope.pattern(&binding.pat);
                self.bindings.push(scope);
            }
            syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
                self.condition(&binary.left);
                self.condition(&binary.right);
            }
            _ => self.visit_expr(expression),
        }
    }

    fn macro_tokens(&mut self, tokens: TokenStream) {
        let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
        if let Ok(expressions) = parser.parse2(tokens.clone()) {
            for expression in &expressions {
                self.visit_expr(expression);
            }
            return;
        }
        let tokens = tokens.into_iter().collect::<Vec<_>>();
        for (index, token) in tokens.iter().enumerate() {
            if let TokenTree::Group(group) = token {
                self.macro_tokens(group.stream());
            } else if let TokenTree::Ident(name) = token {
                let punct = |index: usize, value| matches!(tokens.get(index), Some(TokenTree::Punct(p)) if p.as_char() == value);
                let member = punct(index.wrapping_sub(1), '.') || punct(index.wrapping_sub(1), ':');
                let field = punct(index + 1, ':') && !punct(index + 2, ':');
                if !member && !field {
                    self.reference(&name.to_string(), punct(index + 1, ':'));
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for ImportReferences<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let generics = match item {
            syn::Item::Struct(item) => Some(&item.generics),
            syn::Item::Enum(item) => Some(&item.generics),
            syn::Item::Trait(item) => Some(&item.generics),
            syn::Item::Type(item) => Some(&item.generics),
            syn::Item::Union(item) => Some(&item.generics),
            _ => None,
        };
        let mut scope = Bindings::default();
        if let Some(generics) = generics {
            scope.generics(generics);
        }
        self.bindings.push(scope);
        visit::visit_item(self, item);
        self.bindings.pop();
    }

    // A nested module has its own import scope; its owner visits it separately.
    fn visit_item_mod(&mut self, _: &'ast syn::ItemMod) {}
    fn visit_item_use(&mut self, _: &'ast syn::ItemUse) {}

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path.leading_colon.is_none()
            && let Some(first) = path.segments.first()
        {
            self.reference(
                &first.ident.to_string(),
                self.type_path || path.segments.len() > 1,
            );
        }
        visit::visit_path(self, path);
    }

    fn visit_type(&mut self, ty: &'ast syn::Type) {
        let previous = self.type_path;
        self.type_path = true;
        visit::visit_type(self, ty);
        self.type_path = previous;
    }

    fn visit_expr(&mut self, expression: &'ast syn::Expr) {
        let previous = self.type_path;
        self.type_path = false;
        visit::visit_expr(self, expression);
        self.type_path = previous;
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        // Item functions cannot capture enclosing locals or generic parameters.
        let enclosing = std::mem::replace(&mut self.bindings, self.item_bindings.clone());
        self.function(&item.sig, Some(&item.block));
        self.bindings = enclosing;
    }
    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.function(&item.sig, Some(&item.block));
    }
    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        self.function(&item.sig, item.default.as_ref());
    }
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let mut scope = Bindings::default();
        scope.generics(&item.generics);
        self.bindings.push(scope);
        visit::visit_item_impl(self, item);
        self.bindings.pop();
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let depth = self.bindings.len();
        let items = Bindings::items(block.stmts.iter().filter_map(|stmt| {
            if let syn::Stmt::Item(item) = stmt {
                Some(item)
            } else {
                None
            }
        }));
        self.item_bindings.push(items.clone());
        self.bindings.push(items);
        for statement in &block.stmts {
            self.visit_stmt(statement);
            if let syn::Stmt::Local(local) = statement {
                let mut scope = Bindings::default();
                scope.pattern(&local.pat);
                self.bindings.push(scope);
            }
        }
        self.bindings.truncate(depth);
        self.item_bindings.pop();
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        let mut scope = Bindings::default();
        for input in &closure.inputs {
            scope.pattern(input);
        }
        self.bindings.push(scope);
        visit::visit_expr_closure(self, closure);
        self.bindings.pop();
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.visit_expr(&expression.expr);
        self.visit_pat(&expression.pat);
        let mut scope = Bindings::default();
        scope.pattern(&expression.pat);
        self.bindings.push(scope);
        self.visit_block(&expression.body);
        self.bindings.pop();
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let (pattern, guard) = match &arm.pat {
            syn::Pat::Guard(guard) => (guard.pat.as_ref(), Some(guard.guard.as_ref())),
            pattern => (pattern, None),
        };
        self.visit_pat(pattern);
        let mut scope = Bindings::default();
        scope.pattern(pattern);
        self.bindings.push(scope);
        if let Some(guard) = guard {
            self.visit_expr(guard);
        }
        self.visit_expr(&arm.body);
        self.bindings.pop();
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        let depth = self.bindings.len();
        self.condition(&expression.cond);
        self.visit_block(&expression.then_branch);
        self.bindings.truncate(depth);
        if let Some((_, otherwise)) = &expression.else_branch {
            self.visit_expr(otherwise);
        }
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        let depth = self.bindings.len();
        self.condition(&expression.cond);
        self.visit_block(&expression.body);
        self.bindings.truncate(depth);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        for name in crate::generated_rust_canonicalizer::generated_format_capture_names(mac) {
            self.reference(&name, false);
        }
        self.visit_path(&mac.path);
        if crate::generated_rust_canonicalizer::is_generated_format_macro(mac)
            && let Ok(arguments) = mac.parse_body_with(
                syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
            )
        {
            // A named formatting argument labels a slot; only its value can
            // demand a support import. Use the shared parser's macro boundary.
            for argument in &arguments {
                if let syn::Expr::Assign(argument) = argument {
                    self.visit_expr(&argument.right);
                } else {
                    self.visit_expr(argument);
                }
            }
            return;
        }
        self.macro_tokens(mac.tokens.clone());
    }
}
