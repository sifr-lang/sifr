use std::collections::HashSet;
use syn::visit::{self, Visit};

pub(super) fn remove_dead_generated_cache_assignments(block: &mut syn::Block) {
    clean_block(block, HashSet::new());
}

fn clean_block(block: &mut syn::Block, mut live: HashSet<String>) -> HashSet<String> {
    let mut index = block.stmts.len();
    while index > 0 {
        index -= 1;
        if let syn::Stmt::Expr(syn::Expr::If(branch), _) = &mut block.stmts[index] {
            let mut branch_live = clean_block(&mut branch.then_branch, live.clone());
            if let Some((_, alternative)) = &mut branch.else_branch {
                if let syn::Expr::Block(alternative) = alternative.as_mut() {
                    branch_live.extend(clean_block(&mut alternative.block, live.clone()));
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

        let dead_cache_assignment =
            simple_assignment(&block.stmts[index]).is_some_and(|(name, _)| {
                name.starts_with("sifr_generated_chars_") && !live.contains(&name)
            });
        if dead_cache_assignment {
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
    let mut collector = ReferenceNames::default();
    collector.visit_expr(expression);
    collector.names
}

fn statement_names(statement: &syn::Stmt) -> HashSet<String> {
    let mut collector = ReferenceNames::default();
    collector.visit_stmt(statement);
    collector.names
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

#[derive(Default)]
struct ReferenceNames {
    names: HashSet<String>,
}

impl<'ast> Visit<'ast> for ReferenceNames {
    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        if path.qself.is_none()
            && path.path.segments.len() == 1
            && let Some(segment) = path.path.segments.first()
        {
            self.names.insert(segment.ident.to_string());
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        collect_token_names(rust_macro.tokens.clone(), &mut self.names);
        visit::visit_macro(self, rust_macro);
    }
}

fn collect_token_names(tokens: proc_macro2::TokenStream, names: &mut HashSet<String>) {
    for token in tokens {
        match token {
            proc_macro2::TokenTree::Ident(identifier) => {
                names.insert(identifier.to_string());
            }
            proc_macro2::TokenTree::Group(group) => collect_token_names(group.stream(), names),
            _ => {}
        }
    }
}
