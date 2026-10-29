use std::collections::{HashMap, HashSet};

use syn::visit_mut::{self, VisitMut};

pub(super) fn remove_redundant_local_call_borrows(file: &mut syn::File) {
    LocalFunctionBorrowRewriter.visit_file_mut(file);
}

struct LocalFunctionBorrowRewriter;

impl VisitMut for LocalFunctionBorrowRewriter {
    fn visit_item_fn_mut(&mut self, function: &mut syn::ItemFn) {
        rewrite_function_block(&function.sig, &mut function.block);
        visit_mut::visit_item_fn_mut(self, function);
    }

    fn visit_impl_item_fn_mut(&mut self, function: &mut syn::ImplItemFn) {
        rewrite_function_block(&function.sig, &mut function.block);
        visit_mut::visit_impl_item_fn_mut(self, function);
    }
}

fn rewrite_function_block(signature: &syn::Signature, block: &mut syn::Block) {
    let mut plans = HashMap::<String, Vec<ReferenceKind>>::new();
    let mut ambiguous = HashSet::new();
    for statement in &block.stmts {
        let syn::Stmt::Item(syn::Item::Fn(function)) = statement else {
            continue;
        };
        let key = signature_key(&function.sig);
        let parameters = reference_parameters(&function.sig);
        plans
            .entry(key.clone())
            .and_modify(|known| {
                if *known != parameters {
                    ambiguous.insert(key.clone());
                }
            })
            .or_insert(parameters);
    }
    plans.retain(|key, parameters| {
        parameters.iter().any(|kind| *kind != ReferenceKind::Owned) && !ambiguous.contains(key)
    });
    if plans.is_empty() {
        return;
    }
    let borrowed = signature
        .inputs
        .iter()
        .filter_map(|argument| {
            let syn::FnArg::Typed(parameter) = argument else {
                return None;
            };
            matches!(parameter.ty.as_ref(), syn::Type::Reference(_))
                .then(|| simple_pattern_name(&parameter.pat))
                .flatten()
        })
        .collect();
    LocalCallBorrowRewriter { plans, borrowed }.visit_block_mut(block);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReferenceKind {
    Owned,
    Shared,
    Str,
}

fn reference_parameters(signature: &syn::Signature) -> Vec<ReferenceKind> {
    signature
        .inputs
        .iter()
        .filter_map(|argument| {
            let syn::FnArg::Typed(parameter) = argument else {
                return None;
            };
            Some(match parameter.ty.as_ref() {
                syn::Type::Reference(reference)
                    if matches!(reference.elem.as_ref(), syn::Type::Path(path)
                        if path.path.is_ident("str")) =>
                {
                    ReferenceKind::Str
                }
                syn::Type::Reference(reference) if reference.mutability.is_none() => {
                    ReferenceKind::Shared
                }
                _ => ReferenceKind::Owned,
            })
        })
        .collect()
}

struct LocalCallBorrowRewriter {
    plans: HashMap<String, Vec<ReferenceKind>>,
    borrowed: HashSet<String>,
}

impl VisitMut for LocalCallBorrowRewriter {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let outer = self.borrowed.clone();
        for statement in &mut block.stmts {
            self.visit_stmt_mut(statement);
            if let syn::Stmt::Local(local) = statement {
                for name in super::identifier_names_in_pattern(&local.pat) {
                    self.borrowed.remove(&name);
                }
                if let Some(name) = simple_pattern_name(&local.pat) {
                    let borrowed = matches!(&local.pat, syn::Pat::Type(typed)
                    if matches!(typed.ty.as_ref(), syn::Type::Reference(_)))
                        || local.init.as_ref().is_some_and(|init| {
                            matches!(init.expr.as_ref(), syn::Expr::Reference(_))
                        });
                    if borrowed {
                        self.borrowed.insert(name);
                    }
                }
            }
        }
        self.borrowed = outer;
    }

    fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
        visit_mut::visit_expr_call_mut(self, call);
        let syn::Expr::Path(path) = call.func.as_ref() else {
            return;
        };
        let Some(function) = path.path.get_ident() else {
            return;
        };
        let Some(parameters) = self.plans.get(&format!("{}#{}", function, call.args.len())) else {
            return;
        };
        for (argument, kind) in call.args.iter_mut().zip(parameters) {
            if *kind == ReferenceKind::Owned {
                continue;
            }
            let syn::Expr::Reference(reference) = argument else {
                continue;
            };
            if expression_is_borrowed_binding(&reference.expr, &self.borrowed) {
                *argument = reference.expr.as_ref().clone();
            } else if *kind == ReferenceKind::Str
                && matches!(reference.expr.as_ref(), syn::Expr::Call(call)
                    if matches!(call.func.as_ref(), syn::Expr::Path(path)
                        if path.path.segments.len() == 2
                            && path.path.segments[0].ident == "String"
                            && path.path.segments[1].ident == "new")
                        && call.args.is_empty())
            {
                *argument = syn::parse_quote!("");
            }
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn signature_key(signature: &syn::Signature) -> String {
    let arguments = signature
        .inputs
        .iter()
        .filter(|argument| matches!(argument, syn::FnArg::Typed(_)))
        .count();
    format!("{}#{arguments}", signature.ident)
}

fn simple_pattern_name(pattern: &syn::Pat) -> Option<String> {
    match pattern {
        syn::Pat::Ident(binding) if binding.subpat.is_none() => Some(binding.ident.to_string()),
        syn::Pat::Type(typed) => simple_pattern_name(&typed.pat),
        syn::Pat::Paren(paren) => simple_pattern_name(&paren.pat),
        _ => None,
    }
}

fn expression_is_borrowed_binding(expression: &syn::Expr, borrowed: &HashSet<String>) -> bool {
    matches!(expression, syn::Expr::Path(path)
        if path.path.get_ident().is_some_and(|name| borrowed.contains(&name.to_string())))
}
