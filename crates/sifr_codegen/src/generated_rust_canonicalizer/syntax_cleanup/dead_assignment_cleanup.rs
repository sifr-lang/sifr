use std::collections::HashSet;
use syn::visit::{self, Visit};

pub(super) fn remove_dead_generated_assignments(block: &mut syn::Block) {
    let bool_locals = bool_local_names(block);
    clean_block(block, HashSet::new(), &bool_locals);
}

fn clean_block(
    block: &mut syn::Block,
    mut live: HashSet<String>,
    bool_locals: &HashSet<String>,
) -> HashSet<String> {
    let mut index = block.stmts.len();
    while index > 0 {
        index -= 1;
        if let syn::Stmt::Expr(syn::Expr::If(branch), _) = &mut block.stmts[index] {
            let mut branch_live = clean_block(&mut branch.then_branch, live.clone(), bool_locals);
            if let Some((_, alternative)) = &mut branch.else_branch {
                if let syn::Expr::Block(alternative) = alternative.as_mut() {
                    branch_live.extend(clean_block(
                        &mut alternative.block,
                        live.clone(),
                        bool_locals,
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

        let dead_generated_assignment =
            simple_assignment(&block.stmts[index]).is_some_and(|(name, value)| {
                !live.contains(&name)
                    && (name.starts_with("sifr_generated_chars_")
                        || (bool_locals.contains(&name)
                            && matches!(value, syn::Expr::Lit(literal)
                                if matches!(literal.lit, syn::Lit::Bool(_)))))
            });
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
    live
}

fn bool_local_names(block: &syn::Block) -> HashSet<String> {
    let mut collector = BoolLocalCollector::default();
    collector.visit_block(block);
    collector.names
}

#[derive(Default)]
struct BoolLocalCollector {
    names: HashSet<String>,
}

impl<'ast> Visit<'ast> for BoolLocalCollector {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let syn::Pat::Type(typed) = &local.pat
            && matches!(typed.ty.as_ref(), syn::Type::Path(path)
                if path.qself.is_none() && path.path.is_ident("bool"))
            && let syn::Pat::Ident(binding) = typed.pat.as_ref()
        {
            self.names.insert(binding.ident.to_string());
        }
        visit::visit_local(self, local);
    }
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
