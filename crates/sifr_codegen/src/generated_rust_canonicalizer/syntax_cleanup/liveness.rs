use std::collections::HashSet;
use syn::visit::{self, Visit};

pub(super) fn references_after_statements(statements: &[syn::Stmt]) -> HashSet<String> {
    let mut references = HashSet::new();
    for statement in statements.iter().rev() {
        update_references_crossing_statement(statement, &mut references);
    }
    references
}

pub(super) fn update_references_crossing_statement(
    statement: &syn::Stmt,
    references: &mut HashSet<String>,
) {
    let syn::Stmt::Local(local) = statement else {
        let mut collector = ReferenceCollector::default();
        collector.visit_stmt(statement);
        references.extend(collector.names);
        return;
    };
    let mut bound = HashSet::new();
    PatternNameCollector { names: &mut bound }.visit_pat(&local.pat);
    let mut collector = ReferenceCollector::default();
    if let Some(init) = &local.init {
        collector.visit_expr(&init.expr);
        if let Some((_, diverge)) = &init.diverge {
            collector.visit_expr(diverge);
        }
    }
    references.retain(|name| !bound.contains(name));
    references.extend(collector.names);
}

struct PatternNameCollector<'names> {
    names: &'names mut HashSet<String>,
}

impl<'ast> Visit<'ast> for PatternNameCollector<'_> {
    fn visit_pat_ident(&mut self, binding: &'ast syn::PatIdent) {
        self.names.insert(binding.ident.to_string());
        visit::visit_pat_ident(self, binding);
    }
}

#[derive(Default)]
struct ReferenceCollector {
    names: HashSet<String>,
    scopes: Vec<HashSet<String>>,
}

impl ReferenceCollector {
    fn is_bound(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains(name))
    }

    fn push_pattern_scope(&mut self, pattern: &syn::Pat) {
        let mut names = HashSet::new();
        PatternNameCollector { names: &mut names }.visit_pat(pattern);
        self.scopes.push(names);
    }

    fn visit_condition_and_block(&mut self, condition: &'_ syn::Expr, block: &'_ syn::Block) {
        if let syn::Expr::Let(let_) = condition {
            self.visit_expr(&let_.expr);
            self.push_pattern_scope(&let_.pat);
            self.visit_block(block);
            self.scopes.pop();
        } else {
            self.visit_expr(condition);
            self.visit_block(block);
        }
    }
}

impl<'ast> Visit<'ast> for ReferenceCollector {
    fn visit_block(&mut self, block: &'ast syn::Block) {
        self.scopes.push(HashSet::new());
        for statement in &block.stmts {
            if let syn::Stmt::Local(local) = statement {
                for attribute in &local.attrs {
                    self.visit_attribute(attribute);
                }
                if let Some(init) = &local.init {
                    self.visit_expr(&init.expr);
                    if let Some((_, diverge)) = &init.diverge {
                        self.visit_expr(diverge);
                    }
                }
                let mut bound = HashSet::new();
                PatternNameCollector { names: &mut bound }.visit_pat(&local.pat);
                if let Some(scope) = self.scopes.last_mut() {
                    scope.extend(bound);
                }
            } else {
                self.visit_stmt(statement);
            }
        }
        self.scopes.pop();
    }

    fn visit_expr_if(&mut self, branch: &'ast syn::ExprIf) {
        for attribute in &branch.attrs {
            self.visit_attribute(attribute);
        }
        self.visit_condition_and_block(&branch.cond, &branch.then_branch);
        if let Some((_, alternative)) = &branch.else_branch {
            self.visit_expr(alternative);
        }
    }

    fn visit_expr_while(&mut self, loop_: &'ast syn::ExprWhile) {
        for attribute in &loop_.attrs {
            self.visit_attribute(attribute);
        }
        self.visit_condition_and_block(&loop_.cond, &loop_.body);
    }

    fn visit_expr_for_loop(&mut self, loop_: &'ast syn::ExprForLoop) {
        for attribute in &loop_.attrs {
            self.visit_attribute(attribute);
        }
        self.visit_expr(&loop_.expr);
        self.push_pattern_scope(&loop_.pat);
        self.visit_block(&loop_.body);
        self.scopes.pop();
    }

    fn visit_expr_match(&mut self, match_: &'ast syn::ExprMatch) {
        for attribute in &match_.attrs {
            self.visit_attribute(attribute);
        }
        self.visit_expr(&match_.expr);
        for arm in &match_.arms {
            for attribute in &arm.attrs {
                self.visit_attribute(attribute);
            }
            self.push_pattern_scope(&arm.pat);
            if let syn::Pat::Guard(guard) = &arm.pat {
                self.visit_expr(&guard.guard);
            }
            self.visit_expr(&arm.body);
            self.scopes.pop();
        }
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        for attribute in &closure.attrs {
            self.visit_attribute(attribute);
        }
        let mut names = HashSet::new();
        for input in &closure.inputs {
            PatternNameCollector { names: &mut names }.visit_pat(input);
        }
        self.scopes.push(names);
        self.visit_expr(&closure.body);
        self.scopes.pop();
    }

    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        if path.qself.is_none()
            && path.path.segments.len() == 1
            && let Some(segment) = path.path.segments.first()
            && !self.is_bound(&segment.ident.to_string())
        {
            self.names.insert(segment.ident.to_string());
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        let mut names = HashSet::new();
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        } else {
            collect_token_names(rust_macro.tokens.clone(), &mut names);
        }
        collect_format_capture_names(rust_macro, &mut names);
        let free = names
            .into_iter()
            .filter(|name| !self.is_bound(name))
            .collect::<Vec<_>>();
        self.names.extend(free);
        visit::visit_macro(self, rust_macro);
    }
}

fn collect_format_capture_names(rust_macro: &syn::Macro, names: &mut HashSet<String>) {
    let Some(macro_name) = rust_macro
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
    else {
        return;
    };
    let format_index = match macro_name.as_str() {
        "format" | "print" | "println" | "eprint" | "eprintln" => 0,
        "assert" => 1,
        "assert_eq" | "assert_ne" => 2,
        "write" | "writeln" => 1,
        _ => return,
    };
    let Ok(arguments) = rust_macro.parse_body_with(
        syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
    ) else {
        return;
    };
    let Some(syn::Expr::Lit(format_expression)) = arguments.iter().nth(format_index) else {
        return;
    };
    let syn::Lit::Str(format_literal) = &format_expression.lit else {
        return;
    };
    let format = format_literal.value();
    let mut offset = 0;
    while let Some(start_relative) = format[offset..].find('{') {
        let start = offset + start_relative;
        if format.as_bytes().get(start + 1) == Some(&b'{') {
            offset = start + 2;
            continue;
        }
        let Some(end_relative) = format[start + 1..].find('}') else {
            break;
        };
        let end = start + 1 + end_relative;
        let field = format[start + 1..end].split(':').next().unwrap_or_default();
        if !field.is_empty()
            && field
                .chars()
                .all(|character| character == '_' || character.is_ascii_alphanumeric())
        {
            names.insert(field.to_string());
        }
        offset = end + 1;
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
