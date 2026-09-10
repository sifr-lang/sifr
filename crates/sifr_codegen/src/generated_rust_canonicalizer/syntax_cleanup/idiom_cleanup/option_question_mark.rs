//! A let-else scrutinee supports match ergonomics; `?` requires an owned Option.
//! Rewrite only with lexical value-type evidence, never from a binding's name.

use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

use crate::generated_rust_canonicalizer::api_cleanup::const_types::DropTypes;

pub(super) fn rewrite(file: &mut syn::File) {
    let mut ambiguous = AmbiguousViews::default();
    ambiguous.visit_file(file);
    Rewriter {
        types: DropTypes::for_items(&file.items),
        bindings: HashMap::new(),
        ambiguous_views: ambiguous.0,
    }
    .visit_file_mut(file);
}

struct Rewriter {
    types: DropTypes,
    bindings: HashMap<String, OptionKind>,
    ambiguous_views: HashSet<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum OptionKind {
    Owned,
    Borrowed,
    Unknown,
}

#[derive(Default)]
struct AmbiguousViews(HashSet<String>);

impl<'ast> Visit<'ast> for AmbiguousViews {
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        // An opaque imported extension trait may define a competing view method.
        // Closed standard-library imports cannot add an external impl for Option.
        if !matches!(&item.tree, syn::UseTree::Path(path) if matches!(path.ident.to_string().as_str(), "std" | "core"))
        {
            self.0.extend(["as_ref".to_string(), "as_mut".to_string()]);
        }
        visit::visit_item_use(self, item);
    }
    fn visit_signature(&mut self, signature: &'ast syn::Signature) {
        if matches!(signature.ident.to_string().as_str(), "as_ref" | "as_mut") {
            self.0.insert(signature.ident.to_string());
        }
        visit::visit_signature(self, signature);
    }
    fn visit_use_glob(&mut self, _: &'ast syn::UseGlob) {
        self.0.extend(["as_ref".to_string(), "as_mut".to_string()]);
    }
}

impl Rewriter {
    fn bind(&mut self, pattern: &syn::Pat, kind: OptionKind) {
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
            self.bindings
                .insert(name, if direct { kind } else { OptionKind::Unknown });
        }
    }

    fn type_kind(&self, ty: &syn::Type) -> OptionKind {
        if self.types.is_owned_option(ty) {
            return OptionKind::Owned;
        }
        match ty {
            syn::Type::Reference(reference)
                if self.type_kind(&reference.elem) != OptionKind::Unknown =>
            {
                OptionKind::Borrowed
            }
            syn::Type::Group(group) => self.type_kind(&group.elem),
            syn::Type::Paren(paren) => self.type_kind(&paren.elem),
            _ => OptionKind::Unknown,
        }
    }

    fn expression_kind(&self, expression: &syn::Expr) -> OptionKind {
        match expression {
            syn::Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => self
                .bindings
                .get(&path.path.segments[0].ident.to_string())
                .copied()
                .unwrap_or(OptionKind::Unknown),
            syn::Expr::Paren(paren) => self.expression_kind(&paren.expr),
            syn::Expr::Group(group) => self.expression_kind(&group.expr),
            syn::Expr::Reference(reference)
                if self.expression_kind(&reference.expr) != OptionKind::Unknown =>
            {
                OptionKind::Borrowed
            }
            // Standard Option views return owned Option<&T>/Option<&mut T>,
            // unlike borrowing the Option itself. Reject competing method names.
            syn::Expr::MethodCall(call)
                if call.args.is_empty()
                    && matches!(call.method.to_string().as_str(), "as_ref" | "as_mut")
                    && !self.ambiguous_views.contains(&call.method.to_string())
                    && self.expression_kind(&call.receiver) != OptionKind::Unknown =>
            {
                OptionKind::Owned
            }
            _ => OptionKind::Unknown,
        }
    }

    fn function(&mut self, signature: &syn::Signature, body: &mut syn::Block) {
        let previous = std::mem::take(&mut self.bindings);
        for argument in &signature.inputs {
            if let syn::FnArg::Typed(argument) = argument {
                self.bind(&argument.pat, self.type_kind(&argument.ty));
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
            .is_some_and(|init| self.expression_kind(&init.expr) == OptionKind::Owned)
        {
            super::rewrite_option_let_else_with_question_mark(local);
        }
        let owned = match &local.pat {
            syn::Pat::Type(typed) => self.type_kind(&typed.ty),
            _ => local
                .init
                .as_ref()
                .map_or(OptionKind::Unknown, |init| self.expression_kind(&init.expr)),
        };
        self.bind(&local.pat, owned);
    }

    fn visit_expr_closure_mut(&mut self, closure: &mut syn::ExprClosure) {
        let previous = std::mem::take(&mut self.bindings);
        for pattern in &closure.inputs {
            let owned = match pattern {
                syn::Pat::Type(typed) => self.type_kind(&typed.ty),
                _ => OptionKind::Unknown,
            };
            self.bind(pattern, owned);
        }
        self.visit_expr_mut(&mut closure.body);
        self.bindings = previous;
    }

    fn visit_arm_mut(&mut self, arm: &mut syn::Arm) {
        let previous = self.bindings.clone();
        self.bind(&arm.pat, OptionKind::Unknown);
        visit_mut::visit_arm_mut(self, arm);
        self.bindings = previous;
    }

    fn visit_expr_for_loop_mut(&mut self, loop_: &mut syn::ExprForLoop) {
        self.visit_expr_mut(&mut loop_.expr);
        let previous = self.bindings.clone();
        self.bind(&loop_.pat, OptionKind::Unknown);
        self.visit_block_mut(&mut loop_.body);
        self.bindings = previous;
    }

    fn visit_expr_let_mut(&mut self, let_: &mut syn::ExprLet) {
        self.visit_expr_mut(&mut let_.expr);
        self.bind(&let_.pat, OptionKind::Unknown);
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
