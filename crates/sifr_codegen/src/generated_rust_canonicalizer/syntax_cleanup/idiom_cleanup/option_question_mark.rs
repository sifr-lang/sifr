//! A let-else scrutinee supports match ergonomics; `?` requires an owned Option.
//! Rewrite only with lexical value-type evidence, never from a binding's name.

use std::collections::HashMap;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

use crate::generated_rust_canonicalizer::api_cleanup::const_types::DropTypes;

pub(super) fn rewrite(file: &mut syn::File) {
    Rewriter {
        types: DropTypes::for_items(&file.items),
        bindings: HashMap::new(),
    }
    .visit_file_mut(file);
}

struct Rewriter {
    types: DropTypes,
    bindings: HashMap<String, bool>,
}

impl Rewriter {
    fn bind(&mut self, pattern: &syn::Pat, owned_option: bool) {
        struct Names(Vec<String>);
        impl<'ast> Visit<'ast> for Names {
            fn visit_pat_ident(&mut self, binding: &'ast syn::PatIdent) {
                self.0.push(binding.ident.to_string());
                visit::visit_pat_ident(self, binding);
            }
        }
        let mut names = Names(Vec::new());
        names.visit_pat(pattern);
        let direct = match pattern {
            syn::Pat::Type(typed) => typed.pat.as_ref(),
            pattern => pattern,
        };
        let direct = matches!(direct, syn::Pat::Ident(binding) if binding.by_ref.is_none() && binding.subpat.is_none());
        for name in names.0 {
            self.bindings.insert(name, direct && owned_option);
        }
    }

    fn expression_is_owned_option(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => self
                .bindings
                .get(&path.path.segments[0].ident.to_string())
                .copied()
                .unwrap_or(false),
            syn::Expr::Paren(paren) => self.expression_is_owned_option(&paren.expr),
            syn::Expr::Group(group) => self.expression_is_owned_option(&group.expr),
            _ => false,
        }
    }

    fn function(&mut self, signature: &syn::Signature, body: &mut syn::Block) {
        let previous = std::mem::take(&mut self.bindings);
        for argument in &signature.inputs {
            if let syn::FnArg::Typed(argument) = argument {
                self.bind(&argument.pat, self.types.is_owned_option(&argument.ty));
            }
        }
        self.visit_block_mut(body);
        self.bindings = previous;
    }
}

impl VisitMut for Rewriter {
    fn visit_item_fn_mut(&mut self, function: &mut syn::ItemFn) {
        self.function(&function.sig, &mut function.block);
    }

    fn visit_impl_item_fn_mut(&mut self, function: &mut syn::ImplItemFn) {
        self.function(&function.sig, &mut function.block);
    }

    fn visit_trait_item_fn_mut(&mut self, function: &mut syn::TraitItemFn) {
        if let Some(body) = &mut function.default {
            self.function(&function.sig, body);
        }
    }

    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let previous = self.bindings.clone();
        visit_mut::visit_block_mut(self, block);
        self.bindings = previous;
    }

    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        visit_mut::visit_local_mut(self, local);
        if local
            .init
            .as_ref()
            .is_some_and(|init| self.expression_is_owned_option(&init.expr))
        {
            super::rewrite_option_let_else_with_question_mark(local);
        }
        let owned = match &local.pat {
            syn::Pat::Type(typed) => self.types.is_owned_option(&typed.ty),
            _ => local
                .init
                .as_ref()
                .is_some_and(|init| self.expression_is_owned_option(&init.expr)),
        };
        self.bind(&local.pat, owned);
    }

    fn visit_expr_closure_mut(&mut self, closure: &mut syn::ExprClosure) {
        let previous = std::mem::take(&mut self.bindings);
        for pattern in &closure.inputs {
            let owned =
                matches!(pattern, syn::Pat::Type(typed) if self.types.is_owned_option(&typed.ty));
            self.bind(pattern, owned);
        }
        self.visit_expr_mut(&mut closure.body);
        self.bindings = previous;
    }

    fn visit_arm_mut(&mut self, arm: &mut syn::Arm) {
        let previous = self.bindings.clone();
        self.bind(&arm.pat, false);
        visit_mut::visit_arm_mut(self, arm);
        self.bindings = previous;
    }

    fn visit_expr_for_loop_mut(&mut self, loop_: &mut syn::ExprForLoop) {
        self.visit_expr_mut(&mut loop_.expr);
        let previous = self.bindings.clone();
        self.bind(&loop_.pat, false);
        self.visit_block_mut(&mut loop_.body);
        self.bindings = previous;
    }

    fn visit_expr_let_mut(&mut self, let_: &mut syn::ExprLet) {
        self.visit_expr_mut(&mut let_.expr);
        self.bind(&let_.pat, false);
    }

    fn visit_expr_if_mut(&mut self, if_: &mut syn::ExprIf) {
        let previous = self.bindings.clone();
        self.visit_expr_mut(&mut if_.cond);
        self.visit_block_mut(&mut if_.then_branch);
        self.bindings = previous.clone();
        if let Some((_, branch)) = &mut if_.else_branch {
            self.visit_expr_mut(branch);
        }
        self.bindings = previous;
    }

    fn visit_expr_while_mut(&mut self, while_: &mut syn::ExprWhile) {
        let previous = self.bindings.clone();
        self.visit_expr_mut(&mut while_.cond);
        self.visit_block_mut(&mut while_.body);
        self.bindings = previous;
    }
}
