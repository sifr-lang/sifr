use quote::{ToTokens, quote};
use syn::visit::Visit;
use syn::visit_mut::VisitMut;

pub(super) enum MacroArguments {
    List(syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>),
    Repeat(syn::ExprRepeat),
}

impl MacroArguments {
    pub(super) fn parse(rust_macro: &syn::Macro) -> Option<Self> {
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            return Some(Self::List(arguments));
        }
        if rust_macro
            .path
            .segments
            .last()
            .is_some_and(|part| part.ident == "vec")
        {
            let tokens = &rust_macro.tokens;
            return syn::parse2(quote!([#tokens])).ok().map(Self::Repeat);
        }
        None
    }

    pub(super) fn visit<'ast>(&'ast self, visitor: &mut impl Visit<'ast>) {
        match self {
            Self::List(arguments) => {
                for argument in arguments {
                    visitor.visit_expr(argument);
                }
            }
            Self::Repeat(repeat) => visitor.visit_expr_repeat(repeat),
        }
    }

    pub(super) fn visit_mut(&mut self, visitor: &mut impl VisitMut) {
        match self {
            Self::List(arguments) => {
                for argument in arguments {
                    visitor.visit_expr_mut(argument);
                }
            }
            Self::Repeat(repeat) => visitor.visit_expr_repeat_mut(repeat),
        }
    }

    pub(super) fn tokens(&self) -> proc_macro2::TokenStream {
        match self {
            Self::List(arguments) => arguments.to_token_stream(),
            Self::Repeat(repeat) => {
                let value = &repeat.expr;
                let length = &repeat.len;
                quote!(#value; #length)
            }
        }
    }
}
