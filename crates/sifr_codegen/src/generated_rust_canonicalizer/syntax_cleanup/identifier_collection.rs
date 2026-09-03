use super::mutability_cleanup::collect_token_identifiers;
use std::collections::HashSet;
use syn::visit::Visit;

pub(in super::super) fn statement_identifier_names(statement: &syn::Stmt) -> HashSet<String> {
    let mut collector = IdentifierCollector::default();
    collector.visit_stmt(statement);
    collector.names
}

pub(super) fn identifier_names_in_expr(expression: &syn::Expr) -> HashSet<String> {
    let mut collector = IdentifierCollector::default();
    collector.visit_expr(expression);
    collector.names
}

pub(super) fn identifier_names_in_pattern(pattern: &syn::Pat) -> HashSet<String> {
    let mut collector = IdentifierCollector::default();
    collector.visit_pat(pattern);
    collector.names
}

pub(super) fn referenced_identifier_names_in_expr(expression: &syn::Expr) -> HashSet<String> {
    let mut collector = ReferenceIdentifierCollector::default();
    collector.visit_expr(expression);
    collector.names
}

pub(in super::super) fn expression_has_control_carrier(expression: &syn::Expr) -> bool {
    let mut collector = ControlCarrierCollector { found: false };
    collector.visit_expr(expression);
    collector.found
}

#[derive(Default)]
struct IdentifierCollector {
    names: HashSet<String>,
}

impl<'ast> Visit<'ast> for IdentifierCollector {
    fn visit_ident(&mut self, identifier: &'ast proc_macro2::Ident) {
        self.names.insert(identifier.to_string());
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        } else {
            collect_token_identifiers(rust_macro.tokens.clone(), &mut self.names);
        }
        self.names
            .extend(crate::generated_rust_canonicalizer::format_capture::names(
                rust_macro,
            ));
    }
}

#[derive(Default)]
struct ReferenceIdentifierCollector {
    names: HashSet<String>,
}

impl<'ast> Visit<'ast> for ReferenceIdentifierCollector {
    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        if path.qself.is_none()
            && path.path.segments.len() == 1
            && let Some(segment) = path.path.segments.first()
        {
            self.names.insert(segment.ident.to_string());
        }
        syn::visit::visit_expr_path(self, path);
    }

    fn visit_pat(&mut self, _pattern: &'ast syn::Pat) {}

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        } else {
            collect_token_identifiers(rust_macro.tokens.clone(), &mut self.names);
        }
        self.names
            .extend(crate::generated_rust_canonicalizer::format_capture::names(
                rust_macro,
            ));
    }
}

struct ControlCarrierCollector {
    found: bool,
}

impl<'ast> Visit<'ast> for ControlCarrierCollector {
    fn visit_expr_async(&mut self, _expression: &'ast syn::ExprAsync) {}

    fn visit_expr_closure(&mut self, _expression: &'ast syn::ExprClosure) {}

    fn visit_expr_try(&mut self, _expression: &'ast syn::ExprTry) {
        self.found = true;
    }

    fn visit_expr_await(&mut self, _expression: &'ast syn::ExprAwait) {
        self.found = true;
    }

    fn visit_expr_yield(&mut self, _expression: &'ast syn::ExprYield) {
        self.found = true;
    }

    fn visit_expr_return(&mut self, _expression: &'ast syn::ExprReturn) {
        self.found = true;
    }

    fn visit_expr_break(&mut self, _expression: &'ast syn::ExprBreak) {
        self.found = true;
    }

    fn visit_expr_continue(&mut self, _expression: &'ast syn::ExprContinue) {
        self.found = true;
    }
}
