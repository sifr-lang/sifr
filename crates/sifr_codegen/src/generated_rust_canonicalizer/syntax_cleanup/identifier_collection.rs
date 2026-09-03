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
        collect_format_capture_names(rust_macro, &mut self.names);
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
        collect_format_capture_names(rust_macro, &mut self.names);
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
