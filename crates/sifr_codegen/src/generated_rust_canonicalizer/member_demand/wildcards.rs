use std::collections::{HashMap, HashSet};

use syn::visit_mut::{self, VisitMut};

pub(super) fn rewrite_exhaustive_enum_wildcards(items: &mut [syn::Item]) {
    let definitions = items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Enum(enum_) => Some((
                enum_.ident.to_string(),
                enum_
                    .variants
                    .iter()
                    .map(|variant| (variant.ident.to_string(), variant.fields.clone()))
                    .collect::<HashMap<_, _>>(),
            )),
            _ => None,
        })
        .collect::<HashMap<_, _>>();
    if definitions.is_empty() {
        return;
    }
    let mut cleanup = ExhaustiveEnumWildcardCleanup {
        definitions: &definitions,
    };
    for item in items
        .iter_mut()
        .filter(|item| !matches!(item, syn::Item::Enum(_) | syn::Item::Mod(_)))
    {
        cleanup.visit_item_mut(item);
    }
}

struct ExhaustiveEnumWildcardCleanup<'definitions> {
    definitions: &'definitions HashMap<String, HashMap<String, syn::Fields>>,
}

impl VisitMut for ExhaustiveEnumWildcardCleanup<'_> {
    fn visit_expr_match_mut(&mut self, match_: &mut syn::ExprMatch) {
        visit_mut::visit_expr_match_mut(self, match_);
        let mut owners = HashSet::new();
        let mut covered_variants = HashSet::new();
        for arm in &match_.arms {
            collect_top_level_variant_patterns(
                &arm.pat,
                self.definitions,
                &mut owners,
                &mut covered_variants,
            );
        }
        let mut owners = owners.iter();
        let Some(owner) = owners.next().cloned() else {
            return;
        };
        if owners.next().is_some() {
            return;
        }
        let Some(variants) = self.definitions.get(&owner) else {
            return;
        };
        let remaining = variants
            .keys()
            .filter(|variant| !covered_variants.contains(&(owner.clone(), (*variant).clone())))
            .cloned()
            .collect::<Vec<_>>();
        match remaining.as_slice() {
            [] => match_
                .arms
                .retain(|arm| !is_generated_wildcard_pattern(&arm.pat)),
            [variant] => {
                let Some(fields) = variants.get(variant) else {
                    return;
                };
                for arm in &mut match_.arms {
                    if is_generated_wildcard_pattern(&arm.pat) {
                        arm.pat = explicit_variant_pattern(&owner, variant, fields, &arm.pat);
                    }
                }
            }
            _ => {}
        }
    }
}

fn collect_top_level_variant_patterns(
    pattern: &syn::Pat,
    definitions: &HashMap<String, HashMap<String, syn::Fields>>,
    owners: &mut HashSet<String>,
    variants: &mut HashSet<(String, String)>,
) {
    let path = match pattern {
        syn::Pat::Path(path) => Some(&path.path),
        syn::Pat::Struct(struct_) => Some(&struct_.path),
        syn::Pat::TupleStruct(tuple) => Some(&tuple.path),
        syn::Pat::Or(or_) => {
            for case in &or_.cases {
                collect_top_level_variant_patterns(case, definitions, owners, variants);
            }
            None
        }
        syn::Pat::Guard(guard) => {
            collect_top_level_variant_patterns(&guard.pat, definitions, owners, variants);
            None
        }
        _ => None,
    };
    if let Some(path) = path {
        let segments = path.segments.iter().collect::<Vec<_>>();
        for pair in segments.windows(2) {
            let owner = pair[0].ident.to_string();
            let variant = pair[1].ident.to_string();
            if definitions
                .get(&owner)
                .is_some_and(|variants| variants.contains_key(&variant))
            {
                owners.insert(owner.clone());
                variants.insert((owner, variant));
            }
        }
    }
}

fn is_generated_wildcard_pattern(pattern: &syn::Pat) -> bool {
    matches!(pattern, syn::Pat::Wild(_))
        || matches!(pattern, syn::Pat::Ident(binding) if binding.subpat.is_none())
}

fn explicit_variant_pattern(
    owner: &str,
    variant: &str,
    fields: &syn::Fields,
    wildcard: &syn::Pat,
) -> syn::Pat {
    let owner = syn::Ident::new(owner, proc_macro2::Span::call_site());
    let variant = syn::Ident::new(variant, proc_macro2::Span::call_site());
    let shape: syn::Pat = match fields {
        syn::Fields::Named(_) => syn::parse_quote!(#owner::#variant { .. }),
        syn::Fields::Unnamed(_) => syn::parse_quote!(#owner::#variant(..)),
        syn::Fields::Unit => syn::parse_quote!(#owner::#variant),
    };
    if let syn::Pat::Ident(binding) = wildcard {
        let binding = binding.clone();
        syn::parse_quote!(#binding @ #shape)
    } else {
        shape
    }
}
