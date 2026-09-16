//! Exclude proven inherent calls from extension-trait imports in a physical scope.
use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};

pub(crate) type InherentMethods = HashSet<(String, String)>;

// Only centralized, compiler-owned stdlib nominal names are globally unambiguous.
// Ordinary user type basenames can identify different types in separate modules.
pub(crate) fn collect_inherent_methods(file: &syn::File, methods: &mut InherentMethods) {
    let owner = crate::canonicalize_generated_rust_identifier("__sifr_project_nominals");
    for item in &file.items {
        let syn::Item::Mod(module) = item else {
            continue;
        };
        if module.ident != owner {
            continue;
        }
        let Some((_, items)) = &module.content else {
            continue;
        };
        for item in items {
            let syn::Item::Impl(implementation) = item else {
                continue;
            };
            let Some(name) = type_name(&implementation.self_ty) else {
                continue;
            };
            if implementation.trait_.is_some() || !name.starts_with("SifrGeneratedStdlib") {
                continue;
            }
            for member in &implementation.items {
                if let syn::ImplItem::Fn(method) = member {
                    methods.insert((name.clone(), method.sig.ident.to_string()));
                }
            }
        }
    }
}

pub(super) struct MethodCallCollector<'a> {
    pub(super) method_traits: &'a HashMap<String, HashSet<String>>,
    pub(super) inherent: &'a InherentMethods,
    pub(super) required_traits: HashSet<String>,
    pub(super) bindings: HashMap<String, Option<String>>,
}

fn type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(path) if path.qself.is_none() => {
            let parts = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>();
            match parts.as_slice() {
                [name] if path.path.leading_colon.is_none() => Some(name.clone()),
                [root, name] if root == "crate" => Some(name.clone()),
                [root, module, name]
                    if root == "crate"
                        && module
                            == &crate::canonicalize_generated_rust_identifier(
                                "__sifr_project_nominals",
                            ) =>
                {
                    Some(name.clone())
                }
                _ => None,
            }
        }
        syn::Type::Reference(reference) => type_name(&reference.elem),
        syn::Type::Paren(paren) => type_name(&paren.elem),
        _ => None,
    }
}

impl MethodCallCollector<'_> {
    fn receiver_type(&self, expression: &syn::Expr) -> Option<String> {
        match expression {
            syn::Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => self
                .bindings
                .get(&path.path.segments[0].ident.to_string())
                .cloned()
                .flatten(),
            syn::Expr::Reference(reference) => self.receiver_type(&reference.expr),
            syn::Expr::Paren(paren) => self.receiver_type(&paren.expr),
            syn::Expr::Group(group) => self.receiver_type(&group.expr),
            _ => None,
        }
    }

    fn bind_pattern(&mut self, pattern: &syn::Pat, ty: Option<String>) {
        struct Names(Vec<String>);
        impl<'ast> Visit<'ast> for Names {
            fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
                self.0.push(pattern.ident.to_string());
                visit::visit_pat_ident(self, pattern);
            }
        }
        let mut names = Names(Vec::new());
        names.visit_pat(pattern);
        for name in names.0 {
            self.bindings.insert(name, None);
        }
        match pattern {
            syn::Pat::Ident(binding) => {
                self.bindings.insert(binding.ident.to_string(), ty);
            }
            syn::Pat::Type(typed) => self.bind_pattern(&typed.pat, type_name(&typed.ty)),
            _ => {}
        }
    }
}

impl<'ast> Visit<'ast> for MethodCallCollector<'_> {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let saved = std::mem::take(&mut self.bindings);
        visit::visit_item_fn(self, item);
        self.bindings = saved;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let saved = std::mem::take(&mut self.bindings);
        visit::visit_impl_item_fn(self, item);
        self.bindings = saved;
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        let saved = std::mem::take(&mut self.bindings);
        visit::visit_trait_item_fn(self, item);
        self.bindings = saved;
    }

    fn visit_fn_arg(&mut self, argument: &'ast syn::FnArg) {
        if let syn::FnArg::Typed(argument) = argument {
            self.bind_pattern(&argument.pat, type_name(&argument.ty));
        }
        visit::visit_fn_arg(self, argument);
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let saved = self.bindings.clone();
        visit::visit_block(self, block);
        self.bindings = saved;
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        // The initializer resolves in the outer scope, before this binding shadows it.
        if let Some(init) = &local.init {
            self.visit_local_init(init);
        }
        self.bind_pattern(&local.pat, None);
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        let saved = self.bindings.clone();
        for input in &closure.inputs {
            self.bind_pattern(input, None);
        }
        self.visit_expr(&closure.body);
        self.bindings = saved;
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let saved = self.bindings.clone();
        self.bind_pattern(&arm.pat, None);
        if let syn::Pat::Guard(guard) = &arm.pat {
            self.visit_expr(&guard.guard);
        }
        self.visit_expr(&arm.body);
        self.bindings = saved;
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.visit_expr(&expression.expr);
        let saved = self.bindings.clone();
        self.bind_pattern(&expression.pat, None);
        self.visit_block(&expression.body);
        self.bindings = saved;
    }

    fn visit_expr_let(&mut self, expression: &'ast syn::ExprLet) {
        self.visit_expr(&expression.expr);
        self.bind_pattern(&expression.pat, None);
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        let saved = self.bindings.clone();
        self.visit_expr(&expression.cond);
        self.visit_block(&expression.then_branch);
        self.bindings = saved;
        if let Some((_, otherwise)) = &expression.else_branch {
            self.visit_expr(otherwise);
        }
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        let saved = self.bindings.clone();
        self.visit_expr(&expression.cond);
        self.visit_block(&expression.body);
        self.bindings = saved;
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in arguments {
                self.visit_expr(&argument);
            }
        }
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let name = call.method.to_string();
        let inherent = self
            .receiver_type(&call.receiver)
            .is_some_and(|owner| self.inherent.contains(&(owner, name.clone())));
        if !inherent && let Some(traits) = self.method_traits.get(&name) {
            self.required_traits.extend(traits.iter().cloned());
        }
        visit::visit_expr_method_call(self, call);
    }
}
