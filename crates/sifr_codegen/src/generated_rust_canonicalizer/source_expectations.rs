use quote::ToTokens;
use syn::visit::{self, Visit};

const EXPECTATION_REASON: &str = "generated Rust preserves this exact typed Sifr source contract";

pub(super) fn refresh_function_expectations(
    attrs: &mut Vec<syn::Attribute>,
    signature: &syn::Signature,
    body: &syn::Block,
    owner_has_display: bool,
) {
    remove_generated_expectations(attrs);
    let mut shape = FunctionShape::default();
    shape.visit_signature(signature);
    shape.visit_block(body);
    if bool_parameter_count(signature) > 3 {
        add_expectation(attrs, "fn_params_excessive_bools");
    }
    if shape.asserts_constant_false {
        add_expectation(attrs, "assertions_on_constants");
    }
    if shape.has_approximate_constant {
        add_expectation(attrs, "approx_constant");
    }
    if signature.ident == "to_string" && owner_has_display {
        add_expectation(attrs, "inherent_to_string_shadow_display");
    }
    if !is_snake_case(&signature.ident.to_string()) {
        add_rust_expectation(attrs, "non_snake_case");
    }
    if returns_option(signature) && body_is_single_some(body) {
        add_expectation(attrs, "unnecessary_wraps");
    }
    if single_character_binding_count(signature, body) > 4 {
        add_expectation(attrs, "many_single_char_names");
    }
}

pub(super) fn refresh_struct_expectations(attrs: &mut Vec<syn::Attribute>, fields: &syn::Fields) {
    remove_generated_expectations(attrs);
    let named = fields.iter().collect::<Vec<_>>();
    if named.iter().filter(|field| type_is_bool(&field.ty)).count() > 3 {
        add_expectation(attrs, "struct_excessive_bools");
    }
}

pub(super) fn refresh_const_expectations(attrs: &mut Vec<syn::Attribute>, expression: &syn::Expr) {
    remove_generated_expectations(attrs);
    let mut shape = FunctionShape::default();
    shape.visit_expr(expression);
    if shape.has_approximate_constant {
        add_expectation(attrs, "approx_constant");
    }
}

fn remove_generated_expectations(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attribute| {
        !attribute.path().is_ident("expect")
            || !attribute
                .meta
                .to_token_stream()
                .to_string()
                .contains(EXPECTATION_REASON)
    });
}

fn add_expectation(attrs: &mut Vec<syn::Attribute>, lint: &str) {
    let Ok(lint_path) = syn::parse_str::<syn::Path>(&format!("clippy::{lint}")) else {
        return;
    };
    let reason = syn::LitStr::new(EXPECTATION_REASON, proc_macro2::Span::call_site());
    attrs.push(syn::parse_quote!(#[expect(#lint_path, reason = #reason)]));
}

fn add_rust_expectation(attrs: &mut Vec<syn::Attribute>, lint: &str) {
    let Ok(lint_path) = syn::parse_str::<syn::Path>(lint) else {
        return;
    };
    let reason = syn::LitStr::new(EXPECTATION_REASON, proc_macro2::Span::call_site());
    attrs.push(syn::parse_quote!(#[expect(#lint_path, reason = #reason)]));
}

fn bool_parameter_count(signature: &syn::Signature) -> usize {
    signature
        .inputs
        .iter()
        .filter(|argument| matches!(argument, syn::FnArg::Typed(typed) if type_is_bool(&typed.ty)))
        .count()
}

fn single_character_binding_count(signature: &syn::Signature, body: &syn::Block) -> usize {
    signature
        .inputs
        .iter()
        .filter_map(|argument| match argument {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(typed) => simple_pattern_name(&typed.pat),
        })
        .chain(body.stmts.iter().filter_map(|statement| match statement {
            syn::Stmt::Local(local) => simple_pattern_name(&local.pat),
            _ => None,
        }))
        .filter(|name| name.chars().count() == 1)
        .count()
}

fn simple_pattern_name(pattern: &syn::Pat) -> Option<String> {
    match pattern {
        syn::Pat::Ident(binding) if binding.subpat.is_none() => Some(binding.ident.to_string()),
        syn::Pat::Type(typed) => simple_pattern_name(&typed.pat),
        syn::Pat::Paren(paren) => simple_pattern_name(&paren.pat),
        _ => None,
    }
}

fn type_is_bool(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path) if path.path.is_ident("bool"))
}

fn returns_option(signature: &syn::Signature) -> bool {
    matches!(&signature.output,
        syn::ReturnType::Type(_, ty)
            if matches!(ty.as_ref(), syn::Type::Path(path)
                if path.path.segments.last().is_some_and(|segment| segment.ident == "Option")))
}

fn body_is_single_some(body: &syn::Block) -> bool {
    let [syn::Stmt::Expr(syn::Expr::Call(call), _)] = body.stmts.as_slice() else {
        return false;
    };
    matches!(call.func.as_ref(), syn::Expr::Path(path) if path.path.is_ident("Some"))
}

fn is_snake_case(name: &str) -> bool {
    !name.chars().any(char::is_uppercase)
}

#[derive(Default)]
struct FunctionShape {
    asserts_constant_false: bool,
    has_approximate_constant: bool,
}

impl<'ast> Visit<'ast> for FunctionShape {
    fn visit_lit_float(&mut self, literal: &'ast syn::LitFloat) {
        if let Ok(value) = literal.base10_parse::<f64>()
            && [
                std::f64::consts::PI,
                std::f64::consts::E,
                std::f64::consts::TAU,
            ]
            .iter()
            .any(|constant| (value.abs() - constant).abs() < 0.01)
        {
            self.has_approximate_constant = true;
        }
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        let first_argument_is_false = rust_macro
            .parse_body_with(
                syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
            )
            .ok()
            .and_then(|arguments| arguments.first().cloned())
            .is_some_and(|expression| {
                matches!(expression, syn::Expr::Lit(literal)
                    if matches!(&literal.lit, syn::Lit::Bool(value) if !value.value))
            });
        if rust_macro.path.is_ident("assert") && first_argument_is_false {
            self.asserts_constant_false = true;
        }
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        }
        visit::visit_macro(self, rust_macro);
    }
}
