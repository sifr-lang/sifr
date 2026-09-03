use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

pub(super) fn canonicalize_typed_fallbacks(file: &mut syn::File) {
    TypedFallbackFunctionCleanup.visit_file_mut(file);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FallbackReceiverKind {
    Option,
    Result,
}

struct TypedFallbackFunctionCleanup;

impl TypedFallbackFunctionCleanup {
    fn rewrite_body(signature: &syn::Signature, block: &mut syn::Block) {
        let mut bindings = FallbackBindingCollector {
            kinds: HashMap::new(),
            ambiguous: HashSet::new(),
        };
        for argument in &signature.inputs {
            if let syn::FnArg::Typed(argument) = argument {
                bindings.collect_pattern_type(&argument.pat, &argument.ty);
            }
        }
        bindings.visit_block(block);
        TypedFallbackRewriter {
            binding_kinds: &bindings.kinds,
        }
        .visit_block_mut(block);
    }
}

impl VisitMut for TypedFallbackFunctionCleanup {
    fn visit_item_fn_mut(&mut self, function: &mut syn::ItemFn) {
        Self::rewrite_body(&function.sig, &mut function.block);
        visit_mut::visit_item_fn_mut(self, function);
    }

    fn visit_item_impl_mut(&mut self, impl_: &mut syn::ItemImpl) {
        for item in &mut impl_.items {
            if let syn::ImplItem::Fn(method) = item {
                Self::rewrite_body(&method.sig, &mut method.block);
            }
        }
        visit_mut::visit_item_impl_mut(self, impl_);
    }
}

struct FallbackBindingCollector {
    kinds: HashMap<String, FallbackReceiverKind>,
    ambiguous: HashSet<String>,
}

impl FallbackBindingCollector {
    fn collect_pattern_type(&mut self, pattern: &syn::Pat, ty: &syn::Type) {
        let syn::Pat::Ident(binding) = pattern else {
            return;
        };
        let Some(kind) = fallback_receiver_kind(ty) else {
            return;
        };
        let name = binding.ident.to_string();
        if self.ambiguous.contains(&name) {
            return;
        }
        if self
            .kinds
            .get(&name)
            .is_some_and(|existing| *existing != kind)
        {
            self.kinds.remove(&name);
            self.ambiguous.insert(name);
        } else {
            self.kinds.insert(name, kind);
        }
    }
}

impl<'ast> Visit<'ast> for FallbackBindingCollector {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let syn::Pat::Type(typed) = &local.pat {
            self.collect_pattern_type(&typed.pat, &typed.ty);
        }
        visit::visit_local(self, local);
    }
}

struct TypedFallbackRewriter<'bindings> {
    binding_kinds: &'bindings HashMap<String, FallbackReceiverKind>,
}

impl TypedFallbackRewriter<'_> {
    fn receiver_kind(&self, expression: &syn::Expr) -> Option<FallbackReceiverKind> {
        match expression {
            syn::Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => self
                .binding_kinds
                .get(&path.path.segments[0].ident.to_string())
                .copied(),
            syn::Expr::MethodCall(call) if call.method == "clone" && call.args.is_empty() => {
                self.receiver_kind(&call.receiver)
            }
            syn::Expr::Paren(paren) => self.receiver_kind(&paren.expr),
            syn::Expr::Reference(reference) => self.receiver_kind(&reference.expr),
            _ => None,
        }
    }
}

impl VisitMut for TypedFallbackRewriter<'_> {
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
        let syn::Expr::MethodCall(call) = expression else {
            return;
        };
        if call.method != "unwrap_or"
            || call.args.len() != 1
            || self.receiver_kind(&call.receiver) != Some(FallbackReceiverKind::Option)
        {
            return;
        }
        let Some(default @ syn::Expr::Call(_)) = call.args.first() else {
            return;
        };
        if !pure_fallback_call(default) {
            return;
        }
        let default = default.clone();
        call.method = syn::Ident::new("unwrap_or_else", call.method.span());
        call.args.clear();
        call.args.push(syn::parse_quote!(|| #default));
    }
}

fn fallback_receiver_kind(ty: &syn::Type) -> Option<FallbackReceiverKind> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    match path.path.segments.last()?.ident.to_string().as_str() {
        "Option" => Some(FallbackReceiverKind::Option),
        "Result" => Some(FallbackReceiverKind::Result),
        _ => None,
    }
}

fn pure_fallback_call(expression: &syn::Expr) -> bool {
    let syn::Expr::Call(call) = expression else {
        return false;
    };
    matches!(call.func.as_ref(), syn::Expr::Path(path)
        if path.path.segments.last().is_some_and(|segment|
            matches!(segment.ident.to_string().as_str(), "from" | "from_i64")))
        && call.args.iter().all(pure_fallback_argument)
}

fn pure_fallback_argument(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Lit(_) | syn::Expr::Path(_) => true,
        syn::Expr::MethodCall(call) if call.args.is_empty() && call.method == "len" => {
            pure_fallback_argument(&call.receiver)
        }
        syn::Expr::Paren(paren) => pure_fallback_argument(&paren.expr),
        syn::Expr::Reference(reference) => pure_fallback_argument(&reference.expr),
        _ => false,
    }
}
