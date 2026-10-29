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

/// A targeted motion guard must also account for macros whose expansion is
/// unknown. Shared format parsing resolves captures for the supported families.
pub(super) fn expression_may_reference_name(expression: &syn::Expr, name: &str) -> bool {
    let mut collector = NamedReference {
        name,
        found: false,
        standard_vec: false,
    };
    collector.visit_expr(expression);
    collector.found
}

pub(super) fn statement_may_reference_name(statement: &syn::Stmt, name: &str) -> bool {
    let mut collector = NamedReference {
        name,
        found: false,
        standard_vec: false,
    };
    collector.visit_stmt(statement);
    collector.found
}

struct NamedReference<'name> {
    name: &'name str,
    found: bool,
    standard_vec: bool,
}

impl<'ast> Visit<'ast> for NamedReference<'_> {
    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        self.found |= path.qself.is_none() && path.path.is_ident(self.name);
        syn::visit::visit_expr_path(self, path);
    }

    fn visit_pat_ident(&mut self, binding: &'ast syn::PatIdent) {
        self.found |= binding.ident == self.name;
        syn::visit::visit_pat_ident(self, binding);
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        if !(crate::generated_rust_canonicalizer::is_generated_format_macro(rust_macro)
            || (self.standard_vec && rust_macro.path.is_ident("vec")))
        {
            self.found = true;
            return;
        }
        self.found |= crate::generated_rust_canonicalizer::format_capture::names(rust_macro)
            .contains(self.name);
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        } else {
            // Unparsed macro syntax may contain nested implicit captures.
            self.found = true;
        }
    }
}

/// Lexical facts supplied by the typed initializer pass.
pub(super) struct InitializerReferences {
    pub(super) standard_vec: bool,
}

impl InitializerReferences {
    pub(super) fn expression(&self, expression: &syn::Expr, name: &str) -> bool {
        let mut visitor = NamedReference {
            name,
            found: false,
            standard_vec: self.standard_vec,
        };
        visitor.visit_expr(expression);
        visitor.found
    }
    pub(super) fn statement(&self, statement: &syn::Stmt, name: &str) -> bool {
        let mut visitor = NamedReference {
            name,
            found: false,
            standard_vec: self.standard_vec,
        };
        visitor.visit_stmt(statement);
        visitor.found
    }
}
