use std::collections::{HashMap, HashSet};

use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

pub(super) fn prune_item_members(
    items: &mut [syn::Item],
    trait_methods: &HashMap<String, HashSet<String>>,
    demanded_variants: &HashSet<(String, String)>,
    demanded_trait_methods: &HashSet<String>,
) {
    for item in items {
        match item {
            syn::Item::Enum(enum_) => {
                let owner = enum_.ident.to_string();
                enum_.variants = std::mem::take(&mut enum_.variants)
                    .into_iter()
                    .filter(|variant| {
                        demanded_variants.contains(&(owner.clone(), variant.ident.to_string()))
                    })
                    .collect();
            }
            syn::Item::Trait(trait_) => {
                let methods = trait_.items.iter().filter_map(|item| match item {
                    syn::TraitItem::Fn(method) => Some(method.sig.ident.to_string()),
                    _ => None,
                });
                let known = methods.collect::<HashSet<_>>();
                trait_.items.retain(|item| {
                    !matches!(item, syn::TraitItem::Fn(method)
                        if known.contains(&method.sig.ident.to_string())
                            && !demanded_trait_methods.contains(&method.sig.ident.to_string()))
                });
            }
            syn::Item::Impl(impl_) => {
                let trait_name = impl_
                    .trait_
                    .as_ref()
                    .and_then(|(path, _)| path.segments.last())
                    .map(|segment| segment.ident.to_string());
                if trait_name
                    .as_ref()
                    .is_some_and(|trait_name| trait_methods.contains_key(trait_name))
                {
                    impl_.items.retain(|item| {
                        !matches!(item, syn::ImplItem::Fn(method)
                            if !demanded_trait_methods.contains(&method.sig.ident.to_string()))
                    });
                }
            }
            _ => {}
        }
    }
}

pub(super) fn prune_unused_aggregate_type_parameters(items: &mut [syn::Item]) {
    loop {
        let mut removed_type_arguments = HashMap::new();
        for item in items.iter_mut() {
            let removed = match item {
                syn::Item::Enum(enum_) => {
                    let owner = enum_.ident.to_string();
                    (owner, remove_unused_enum_type_parameters(enum_))
                }
                syn::Item::Struct(struct_) => {
                    let owner = struct_.ident.to_string();
                    (owner, remove_unused_struct_type_parameters(struct_))
                }
                _ => continue,
            };
            if !removed.1.is_empty() {
                removed_type_arguments.insert(removed.0, removed.1);
            }
        }
        if removed_type_arguments.is_empty() {
            return;
        }
        cleanup_removed_type_arguments(items, &removed_type_arguments);
    }
}

pub(super) fn cleanup_removed_type_arguments(
    items: &mut [syn::Item],
    removed_type_arguments: &HashMap<String, HashSet<usize>>,
) {
    let mut cleanup = RemovedTypeArgumentCleanup {
        removed_type_arguments,
    };
    for item in items
        .iter_mut()
        .filter(|item| !matches!(item, syn::Item::Mod(_)))
    {
        cleanup.visit_item_mut(item);
    }
}

fn remove_unused_enum_type_parameters(enum_: &mut syn::ItemEnum) -> HashSet<usize> {
    let parameter_names = enum_
        .generics
        .params
        .iter()
        .map(|parameter| match parameter {
            syn::GenericParam::Type(type_) => Some(type_.ident.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let candidates = parameter_names
        .iter()
        .flatten()
        .cloned()
        .collect::<HashSet<_>>();
    let mut uses = GenericNameUseCollector {
        candidates: &candidates,
        used: HashSet::new(),
    };
    for variant in &enum_.variants {
        uses.visit_variant(variant);
    }
    let removed_names = parameter_names
        .iter()
        .flatten()
        .filter(|name| !uses.used.contains(*name))
        .cloned()
        .collect::<HashSet<_>>();
    let removed_positions = parameter_names
        .iter()
        .enumerate()
        .filter_map(|(index, name)| {
            name.as_ref()
                .is_some_and(|name| removed_names.contains(name))
                .then_some(index)
        })
        .collect::<HashSet<_>>();
    remove_generic_parameters(&mut enum_.generics, &removed_names);
    removed_positions
}

fn remove_unused_struct_type_parameters(struct_: &mut syn::ItemStruct) -> HashSet<usize> {
    let parameter_names = struct_
        .generics
        .params
        .iter()
        .map(|parameter| match parameter {
            syn::GenericParam::Type(type_) => Some(type_.ident.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let candidates = parameter_names
        .iter()
        .flatten()
        .cloned()
        .collect::<HashSet<_>>();
    let mut uses = GenericNameUseCollector {
        candidates: &candidates,
        used: HashSet::new(),
    };
    uses.visit_fields(&struct_.fields);
    let removed_names = parameter_names
        .iter()
        .flatten()
        .filter(|name| !uses.used.contains(*name))
        .cloned()
        .collect::<HashSet<_>>();
    let removed_positions = parameter_names
        .iter()
        .enumerate()
        .filter_map(|(index, name)| {
            name.as_ref()
                .is_some_and(|name| removed_names.contains(name))
                .then_some(index)
        })
        .collect::<HashSet<_>>();
    remove_generic_parameters(&mut struct_.generics, &removed_names);
    removed_positions
}

struct GenericNameUseCollector<'names> {
    candidates: &'names HashSet<String>,
    used: HashSet<String>,
}

impl<'ast> Visit<'ast> for GenericNameUseCollector<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        for segment in &path.segments {
            let name = segment.ident.to_string();
            if self.candidates.contains(&name) {
                self.used.insert(name);
            }
        }
        visit::visit_path(self, path);
    }
}

struct RemovedTypeArgumentCleanup<'definitions> {
    removed_type_arguments: &'definitions HashMap<String, HashSet<usize>>,
}

impl VisitMut for RemovedTypeArgumentCleanup<'_> {
    fn visit_path_segment_mut(&mut self, segment: &mut syn::PathSegment) {
        visit_mut::visit_path_segment_mut(self, segment);
        let Some(removed) = self.removed_type_arguments.get(&segment.ident.to_string()) else {
            return;
        };
        let syn::PathArguments::AngleBracketed(arguments) = &mut segment.arguments else {
            return;
        };
        arguments.args = std::mem::take(&mut arguments.args)
            .into_iter()
            .enumerate()
            .filter_map(|(index, argument)| (!removed.contains(&index)).then_some(argument))
            .collect();
        if arguments.args.is_empty() {
            segment.arguments = syn::PathArguments::None;
        }
    }
}

pub(super) fn prune_unconstrained_impl_generics(items: &mut [syn::Item]) {
    for item in items {
        let syn::Item::Impl(impl_) = item else {
            continue;
        };
        let candidates = impl_
            .generics
            .type_params()
            .map(|parameter| parameter.ident.to_string())
            .collect::<HashSet<_>>();
        let mut uses = GenericNameUseCollector {
            candidates: &candidates,
            used: HashSet::new(),
        };
        if let Some((trait_path, _)) = &impl_.trait_ {
            uses.visit_path(trait_path);
        }
        uses.visit_type(&impl_.self_ty);
        for impl_item in &impl_.items {
            uses.visit_impl_item(impl_item);
        }
        let removed = candidates
            .difference(&uses.used)
            .cloned()
            .collect::<HashSet<_>>();
        remove_generic_parameters(&mut impl_.generics, &removed);
    }
}

fn remove_generic_parameters(generics: &mut syn::Generics, removed: &HashSet<String>) {
    if removed.is_empty() {
        return;
    }
    generics.params = std::mem::take(&mut generics.params)
        .into_iter()
        .filter(|parameter| {
            !matches!(parameter, syn::GenericParam::Type(type_)
                if removed.contains(&type_.ident.to_string()))
        })
        .collect();
    if let Some(where_clause) = &mut generics.where_clause {
        where_clause.predicates = std::mem::take(&mut where_clause.predicates)
            .into_iter()
            .filter(|predicate| {
                let mut uses = GenericNameUseCollector {
                    candidates: removed,
                    used: HashSet::new(),
                };
                uses.visit_where_predicate(predicate);
                uses.used.is_empty()
            })
            .collect();
        if where_clause.predicates.is_empty() {
            generics.where_clause = None;
        }
    }
    if generics.params.is_empty() {
        generics.lt_token = None;
        generics.gt_token = None;
    }
}
