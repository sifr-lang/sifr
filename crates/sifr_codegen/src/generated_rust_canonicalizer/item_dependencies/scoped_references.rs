use proc_macro2::{TokenStream, TokenTree};
use std::collections::HashSet;
use syn::parse::Parser;
use syn::visit::{self, Visit};

pub(super) fn item_references(item: &syn::Item, definitions: &HashSet<String>) -> HashSet<String> {
    let mut collector = ScopedReferences {
        definitions,
        bindings: HashSet::new(),
        type_bindings: HashSet::new(),
        references: HashSet::new(),
    };
    collector.visit_item(item);
    collector.references
}

struct ScopedReferences<'scope> {
    definitions: &'scope HashSet<String>,
    bindings: HashSet<String>,
    type_bindings: HashSet<String>,
    references: HashSet<String>,
}

impl ScopedReferences<'_> {
    fn path_reference(&mut self, path: &syn::Path, type_namespace: bool) {
        let mut segments = path.segments.iter();
        if let Some(first) = segments.next() {
            let qualified = matches!(first.ident.to_string().as_str(), "crate" | "self" | "super");
            let candidate = if qualified {
                segments.next()
            } else {
                Some(first)
            };
            if let Some(candidate) = candidate {
                let name = candidate.ident.to_string();
                let bindings = if type_namespace {
                    &self.type_bindings
                } else {
                    &self.bindings
                };
                if self.definitions.contains(&name) && (qualified || !bindings.contains(&name)) {
                    self.references.insert(name);
                }
            }
        }
    }

    fn reference(&mut self, name: String, qualified: bool) {
        if self.definitions.contains(&name) && (qualified || !self.bindings.contains(&name)) {
            self.references.insert(name);
        }
    }

    fn bind(&mut self, pattern: &syn::Pat) {
        let mut bindings = PatternBindings::default();
        bindings.visit_pat(pattern);
        self.bindings.extend(bindings.names);
    }

    fn signature_bindings(&mut self, signature: &syn::Signature) {
        for parameter in &signature.generics.params {
            match parameter {
                syn::GenericParam::Type(parameter) => {
                    self.type_bindings.insert(parameter.ident.to_string());
                }
                syn::GenericParam::Const(parameter) => {
                    self.bindings.insert(parameter.ident.to_string());
                }
                syn::GenericParam::Lifetime(_) => {}
            }
        }
        for argument in &signature.inputs {
            if let syn::FnArg::Typed(argument) = argument {
                self.bind(&argument.pat);
            }
        }
    }

    // A let-chain introduces bindings only for later operands and the success body.
    fn condition(&mut self, expression: &syn::Expr) {
        match expression {
            syn::Expr::Let(expression) => {
                self.visit_expr(&expression.expr);
                self.visit_pat(&expression.pat);
                self.bind(&expression.pat);
            }
            syn::Expr::Binary(expression) if matches!(expression.op, syn::BinOp::And(_)) => {
                self.condition(&expression.left);
                self.condition(&expression.right);
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
                continue;
            }
            let TokenTree::Ident(identifier) = token else {
                continue;
            };
            let preceded_by_member_access = matches!(tokens.get(index.wrapping_sub(1)), Some(TokenTree::Punct(punctuation)) if punctuation.as_char() == '.');
            let followed_by_field_separator = matches!(tokens.get(index + 1), Some(TokenTree::Punct(first)) if first.as_char() == ':')
                && !matches!(tokens.get(index + 2), Some(TokenTree::Punct(second)) if second.as_char() == ':');
            if !preceded_by_member_access && !followed_by_field_separator {
                self.reference(identifier.to_string(), false);
            }
        }
    }
}

impl<'ast> Visit<'ast> for ScopedReferences<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.path_reference(path, path.segments.len() > 1);
        visit::visit_path(self, path);
    }

    fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
        self.path_reference(&ty.path, true);
        if let Some(qself) = &ty.qself {
            self.visit_qself(qself);
        }
        for segment in &ty.path.segments {
            self.visit_path_arguments(&segment.arguments);
        }
    }

    fn visit_expr_struct(&mut self, expression: &'ast syn::ExprStruct) {
        self.path_reference(&expression.path, true);
        if let Some(qself) = &expression.qself {
            self.visit_qself(qself);
        }
        for segment in &expression.path.segments {
            self.visit_path_arguments(&segment.arguments);
        }
        for field in &expression.fields {
            self.visit_field_value(field);
        }
        if let Some(rest) = &expression.rest {
            self.visit_expr(rest);
        }
    }

    fn visit_trait_bound(&mut self, bound: &'ast syn::TraitBound) {
        self.path_reference(&bound.path, true);
        for segment in &bound.path.segments {
            self.visit_path_arguments(&segment.arguments);
        }
    }

    fn visit_item_use(&mut self, _: &'ast syn::ItemUse) {}

    fn visit_item(&mut self, item: &'ast syn::Item) {
        let saved = self.bindings.clone();
        let saved_types = self.type_bindings.clone();
        visit::visit_item(self, item);
        self.bindings = saved;
        self.type_bindings = saved_types;
    }

    fn visit_generics(&mut self, generics: &'ast syn::Generics) {
        for parameter in &generics.params {
            match parameter {
                syn::GenericParam::Type(parameter) => {
                    self.type_bindings.insert(parameter.ident.to_string());
                }
                syn::GenericParam::Const(parameter) => {
                    self.bindings.insert(parameter.ident.to_string());
                }
                syn::GenericParam::Lifetime(_) => {}
            }
        }
        visit::visit_generics(self, generics);
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        let saved = std::mem::take(&mut self.bindings);
        let saved_types = std::mem::take(&mut self.type_bindings);
        self.signature_bindings(&function.sig);
        visit::visit_item_fn(self, function);
        self.bindings = saved;
        self.type_bindings = saved_types;
    }

    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        let saved = self.bindings.clone();
        let saved_types = self.type_bindings.clone();
        self.signature_bindings(&function.sig);
        visit::visit_impl_item_fn(self, function);
        self.bindings = saved;
        self.type_bindings = saved_types;
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let saved = self.bindings.clone();
        let saved_types = self.type_bindings.clone();
        // Block-local items are visible throughout their block, unlike let bindings.
        for statement in &block.stmts {
            if let syn::Stmt::Item(item) = statement {
                if let Some(name) = super::item_definition_name(item) {
                    match item {
                        syn::Item::Struct(item) => {
                            self.type_bindings.insert(name.clone());
                            if !matches!(item.fields, syn::Fields::Named(_)) {
                                self.bindings.insert(name);
                            }
                        }
                        syn::Item::Enum(_)
                        | syn::Item::Trait(_)
                        | syn::Item::Type(_)
                        | syn::Item::Union(_) => {
                            self.type_bindings.insert(name);
                        }
                        syn::Item::Impl(_) => {}
                        _ => {
                            self.bindings.insert(name);
                        }
                    }
                } else if let syn::Item::Use(item) = item {
                    let mut names = std::collections::BTreeSet::new();
                    super::super::collect_use_bindings(&item.tree, &mut names);
                    self.bindings.extend(names.iter().cloned());
                    self.type_bindings.extend(names);
                }
            }
        }
        visit::visit_block(self, block);
        self.bindings = saved;
        self.type_bindings = saved_types;
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        // The initializer and let-else diverging arm see the previous binding.
        visit::visit_local(self, local);
        self.bind(&local.pat);
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        let saved = self.bindings.clone();
        for pattern in &closure.inputs {
            self.bind(pattern);
        }
        visit::visit_expr_closure(self, closure);
        self.bindings = saved;
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.visit_expr(&expression.expr);
        self.visit_pat(&expression.pat);
        let saved = self.bindings.clone();
        self.bind(&expression.pat);
        self.visit_block(&expression.body);
        self.bindings = saved;
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let (pattern, guard) = match &arm.pat {
            syn::Pat::Guard(guard) => (guard.pat.as_ref(), Some(&guard.guard)),
            pattern => (pattern, None),
        };
        self.visit_pat(pattern);
        let saved = self.bindings.clone();
        self.bind(pattern);
        if let Some(guard) = guard {
            self.condition(guard);
        }
        self.visit_expr(&arm.body);
        self.bindings = saved;
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        let saved = self.bindings.clone();
        self.condition(&expression.cond);
        self.visit_block(&expression.then_branch);
        self.bindings = saved;
        if let Some((_, branch)) = &expression.else_branch {
            self.visit_expr(branch);
        }
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        let saved = self.bindings.clone();
        self.condition(&expression.cond);
        self.visit_block(&expression.body);
        self.bindings = saved;
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        for name in super::super::format_capture::names(rust_macro) {
            self.reference(name, false);
        }
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                if let syn::Expr::Assign(argument) = argument
                    && super::super::format_capture::is_format_macro(rust_macro)
                {
                    self.visit_expr(&argument.right);
                } else {
                    self.visit_expr(argument);
                }
            }
        } else {
            self.macro_tokens(rust_macro.tokens.clone());
        }
        visit::visit_macro(self, rust_macro);
    }
}

#[derive(Default)]
struct PatternBindings {
    names: HashSet<String>,
}

impl<'ast> Visit<'ast> for PatternBindings {
    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        self.names.insert(pattern.ident.to_string());
        visit::visit_pat_ident(self, pattern);
    }
}
