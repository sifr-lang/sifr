//! Const promotion must prove ownership transfer as well as expression legality.
//! Unknown owned types may have destructors, even when their parameter is unused.

use std::collections::HashSet;

use super::super::member_demand::type_has_trivial_drop;

pub(super) fn function_has_no_implicit_drop(signature: &syn::Signature, body: &syn::Block) -> bool {
    let mut state = DropState::default();
    for argument in &signature.inputs {
        match argument {
            syn::FnArg::Typed(parameter) if !type_has_trivial_drop(&parameter.ty) => {
                if !bind_owned(&parameter.pat, &mut state.owned) {
                    return false;
                }
            }
            syn::FnArg::Receiver(receiver)
                if !matches!(receiver.kind, syn::ReceiverKind::Reference(..)) =>
            {
                state.owned.insert("self".to_string());
            }
            syn::FnArg::Typed(parameter) => collect_bindings(&parameter.pat, &mut state.trivial),
            syn::FnArg::Receiver(_) => {
                state.trivial.insert("self".to_string());
            }
        }
    }
    block_transfers(body, &mut state) && state.owned.is_empty()
}

#[derive(Clone, Default)]
struct DropState {
    owned: HashSet<String>,
    trivial: HashSet<String>,
}

fn collect_bindings(pattern: &syn::Pat, bindings: &mut HashSet<String>) {
    use syn::visit::{self, Visit};
    struct Collector<'a>(&'a mut HashSet<String>);
    impl<'ast> Visit<'ast> for Collector<'_> {
        fn visit_pat_ident(&mut self, binding: &'ast syn::PatIdent) {
            self.0.insert(binding.ident.to_string());
            visit::visit_pat_ident(self, binding);
        }
    }
    Collector(bindings).visit_pat(pattern);
}

fn bind_owned(pattern: &syn::Pat, owned: &mut HashSet<String>) -> bool {
    let syn::Pat::Ident(binding) = pattern else {
        // A wildcard, partial destructure or reference binding does not prove
        // that the whole incoming value is moved instead of being dropped.
        return false;
    };
    binding.by_ref.is_none() && binding.subpat.is_none() && owned.insert(binding.ident.to_string())
}

fn block_transfers(block: &syn::Block, state: &mut DropState) -> bool {
    for (index, statement) in block.stmts.iter().enumerate() {
        match statement {
            syn::Stmt::Local(local) => {
                let Some(init) = &local.init else {
                    return false;
                };
                let (pattern, trivial) = match &local.pat {
                    syn::Pat::Type(typed) => (&*typed.pat, type_has_trivial_drop(&typed.ty)),
                    pattern => (pattern, expression_is_trivial(&init.expr, &state.trivial)),
                };
                let mut bindings = HashSet::new();
                collect_bindings(pattern, &mut bindings);
                // A shadowed value is still alive under a distinct Rust binding.
                // Do not let a use of the new binding discharge the old owner.
                if bindings
                    .iter()
                    .any(|name| state.owned.contains(name) || state.trivial.contains(name))
                    || init.diverge.is_some()
                    || !expression_transfers(&init.expr, state)
                {
                    return false;
                }
                if trivial {
                    state.trivial.extend(bindings);
                } else if !bind_owned(pattern, &mut state.owned) {
                    return false;
                }
            }
            syn::Stmt::Expr(expression, None) if index + 1 == block.stmts.len() => {
                return expression_transfers(expression, state);
            }
            syn::Stmt::Expr(syn::Expr::Return(return_), _) => {
                return return_
                    .expr
                    .as_ref()
                    .is_none_or(|value| expression_transfers(value, state));
            }
            syn::Stmt::Item(_) => {}
            // A discarded expression can itself create a value with Drop.
            // Expression legality alone cannot establish that it is const-safe.
            _ => return false,
        }
    }
    true
}

fn expression_transfers(expression: &syn::Expr, state: &mut DropState) -> bool {
    match expression {
        syn::Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => {
            state.owned.remove(&path.path.segments[0].ident.to_string());
            true
        }
        syn::Expr::Lit(_) | syn::Expr::Path(_) => true,
        syn::Expr::Paren(paren) => expression_transfers(&paren.expr, state),
        syn::Expr::Group(group) => expression_transfers(&group.expr, state),
        syn::Expr::Tuple(tuple) => tuple
            .elems
            .iter()
            .all(|value| expression_transfers(value, state)),
        syn::Expr::Array(array) => array
            .elems
            .iter()
            .all(|value| expression_transfers(value, state)),
        syn::Expr::Struct(struct_) => {
            struct_
                .fields
                .iter()
                .all(|field| expression_transfers(&field.expr, state))
                && struct_
                    .rest
                    .as_ref()
                    .is_none_or(|rest| expression_transfers(rest, state))
        }
        syn::Expr::Call(call) => call
            .args
            .iter()
            .all(|value| expression_transfers(value, state)),
        syn::Expr::Block(block) => block_transfers(&block.block, state),
        syn::Expr::Return(return_) => return_
            .expr
            .as_ref()
            .is_none_or(|value| expression_transfers(value, state)),
        syn::Expr::If(if_) => {
            // The condition is borrowed for its truth value; it cannot discharge
            // ownership. Each complete branch must transfer the same inputs.
            if !matches!(if_.cond.as_ref(), syn::Expr::Path(_))
                && !expression_is_trivial(&if_.cond, &state.trivial)
            {
                return false;
            }
            let mut then_state = state.clone();
            let mut else_state = state.clone();
            if !block_transfers(&if_.then_branch, &mut then_state)
                || !if_
                    .else_branch
                    .as_ref()
                    .is_some_and(|(_, branch)| expression_transfers(branch, &mut else_state))
                || then_state.owned != else_state.owned
            {
                return false;
            }
            state.owned = then_state.owned;
            true
        }
        // Borrows and field reads never transfer an owned parameter. The
        // independent expression checker proves whether these are const-legal.
        syn::Expr::Reference(_) | syn::Expr::Field(_) | syn::Expr::Unary(_) => true,
        _ => false,
    }
}

fn expression_is_trivial(expression: &syn::Expr, trivial: &HashSet<String>) -> bool {
    match expression {
        syn::Expr::Lit(_) => true,
        syn::Expr::Reference(_) => true,
        syn::Expr::Path(path) => {
            path.qself.is_none()
                && path.path.segments.len() == 1
                && trivial.contains(&path.path.segments[0].ident.to_string())
        }
        syn::Expr::Paren(paren) => expression_is_trivial(&paren.expr, trivial),
        syn::Expr::Group(group) => expression_is_trivial(&group.expr, trivial),
        syn::Expr::Tuple(tuple) => tuple
            .elems
            .iter()
            .all(|value| expression_is_trivial(value, trivial)),
        syn::Expr::Array(array) => array
            .elems
            .iter()
            .all(|value| expression_is_trivial(value, trivial)),
        _ => false,
    }
}
