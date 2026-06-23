//! Proc-macros for [`quote-use`](https://docs.rs/quote-use/).

use proc_macro2::{Spacing, TokenStream, TokenTree};
use proc_macro_utils::TokenStreamExt;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{Result, Token};
use use_parser::{Use, UseItem};

mod prelude;

mod use_parser;

/// Internal, only used through macros in [`quote_use`](https://docs.rs/quote-use).
/// Input is `quote_use_impl!((<path to quote macro>) ([span_expr =>])
/// (<tokens>))`.
#[proc_macro]
pub fn quote_use_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = input.parser();
    // This is internal, i.e., these expects are fine, they are not error handling.
    let path = input
        .next_group()
        .expect("there should be three `(...)`")
        .stream();
    let span = input
        .next_group()
        .expect("there should be three `(...)`")
        .stream();
    let uses = input
        .next_group()
        .expect("there should be three `(...)`")
        .stream();
    let uses: QuoteUse = match syn::parse2(uses) {
        Ok(uses) => uses,
        Err(err) => return err.into_compile_error().into(),
    };

    quote! {
        #path!{
            #span
            #uses
        }
    }
    .into()
}

struct QuoteUse(Vec<Use>, TokenStream);
impl Parse for QuoteUse {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut uses = Vec::new();
        while input.peek(Token![#]) && input.peek2(Token![use]) {
            input.parse::<Token![#]>().expect("# was peeked before");
            uses.extend_from_slice(&UseItem::parse(input)?.0);
        }

        Ok(QuoteUse(uses, input.parse()?))
    }
}

impl ToTokens for QuoteUse {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self(uses, tail) = self;
        let mut prelude = true;
        let mut std = true;
        let mut uses: Vec<_> = uses
            .iter()
            .filter(|u| {
                if u.1 == "no_prelude" {
                    prelude = false;
                    false
                } else if u.1 == "no_std" {
                    std = false;
                    false
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        if prelude {
            uses.extend(prelude::prelude(std));
        }

        tokens.extend(replace_in_group(&uses, tail.clone()));
    }
}

fn replace_in_group(uses: &[Use], tokens: TokenStream) -> TokenStream {
    use State::*;
    #[derive(Clone, Copy)]
    enum State {
        Path,
        Pound,
        Normal,
    }
    let mut state = Normal;

    tokens
        .into_iter()
        .flat_map(|token| {
            match (&token, state) {
                (TokenTree::Ident(ident), Normal) => {
                    if let Some(Use(path, _)) = uses.iter().find(|item| &item.1 == ident) {
                        return quote!(#path);
                    }
                }
                // first colon
                (TokenTree::Punct(punct), _)
                    if punct.spacing() == Spacing::Joint && punct.as_char() == ':' =>
                {
                    state = Path;
                }
                // second colon
                (TokenTree::Punct(punct), _) if punct.as_char() == ':' => (),
                // quote var `#ident`
                (TokenTree::Punct(punct), _) if punct.as_char() == '#' => {
                    state = Pound;
                }
                (TokenTree::Group(group), _) => {
                    let tokens = replace_in_group(uses, group.stream());
                    return match group.delimiter() {
                        proc_macro2::Delimiter::Parenthesis => quote!((#tokens)),
                        proc_macro2::Delimiter::Brace => quote!({#tokens}),
                        proc_macro2::Delimiter::Bracket => quote!([#tokens]),
                        proc_macro2::Delimiter::None => tokens,
                    };
                }
                _ => {
                    state = Normal;
                }
            };
            quote!(#token)
        })
        .collect()
}
