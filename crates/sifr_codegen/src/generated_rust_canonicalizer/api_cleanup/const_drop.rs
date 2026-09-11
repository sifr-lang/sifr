//! Const promotion must prove ownership transfer as well as expression legality.
//! Unknown owned types may have destructors, even when their parameter is unused.

use std::collections::HashSet;

use super::const_types::DropTypes;

pub(super) fn function_has_no_implicit_drop(
    signature: &syn::Signature,
    body: &syn::Block,
    types: &DropTypes,
) -> bool {
    let mut state = DropState {
        types: types.clone(),
        ..DropState::default()
    };
    for argument in &signature.inputs {
        match argument {
            syn::FnArg::Typed(parameter) if !types.is_trivial(&parameter.ty) => {
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
    types: DropTypes,
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
    block_flow(block, state, ValueUse::Used).is_some()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Flow {
    Continues,
    Returned,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ValueUse {
    Used,
    Discarded,
}

fn block_flow(block: &syn::Block, state: &mut DropState, tail_use: ValueUse) -> Option<Flow> {
    let outer_trivial = state.trivial.clone();
    let mut locals = HashSet::new();
    for (index, statement) in block.stmts.iter().enumerate() {
        let flow = match statement {
            syn::Stmt::Local(local) => {
                let init = local.init.as_ref()?;
                let (pattern, trivial) = match &local.pat {
                    syn::Pat::Type(typed) => (&*typed.pat, state.types.is_trivial(&typed.ty)),
                    pattern => (pattern, expression_is_trivial(&init.expr, &state.trivial)),
                };
                let mut bindings = HashSet::new();
                collect_bindings(pattern, &mut bindings);
                // A shadowed value is still alive under a distinct Rust binding.
                // Do not let a use of the new binding discharge the old owner.
                if bindings.iter().any(|name| state.owned.contains(name)) {
                    return None;
                }
                if expression_flow(&init.expr, state, ValueUse::Used)? == Flow::Returned {
                    state.trivial = outer_trivial;
                    return Some(Flow::Returned);
                }
                if let Some((_, diverge)) = &init.diverge {
                    // Refutable destructuring is safe only for a proven trivial
                    // value. The nonmatching path must return without leaving
                    // any other owner behind; it does not consume continuing owners.
                    if !trivial {
                        return None;
                    }
                    let mut failed_match = state.clone();
                    if expression_flow(diverge, &mut failed_match, ValueUse::Used)?
                        != Flow::Returned
                    {
                        return None;
                    }
                }
                locals.extend(bindings.iter().cloned());
                // A trivial shadow can die freely, but its new binding must not
                // inherit triviality when the initializer instead owns a value.
                state.trivial.retain(|name| !bindings.contains(name));
                if trivial {
                    state.trivial.extend(bindings);
                } else if !bind_owned(pattern, &mut state.owned) {
                    return None;
                }
                Flow::Continues
            }
            syn::Stmt::Expr(expression, None) if index + 1 == block.stmts.len() => {
                expression_flow(expression, state, tail_use)?
            }
            syn::Stmt::Expr(expression, _) => {
                expression_flow(expression, state, ValueUse::Discarded)?
            }
            syn::Stmt::Item(_) => Flow::Continues,
            syn::Stmt::Macro(_) => return None,
        };
        if flow == Flow::Returned {
            state.trivial = outer_trivial;
            return Some(flow);
        }
    }
    // Locals leave this lexical scope even when the enclosing function proceeds.
    if !state.owned.is_disjoint(&locals) {
        return None;
    }
    state.trivial = outer_trivial;
    Some(Flow::Continues)
}

fn expression_flow(
    expression: &syn::Expr,
    state: &mut DropState,
    value_use: ValueUse,
) -> Option<Flow> {
    // Control-flow branches carry their own value-use context. Other discarded
    // expressions must prove their result cannot require destruction.
    if value_use == ValueUse::Discarded
        && !matches!(
            expression,
            syn::Expr::Return(_)
                | syn::Expr::If(_)
                | syn::Expr::Match(_)
                | syn::Expr::Block(_)
                | syn::Expr::Paren(_)
                | syn::Expr::Group(_)
        )
        && !expression_is_trivial(expression, &state.trivial)
    {
        return None;
    }
    match expression {
        syn::Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => {
            state.owned.remove(&path.path.segments[0].ident.to_string());
            Some(Flow::Continues)
        }
        syn::Expr::Lit(_) | syn::Expr::Path(_) => Some(Flow::Continues),
        syn::Expr::Paren(paren) => expression_flow(&paren.expr, state, value_use),
        syn::Expr::Group(group) => expression_flow(&group.expr, state, value_use),
        syn::Expr::Tuple(tuple) => expressions_flow(tuple.elems.iter(), state),
        syn::Expr::Array(array) => expressions_flow(array.elems.iter(), state),
        syn::Expr::Struct(struct_) => {
            // Struct update transfers only unmentioned fields. The overwritten
            // fields of its base can still need destruction at scope exit.
            if struct_.rest.is_some() {
                return None;
            }
            expressions_flow(struct_.fields.iter().map(|field| &field.expr), state)
        }
        syn::Expr::Call(call) => expressions_flow(call.args.iter(), state),
        syn::Expr::Block(block) => block_flow(&block.block, state, value_use),
        syn::Expr::Return(return_) => {
            if let Some(value) = &return_.expr {
                expression_flow(value, state, ValueUse::Used)?;
            }
            state.owned.is_empty().then_some(Flow::Returned)
        }
        syn::Expr::If(if_) => {
            // The condition is borrowed for its truth value; it cannot discharge
            // ownership. Each complete branch must transfer the same inputs.
            condition_preserves_owners(&if_.cond, state)?;
            let mut then_state = state.clone();
            let mut else_state = state.clone();
            let then_flow = block_flow(&if_.then_branch, &mut then_state, value_use)?;
            let else_flow = if let Some((_, branch)) = &if_.else_branch {
                expression_flow(branch, &mut else_state, value_use)?
            } else {
                Flow::Continues
            };
            merge_branches(state, [(then_state, then_flow), (else_state, else_flow)])
        }
        syn::Expr::Match(match_) => {
            // Matching a scalar or reference cannot partially drop an owner.
            // Owned destructuring remains unproven, not silently discharged.
            if !expression_is_trivial(&match_.expr, &state.trivial) {
                return None;
            }
            condition_preserves_owners(&match_.expr, state)?;
            let mut branches = Vec::new();
            for arm in &match_.arms {
                let (pattern, guard) = match &arm.pat {
                    syn::Pat::Guard(guard) => (guard.pat.as_ref(), Some(guard.guard.as_ref())),
                    pattern => (pattern, None),
                };
                let mut branch = state.clone();
                let mut bindings = HashSet::new();
                collect_bindings(pattern, &mut bindings);
                if !bindings.is_disjoint(&branch.owned) {
                    return None;
                }
                branch.trivial.extend(bindings);
                if let Some(guard) = guard {
                    condition_preserves_owners(guard, &branch)?;
                }
                let flow = expression_flow(&arm.body, &mut branch, value_use)?;
                branches.push((branch, flow));
            }
            merge_branches(state, branches)
        }
        // Borrows and field reads never transfer an owned parameter. The
        // independent expression checker proves whether these are const-legal.
        syn::Expr::Reference(reference) => {
            is_existing_place(&reference.expr, state).then_some(Flow::Continues)
        }
        syn::Expr::Cast(cast) => {
            // Casting a borrowed enum discriminant is safe, but it does not
            // transfer an owned enum or dispose of an owned temporary safely.
            (is_existing_place(&cast.expr, state)
                || expression_is_trivial(&cast.expr, &state.trivial))
            .then_some(Flow::Continues)
        }
        syn::Expr::Field(_) | syn::Expr::Unary(_) => Some(Flow::Continues),
        _ => None,
    }
}

fn expressions_flow<'a>(
    expressions: impl IntoIterator<Item = &'a syn::Expr>,
    state: &mut DropState,
) -> Option<Flow> {
    for expression in expressions {
        if expression_flow(expression, state, ValueUse::Used)? == Flow::Returned {
            return Some(Flow::Returned);
        }
    }
    Some(Flow::Continues)
}

fn condition_preserves_owners(expression: &syn::Expr, state: &DropState) -> Option<()> {
    let mut condition = state.clone();
    (expression_flow(expression, &mut condition, ValueUse::Used)? == Flow::Continues
        && condition.owned == state.owned)
        .then_some(())
}

fn merge_branches(
    state: &mut DropState,
    branches: impl IntoIterator<Item = (DropState, Flow)>,
) -> Option<Flow> {
    let mut continuing = None;
    let mut seen = false;
    for (branch, flow) in branches {
        seen = true;
        if flow == Flow::Continues {
            if let Some(owners) = &continuing {
                if owners != &branch.owned {
                    return None;
                }
            } else {
                continuing = Some(branch.owned);
            }
        }
    }
    if !seen {
        return None;
    }
    if let Some(owners) = continuing {
        state.owned = owners;
        Some(Flow::Continues)
    } else {
        state.owned.clear();
        Some(Flow::Returned)
    }
}

fn is_existing_place(expression: &syn::Expr, state: &DropState) -> bool {
    match expression {
        syn::Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => {
            let name = path.path.segments[0].ident.to_string();
            state.owned.contains(&name) || state.trivial.contains(&name)
        }
        syn::Expr::Field(field) => is_existing_place(&field.base, state),
        syn::Expr::Paren(paren) => is_existing_place(&paren.expr, state),
        syn::Expr::Group(group) => is_existing_place(&group.expr, state),
        syn::Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Deref(_)) => {
            is_existing_place(&unary.expr, state)
        }
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
