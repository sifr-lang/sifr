use std::collections::HashMap;

use proc_macro2::{TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::parse::Parser;
use syn::visit_mut::{self, VisitMut};

type VariantMap = HashMap<String, HashMap<String, String>>;

pub(super) fn canonicalize_local_enum_variants(file: &mut syn::File) {
    let mut candidates = Vec::new();
    collect_mapping_candidates(&file.items, &mut candidates);
    let mut owner_counts = HashMap::<String, usize>::new();
    for (owner, _) in &candidates {
        *owner_counts.entry(owner.clone()).or_default() += 1;
    }
    let mappings = candidates
        .into_iter()
        .filter(|(owner, _)| owner_counts.get(owner).copied() == Some(1))
        .collect::<VariantMap>();
    if mappings.is_empty() {
        return;
    }
    rewrite_definitions(&mut file.items, &mappings);
    VariantReferenceRewriter {
        mappings: &mappings,
        impl_owner: None,
    }
    .visit_file_mut(file);
}

fn collect_mapping_candidates(
    items: &[syn::Item],
    candidates: &mut Vec<(String, HashMap<String, String>)>,
) {
    for item in items {
        match item {
            syn::Item::Enum(enum_)
                if enum_
                    .variants
                    .iter()
                    .all(|variant| matches!(variant.fields, syn::Fields::Unit)) =>
            {
                let variant_candidates = enum_
                    .variants
                    .iter()
                    .map(|variant| {
                        let original = variant.ident.to_string();
                        let canonical = upper_camel_variant(&original);
                        (original, canonical)
                    })
                    .collect::<Vec<_>>();
                if variant_candidates
                    .iter()
                    .all(|(original, canonical)| original == canonical)
                {
                    continue;
                }
                let mut counts = HashMap::<String, usize>::new();
                for (_, canonical) in &variant_candidates {
                    *counts.entry(canonical.clone()).or_default() += 1;
                }
                let variants = variant_candidates
                    .into_iter()
                    .map(|(original, mut canonical)| {
                        if counts.get(&canonical).copied().unwrap_or_default() > 1 {
                            canonical.push('X');
                            for byte in original.as_bytes() {
                                append_hex_byte(&mut canonical, *byte);
                            }
                        }
                        (original, canonical)
                    })
                    .collect();
                candidates.push((enum_.ident.to_string(), variants));
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &module.content {
                    collect_mapping_candidates(nested, candidates);
                }
            }
            _ => {}
        }
    }
}

fn append_hex_byte(output: &mut String, byte: u8) {
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
    output.push(char::from(HEX_DIGITS[usize::from(byte >> 4)]));
    output.push(char::from(HEX_DIGITS[usize::from(byte & 0x0f)]));
}

fn upper_camel_variant(original: &str) -> String {
    if !original.contains('_')
        && original.chars().any(char::is_lowercase)
        && original.chars().next().is_some_and(char::is_uppercase)
    {
        return original.to_string();
    }
    let mut canonical = String::new();
    for word in original.split('_').filter(|word| !word.is_empty()) {
        let mut characters = word.chars();
        if let Some(first) = characters.next() {
            canonical.extend(first.to_uppercase());
            canonical.extend(characters.flat_map(char::to_lowercase));
        }
    }
    if canonical.is_empty() {
        canonical.push_str("Variant");
    }
    if canonical == "Self" {
        canonical.push_str("Value");
    }
    canonical
}

fn rewrite_definitions(items: &mut Vec<syn::Item>, mappings: &VariantMap) {
    let mut rewritten = Vec::with_capacity(items.len());
    for mut item in std::mem::take(items) {
        match &mut item {
            syn::Item::Enum(enum_) => {
                let owner = enum_.ident.to_string();
                let Some(variants) = mappings.get(&owner) else {
                    rewritten.push(item);
                    continue;
                };
                remove_debug_derive(&mut enum_.attrs);
                let debug_arms = enum_
                    .variants
                    .iter_mut()
                    .map(|variant| {
                        let original = variant.ident.to_string();
                        let canonical = variants.get(&original).unwrap_or(&original);
                        variant.ident = syn::Ident::new(canonical, variant.ident.span());
                        let variant_ident = &variant.ident;
                        let text = syn::LitStr::new(&original, variant.ident.span());
                        quote!(Self::#variant_ident => #text)
                    })
                    .collect::<Vec<_>>();
                let enum_ident = enum_.ident.clone();
                rewritten.push(item);
                rewritten.push(syn::parse_quote! {
                    impl ::std::fmt::Debug for #enum_ident {
                        fn fmt(
                            &self,
                            formatter: &mut ::std::fmt::Formatter<'_>,
                        ) -> ::std::fmt::Result {
                            formatter.write_str(match self { #(#debug_arms),* })
                        }
                    }
                });
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &mut module.content {
                    rewrite_definitions(nested, mappings);
                }
                rewritten.push(item);
            }
            _ => rewritten.push(item),
        }
    }
    *items = rewritten;
}

fn remove_debug_derive(attributes: &mut Vec<syn::Attribute>) {
    attributes.retain_mut(|attribute| {
        let syn::Meta::List(meta) = &mut attribute.meta else {
            return true;
        };
        if !meta.path.is_ident("derive") {
            return true;
        }
        let Ok(paths) = syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated
            .parse2(meta.tokens.clone())
        else {
            return true;
        };
        let retained = paths
            .into_iter()
            .filter(|path| {
                path.segments
                    .last()
                    .is_none_or(|segment| segment.ident != "Debug")
            })
            .collect::<syn::punctuated::Punctuated<_, syn::Token![,]>>();
        meta.tokens = retained.to_token_stream();
        !retained.is_empty()
    });
}

struct VariantReferenceRewriter<'mappings> {
    mappings: &'mappings VariantMap,
    impl_owner: Option<String>,
}

impl VariantReferenceRewriter<'_> {
    fn rewrite_path(&self, path: &mut syn::Path) {
        let mut index = 0;
        while index + 1 < path.segments.len() {
            let owner = if path.segments[index].ident == "Self" {
                self.impl_owner.clone()
            } else {
                Some(path.segments[index].ident.to_string())
            };
            let replacement = owner
                .as_deref()
                .and_then(|owner| self.mappings.get(owner))
                .and_then(|variants| variants.get(&path.segments[index + 1].ident.to_string()))
                .cloned();
            if let Some(replacement) = replacement {
                let span = path.segments[index + 1].ident.span();
                path.segments[index + 1].ident = syn::Ident::new(&replacement, span);
            }
            index += 1;
        }
    }

    fn rewrite_tokens(&self, tokens: TokenStream) -> TokenStream {
        let mut tokens = tokens.into_iter().collect::<Vec<_>>();
        for token in &mut tokens {
            if let TokenTree::Group(group) = token {
                let mut rewritten =
                    proc_macro2::Group::new(group.delimiter(), self.rewrite_tokens(group.stream()));
                rewritten.set_span(group.span());
                *token = TokenTree::Group(rewritten);
            }
        }
        for index in 0..tokens.len().saturating_sub(3) {
            let (TokenTree::Ident(owner), TokenTree::Punct(first), TokenTree::Punct(second)) =
                (&tokens[index], &tokens[index + 1], &tokens[index + 2])
            else {
                continue;
            };
            if first.as_char() != ':' || second.as_char() != ':' {
                continue;
            }
            let owner = if owner == "Self" {
                self.impl_owner.clone()
            } else {
                Some(owner.to_string())
            };
            let Some(TokenTree::Ident(member)) = tokens.get_mut(index + 3) else {
                continue;
            };
            if let Some(replacement) = owner
                .as_deref()
                .and_then(|owner| self.mappings.get(owner))
                .and_then(|variants| variants.get(&member.to_string()))
            {
                *member = proc_macro2::Ident::new(replacement, member.span());
            }
        }
        tokens.into_iter().collect()
    }
}

impl VisitMut for VariantReferenceRewriter<'_> {
    fn visit_item_impl_mut(&mut self, implementation: &mut syn::ItemImpl) {
        let previous = self
            .impl_owner
            .replace(super::impl_self_type_name(&implementation.self_ty).unwrap_or_default());
        visit_mut::visit_item_impl_mut(self, implementation);
        self.impl_owner = previous;
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        self.rewrite_path(path);
        visit_mut::visit_path_mut(self, path);
    }

    fn visit_macro_mut(&mut self, rust_macro: &mut syn::Macro) {
        rust_macro.tokens = self.rewrite_tokens(rust_macro.tokens.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::canonicalize_local_enum_variants;
    use quote::ToTokens;

    #[test]
    fn same_named_nested_enums_do_not_share_an_unqualified_variant_map() {
        let mut file = syn::parse_file(
            r#"
                mod left {
                    #[derive(Debug)]
                    enum State { NOT_READY }
                    fn state() -> State { State::NOT_READY }
                }
                mod right {
                    #[derive(Debug)]
                    enum State { NOT_READY, READY }
                    fn state() -> State { State::READY }
                }
            "#,
        )
        .expect("test source should parse");

        canonicalize_local_enum_variants(&mut file);
        let canonical = file.to_token_stream().to_string();

        assert_eq!(canonical.matches("enum State").count(), 2, "{canonical}");
        assert!(canonical.contains("State :: NOT_READY"), "{canonical}");
        assert!(canonical.contains("State :: READY"), "{canonical}");
        assert!(
            !canonical.contains("impl :: std :: fmt :: Debug"),
            "{canonical}"
        );
    }
}
