use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

pub(super) fn remove_known_sifr_int_identity_conversions(file: &mut syn::File) {
    let mut methods = SifrIntMethodCollector {
        methods: HashSet::new(),
    };
    methods.visit_file(file);
    if methods.methods.is_empty() {
        return;
    }
    let mut cleanup = FunctionIdentityConversionCleanup {
        methods: &methods.methods,
    };
    cleanup.visit_file_mut(file);
}

struct SifrIntMethodCollector {
    methods: HashSet<(String, String)>,
}

impl<'ast> Visit<'ast> for SifrIntMethodCollector {
    fn visit_item_impl(&mut self, impl_: &'ast syn::ItemImpl) {
        let Some(owner) = named_type(&impl_.self_ty) else {
            visit::visit_item_impl(self, impl_);
            return;
        };
        for item in &impl_.items {
            let syn::ImplItem::Fn(method) = item else {
                continue;
            };
            if return_type_name(&method.sig).as_deref() == Some("SifrInt") {
                self.methods
                    .insert((owner.clone(), method.sig.ident.to_string()));
            }
        }
        visit::visit_item_impl(self, impl_);
    }
}

struct FunctionIdentityConversionCleanup<'methods> {
    methods: &'methods HashSet<(String, String)>,
}

impl FunctionIdentityConversionCleanup<'_> {
    fn rewrite_body(
        &self,
        signature: &syn::Signature,
        block: &mut syn::Block,
        self_owner: Option<&str>,
    ) {
        let mut bindings = TypedBindingCollector {
            owners: HashMap::new(),
            ambiguous: HashSet::new(),
        };
        for argument in &signature.inputs {
            if let syn::FnArg::Typed(argument) = argument {
                bindings.collect_pattern_type(&argument.pat, &argument.ty);
            }
        }
        bindings.visit_block(block);
        KnownSifrIntIdentityRewriter {
            methods: self.methods,
            binding_owners: &bindings.owners,
            self_owner,
        }
        .visit_block_mut(block);
    }
}

impl VisitMut for FunctionIdentityConversionCleanup<'_> {
    fn visit_item_fn_mut(&mut self, function: &mut syn::ItemFn) {
        self.rewrite_body(&function.sig, &mut function.block, None);
        visit_mut::visit_item_fn_mut(self, function);
    }

    fn visit_item_impl_mut(&mut self, impl_: &mut syn::ItemImpl) {
        let owner = named_type(&impl_.self_ty);
        for item in &mut impl_.items {
            if let syn::ImplItem::Fn(method) = item {
                self.rewrite_body(&method.sig, &mut method.block, owner.as_deref());
            }
        }
        visit_mut::visit_item_impl_mut(self, impl_);
    }
}

struct TypedBindingCollector {
    owners: HashMap<String, String>,
    ambiguous: HashSet<String>,
}

impl TypedBindingCollector {
    fn collect_pattern_type(&mut self, pattern: &syn::Pat, ty: &syn::Type) {
        let syn::Pat::Ident(binding) = pattern else {
            return;
        };
        let Some(owner) = named_type(ty) else {
            return;
        };
        let name = binding.ident.to_string();
        if self.ambiguous.contains(&name) {
            return;
        }
        if self
            .owners
            .get(&name)
            .is_some_and(|existing| existing != &owner)
        {
            self.owners.remove(&name);
            self.ambiguous.insert(name);
        } else {
            self.owners.insert(name, owner);
        }
    }
}

impl<'ast> Visit<'ast> for TypedBindingCollector {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let syn::Pat::Type(typed) = &local.pat {
            self.collect_pattern_type(&typed.pat, &typed.ty);
        }
        visit::visit_local(self, local);
    }
}

struct KnownSifrIntIdentityRewriter<'scope> {
    methods: &'scope HashSet<(String, String)>,
    binding_owners: &'scope HashMap<String, String>,
    self_owner: Option<&'scope str>,
}

impl VisitMut for KnownSifrIntIdentityRewriter<'_> {
    fn visit_macro_mut(&mut self, rust_macro: &mut syn::Macro) {
        let Ok(mut arguments) =
            rust_macro.parse_body_with(Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated)
        else {
            return;
        };
        for argument in &mut arguments {
            self.visit_expr_mut(argument);
        }
        rust_macro.tokens = quote!(#arguments);
    }

    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        let syn::Expr::Call(conversion) = expression else {
            return;
        };
        if conversion.args.len() != 1
            || !matches!(conversion.func.as_ref(), syn::Expr::Path(path)
                if path.path.segments.len() == 2
                    && path.path.segments[0].ident == "SifrInt"
                    && path.path.segments[1].ident == "from")
        {
            return;
        }
        let Some(syn::Expr::MethodCall(method)) = conversion.args.first() else {
            return;
        };
        let owner = match method.receiver.as_ref() {
            syn::Expr::Path(path) if path.path.is_ident("self") => self.self_owner,
            syn::Expr::Path(path) if path.path.segments.len() == 1 => self
                .binding_owners
                .get(&path.path.segments[0].ident.to_string())
                .map(String::as_str),
            _ => None,
        };
        let is_identity = owner.is_some_and(|owner| {
            self.methods
                .contains(&(owner.to_string(), method.method.to_string()))
        });
        if is_identity && let Some(replacement) = conversion.args.first().cloned() {
            *expression = replacement;
        }
    }
}

fn named_type(ty: &syn::Type) -> Option<String> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    path.path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

fn return_type_name(signature: &syn::Signature) -> Option<String> {
    let syn::ReturnType::Type(_, ty) = &signature.output else {
        return None;
    };
    named_type(ty)
}
