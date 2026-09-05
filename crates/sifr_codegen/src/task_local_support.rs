//! The declaration grammar of the compiler-owned `tokio::task_local!` support.
//!
//! These are static declarations without initializers, not ordinary Rust items.
//! Share their identity between visibility, dependency discovery, and pruning.

use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

pub(crate) struct TaskLocal {
    attrs: Vec<syn::Attribute>,
    pub(crate) visibility: syn::Visibility,
    pub(crate) name: syn::Ident,
    ty: syn::Type,
}

impl ToTokens for TaskLocal {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            attrs,
            visibility,
            name,
            ty,
        } = self;
        tokens.extend(quote!(#(#attrs)* #visibility static #name: #ty;));
    }
}

pub(crate) struct TaskLocals(pub(crate) Vec<TaskLocal>);

impl Parse for TaskLocals {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut declarations = Vec::new();
        while !input.is_empty() {
            let attrs = input.call(syn::Attribute::parse_outer)?;
            let visibility = input.parse()?;
            input.parse::<syn::Token![static]>()?;
            let name = input.parse()?;
            input.parse::<syn::Token![:]>()?;
            let ty = input.parse()?;
            if !input.is_empty() {
                input.parse::<syn::Token![;]>()?;
            }
            declarations.push(TaskLocal {
                attrs,
                visibility,
                name,
                ty,
            });
        }
        Ok(Self(declarations))
    }
}

pub(crate) fn declarations(mac: &syn::Macro) -> Option<TaskLocals> {
    let mut segments = mac.path.segments.iter();
    if segments
        .next()
        .is_none_or(|segment| segment.ident != "tokio")
        || segments
            .next()
            .is_none_or(|segment| segment.ident != "task_local")
        || segments.next().is_some()
        || mac
            .path
            .segments
            .iter()
            .any(|segment| !segment.arguments.is_empty())
    {
        return None;
    }
    Some(syn::parse2(mac.tokens.clone()).unwrap_or_else(|error| {
        panic!("invalid compiler-owned tokio::task_local declaration: {error}")
    }))
}

/// Give each macro-owned declaration an independent dependency/pruning entry.
pub(crate) fn split_declarations(item: syn::Item) -> Vec<syn::Item> {
    if let syn::Item::Macro(mac) = &item
        && let Some(TaskLocals(declarations)) = declarations(&mac.mac)
    {
        return declarations
            .into_iter()
            .map(|declaration| {
                let mut single = mac.clone();
                single.mac.tokens = declaration.into_token_stream();
                syn::Item::Macro(single)
            })
            .collect();
    }
    vec![item]
}

#[cfg(test)]
#[path = "task_local_support_tests.rs"]
mod tests;
