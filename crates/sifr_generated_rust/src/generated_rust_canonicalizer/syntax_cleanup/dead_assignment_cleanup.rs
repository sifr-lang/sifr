use std::collections::HashSet;
use syn::visit::{self, Visit};

pub(super) fn remove_dead_generated_assignments(
    block: &mut syn::Block,
    discardable: &impl Fn(&syn::Stmt) -> bool,
) {
    clean_block(block, HashSet::new(), discardable);
}

fn clean_block(
    block: &mut syn::Block,
    mut live: HashSet<String>,
    discardable: &impl Fn(&syn::Stmt) -> bool,
) -> HashSet<String> {
    // A branch-local declaration must not kill an outer binding that remains
    // live after the branch, even when both branches shadow the same name.
    let mut shadowed_live = HashSet::new();
    for statement in &block.stmts {
        if let syn::Stmt::Local(local) = statement {
            let mut bound = HashSet::new();
            PatternNames { names: &mut bound }.visit_pat(&local.pat);
            shadowed_live.extend(live.intersection(&bound).cloned());
        }
    }
    let mut index = block.stmts.len();
    while index > 0 {
        index -= 1;
        if let syn::Stmt::Expr(syn::Expr::If(branch), _) = &mut block.stmts[index] {
            let mut branch_live = clean_block(&mut branch.then_branch, live.clone(), discardable);
            if let Some((_, alternative)) = &mut branch.else_branch {
                if let syn::Expr::Block(alternative) = alternative.as_mut() {
                    branch_live.extend(clean_block(
                        &mut alternative.block,
                        live.clone(),
                        discardable,
                    ));
                } else {
                    branch_live.extend(expression_names(alternative));
                    branch_live.extend(live.clone());
                }
            } else {
                branch_live.extend(live.clone());
            }
            branch_live.extend(expression_names(&branch.cond));
            live = branch_live;
            continue;
        }

        let dead_generated_assignment = simple_assignment(&block.stmts[index])
            .is_some_and(|(name, _)| !live.contains(&name) && discardable(&block.stmts[index]));
        if dead_generated_assignment {
            block.stmts.remove(index);
            continue;
        }

        if let Some((name, value)) = simple_assignment(&block.stmts[index]) {
            live.remove(&name);
            live.extend(expression_names(value));
            continue;
        }
        if let syn::Stmt::Local(local) = &block.stmts[index] {
            let mut bound = HashSet::new();
            PatternNames { names: &mut bound }.visit_pat(&local.pat);
            live.retain(|name| !bound.contains(name));
            if let Some(init) = &local.init {
                live.extend(expression_names(&init.expr));
                if let Some((_, diverge)) = &init.diverge {
                    live.extend(expression_names(diverge));
                }
            }
            continue;
        }
        live.extend(statement_names(&block.stmts[index]));
    }
    live.extend(shadowed_live);
    live
}

fn simple_assignment(statement: &syn::Stmt) -> Option<(String, &syn::Expr)> {
    let syn::Stmt::Expr(syn::Expr::Assign(assignment), Some(_)) = statement else {
        return None;
    };
    let syn::Expr::Path(path) = assignment.left.as_ref() else {
        return None;
    };
    (path.qself.is_none() && path.path.segments.len() == 1).then(|| {
        (
            path.path.segments[0].ident.to_string(),
            assignment.right.as_ref(),
        )
    })
}

fn expression_names(expression: &syn::Expr) -> HashSet<String> {
    super::referenced_identifier_names_in_expr(expression)
}

fn statement_names(statement: &syn::Stmt) -> HashSet<String> {
    super::statement_identifier_names(statement)
}

struct PatternNames<'names> {
    names: &'names mut HashSet<String>,
}

impl<'ast> Visit<'ast> for PatternNames<'_> {
    fn visit_pat_ident(&mut self, binding: &'ast syn::PatIdent) {
        self.names.insert(binding.ident.to_string());
        visit::visit_pat_ident(self, binding);
    }
}
