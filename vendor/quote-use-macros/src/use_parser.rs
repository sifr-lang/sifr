use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{braced, Error, Result, Token};

#[derive(Debug, Clone)]
pub enum IdentOrPounded {
    Ident(Ident),
    Pounded(Token![#], TokenTree),
}

impl IdentOrPounded {
    fn is_self(&self) -> bool {
        if let Self::Ident(ident) = self {
            ident == "self"
        } else {
            false
        }
    }

    fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }
}

impl ToTokens for IdentOrPounded {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            IdentOrPounded::Ident(ident) => ident.to_tokens(tokens),
            IdentOrPounded::Pounded(pound, tt) => {
                pound.to_tokens(tokens);
                tt.to_tokens(tokens);
            }
        }
    }
}

impl Parse for IdentOrPounded {
    fn parse(input: ParseStream) -> Result<Self> {
        Ident::parse_any(input)
            .map(Self::Ident)
            .or_else(|_| Ok(Self::Pounded(input.parse()?, input.parse()?)))
    }
}

#[derive(Clone, Debug, Default)]
pub struct Path(Vec<IdentOrPounded>);

impl Path {
    fn push(&mut self, value: IdentOrPounded) {
        self.0.push(value);
    }

    fn pop_self(&mut self) -> bool {
        self.0.last().map_or(false, IdentOrPounded::is_self) && {
            self.pop();
            true
        }
    }

    fn get_ident(&self) -> Result<&Ident> {
        match self.0.last().expect("path should contain a segment") {
            IdentOrPounded::Ident(ident) => Ok(ident),
            IdentOrPounded::Pounded(pound, _) => Err(Error::new_spanned(
                pound,
                "expected ident as last path segment",
            )),
        }
    }

    fn pop(&mut self) {
        self.0
            .pop()
            .expect("path should contain at least one segment");
    }
}

impl ToTokens for Path {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let first = self.0.first().expect("path should contain a segment");
        let colons = first.is_ident().then_some(quote!(::));
        let tail = &self.0[1..];
        quote!(#colons #first #(::#tail)*).to_tokens(tokens)
    }
}

#[derive(Clone, Debug)]
pub struct Use(pub Path, pub Ident);

#[derive(Clone, Debug, Default)]
pub struct UseItem(pub Vec<Use>);

// INPUTS:
// a::b::{a::{}, b}
fn parse_use_segment(
    parent: &Path,
    input: ParseStream,
    output: &mut Vec<Use>,
    inner: bool,
) -> Result<()> {
    let mut path = parent.clone();
    loop {
        let la = input.lookahead1();
        if la.peek(Ident::peek_any) || la.peek(Token![#]) {
            path.push(input.parse()?);
            let la = input.lookahead1();
            if inner && (la.peek(Token![,]) || input.is_empty()) || !inner && la.peek(Token![;]) {
                path.pop_self();
                output.push(Use(path.clone(), path.get_ident()?.clone()));
                break;
            } else if la.peek(Token![as]) {
                input.parse::<Token![as]>()?;
                let alias: Ident = input.parse()?;
                path.pop_self();
                output.push(Use(path, alias));
                break;
            } else if la.peek(Token![::]) {
                input.parse::<Token![::]>()?;
                continue;
            } else {
                return Err(la.error());
            }
        } else if la.peek(Brace) {
            // A group
            let content;
            braced!(content in input);
            loop {
                parse_use_segment(&path, &content, output, true)?;
                if content.is_empty() {
                    break;
                } else {
                    content.parse::<Token![,]>()?;
                    if content.is_empty() {
                        break;
                    }
                }
            }
            let la = input.lookahead1();
            if inner && (input.is_empty() || la.peek(Token![,])) || !inner && la.peek(Token![;]) {
                break;
            } else {
                return Err(la.error());
            }
        } else {
            return Err(la.error());
        }
    }
    Ok(())
}

impl Parse for UseItem {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }
        let mut output = Vec::new();
        <Token![use]>::parse(input)?;
        Option::<Token![::]>::parse(input)?;

        parse_use_segment(&Default::default(), input, &mut output, false)?;

        <Token![;]>::parse(input)?;

        Ok(Self(output))
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use quote::ToTokens;
    use syn::parse::Parser;
    use syn::parse_str;

    use super::*;

    macro_rules! assert_use_item {
        ($use:literal, $($path:literal as $ident:ident),* $(,)*) => {
            let UseItem(uses) = parse_str($use).unwrap();
            let mut uses = uses.into_iter();
            $(
                let Use(path, ident) = uses.next().unwrap();
                assert_eq!(path.into_token_stream().to_string().replace(' ', ""), $path);
                assert_eq!(ident, stringify!($ident));
            )*
        };
    }

    #[test]
    fn use_item() {
        assert_use_item!("use ::a::b;", "::a::b" as b);
        assert_use_item!(
            "use a::{c, self, b};",
            "::a::c" as c,
            "::a" as a,
            "::a::b" as b
        );
        assert_use_item!("use a::{self as c, b as a};", "::a" as c, "::a::b" as a);
        assert_use_item!(
            "use a::{b::{a, b}, c};",
            "::a::b::a" as a,
            "::a::b::b" as b,
            "::a::c" as c
        );
        assert_use_item!("use #var::a;", "#var::a" as a);
        assert_use_item!("use ::a::#var::a;", "::a::#var::a" as a);
        assert_use_item!("use ::a::#var as a;", "::a::#var" as a);
    }

    macro_rules! assert_error {
        ($use:literal) => {
            UseItem::parse.parse_str($use).unwrap_err();
        };
    }

    #[test]
    fn error() {
        assert_error!("use ::a::#b;");
    }
}
