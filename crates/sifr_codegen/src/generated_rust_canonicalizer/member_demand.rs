use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

mod generic_cleanup;
mod wildcards;

use generic_cleanup::{
    cleanup_removed_type_arguments, prune_item_members, prune_unconstrained_impl_generics,
};
use wildcards::rewrite_exhaustive_enum_wildcards;

pub(super) fn prune_unused_members(file: &mut syn::File) {
    prune_unused_members_in_scope(&mut file.items, &HashSet::new());
}

fn prune_unused_members_in_scope(
    items: &mut [syn::Item],
    external_variant_demand: &HashSet<(String, String)>,
) {
    let enum_variants = collect_enum_variants(items);
    let trait_methods = collect_trait_methods(items);
    let mut demand = MemberDemandCollector {
        enum_variants: &enum_variants,
        trait_methods: &trait_methods,
        demanded_variants: HashSet::new(),
        demanded_trait_methods: HashSet::new(),
        impl_owner: None,
    };
    for item in items
        .iter()
        .filter(|item| !matches!(item, syn::Item::Mod(_)))
    {
        demand.visit_item(item);
    }
    let mut pattern_demand = NonMatchPatternDemandCollector {
        enum_variants: &enum_variants,
        demanded_variants: HashSet::new(),
        impl_owner: None,
    };
    for item in items
        .iter()
        .filter(|item| !matches!(item, syn::Item::Mod(_)))
    {
        pattern_demand.visit_item(item);
    }
    demand
        .demanded_variants
        .extend(pattern_demand.demanded_variants);
    demand
        .demanded_variants
        .extend(external_variant_demand.iter().cloned());
    let removed_variants = enum_variants
        .iter()
        .flat_map(|(owner, variants)| {
            variants.iter().filter_map(|variant| {
                let member = (owner.clone(), variant.clone());
                (!demand.demanded_variants.contains(&member)).then_some(member)
            })
        })
        .collect::<HashSet<_>>();
    let mut pattern_cleanup = RemovedVariantPatternCleanup {
        removed_variants: &removed_variants,
        impl_owner: None,
    };
    for item in items
        .iter_mut()
        .filter(|item| !matches!(item, syn::Item::Mod(_)))
    {
        pattern_cleanup.visit_item_mut(item);
    }
    let mut removed_type_arguments = HashMap::new();
    prune_item_members(
        items,
        &trait_methods,
        &demand.demanded_variants,
        &demand.demanded_trait_methods,
        &mut removed_type_arguments,
    );
    rewrite_exhaustive_enum_wildcards(items);
    cleanup_removed_type_arguments(items, &removed_type_arguments);
    prune_unconstrained_impl_generics(items);
    for module_index in 0..items.len() {
        let module_plan = match &items[module_index] {
            syn::Item::Mod(module) => module.content.as_ref().map(|(_, nested)| {
                let module_name = module.ident.to_string();
                let variants = collect_enum_variants(nested);
                let aliases = collect_module_enum_aliases_for_scope(items, &module_name, &variants);
                let demand = external_module_variant_demand(
                    items,
                    module_index,
                    &module_name,
                    &variants,
                    &aliases,
                );
                (module_name, variants, aliases, demand)
            }),
            _ => None,
        };
        let Some((module_name, variants_before, aliases, external_demand)) = module_plan else {
            continue;
        };
        let variants_after = if let syn::Item::Mod(module) = &mut items[module_index]
            && let Some((_, nested)) = &mut module.content
        {
            prune_unused_members_in_scope(nested, &external_demand);
            collect_enum_variants(nested)
        } else {
            HashMap::new()
        };
        let removed_generated_variants = variants_before
            .iter()
            .filter(|(owner, _)| is_generated_union_name(owner))
            .flat_map(|(owner, variants)| {
                variants
                    .iter()
                    .filter(|variant| {
                        !variants_after
                            .get(owner)
                            .is_some_and(|retained| retained.contains(*variant))
                    })
                    .map(|variant| (owner.clone(), variant.clone()))
            })
            .collect::<HashSet<_>>();
        if removed_generated_variants.is_empty() {
            continue;
        }
        let mut cleanup = ExternalRemovedVariantPatternCleanup {
            module_name: &module_name,
            aliases: &aliases,
            removed_variants: &removed_generated_variants,
        };
        for (index, item) in items.iter_mut().enumerate() {
            if index != module_index {
                cleanup.visit_item_mut(item);
            }
        }
    }
}

fn is_generated_union_name(name: &str) -> bool {
    name.starts_with("__SifrUnion") || name.starts_with("SifrGeneratedUnion")
}

fn external_module_variant_demand(
    items: &[syn::Item],
    module_index: usize,
    module_name: &str,
    variants: &HashMap<String, HashSet<String>>,
    aliases: &HashMap<String, String>,
) -> HashSet<(String, String)> {
    if variants.is_empty() {
        return HashSet::new();
    }
    let mut collector = ExternalVariantDemandCollector {
        module_name,
        variants,
        aliases,
        demanded: HashSet::new(),
    };
    for (index, item) in items.iter().enumerate() {
        if index != module_index && !matches!(item, syn::Item::Use(_) | syn::Item::Mod(_)) {
            collector.visit_item(item);
        }
    }
    collector.demanded
}

fn collect_module_enum_aliases_for_scope(
    items: &[syn::Item],
    module_name: &str,
    variants: &HashMap<String, HashSet<String>>,
) -> HashMap<String, String> {
    let mut aliases = HashMap::new();
    for item in items {
        if let syn::Item::Use(use_) = item {
            collect_module_enum_aliases(&use_.tree, module_name, false, variants, &mut aliases);
        }
    }
    aliases
}

fn collect_module_enum_aliases(
    tree: &syn::UseTree,
    module_name: &str,
    inside_module: bool,
    variants: &HashMap<String, HashSet<String>>,
    aliases: &mut HashMap<String, String>,
) {
    match tree {
        syn::UseTree::Path(path) => collect_module_enum_aliases(
            &path.tree,
            module_name,
            inside_module || path.ident == module_name,
            variants,
            aliases,
        ),
        syn::UseTree::Name(name)
            if inside_module && variants.contains_key(&name.ident.to_string()) =>
        {
            aliases.insert(name.ident.to_string(), name.ident.to_string());
        }
        syn::UseTree::Rename(rename)
            if inside_module && variants.contains_key(&rename.ident.to_string()) =>
        {
            aliases.insert(rename.rename.to_string(), rename.ident.to_string());
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_module_enum_aliases(item, module_name, inside_module, variants, aliases);
            }
        }
        syn::UseTree::Glob(_) if inside_module => {
            aliases.extend(variants.keys().map(|owner| (owner.clone(), owner.clone())));
        }
        syn::UseTree::Name(_) | syn::UseTree::Rename(_) | syn::UseTree::Glob(_) => {}
    }
}

struct ExternalVariantDemandCollector<'definitions> {
    module_name: &'definitions str,
    variants: &'definitions HashMap<String, HashSet<String>>,
    aliases: &'definitions HashMap<String, String>,
    demanded: HashSet<(String, String)>,
}

impl ExternalVariantDemandCollector<'_> {
    fn collect_path(&mut self, path: &syn::Path) {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        for triple in segments.windows(3) {
            if triple[0] == self.module_name
                && self
                    .variants
                    .get(&triple[1])
                    .is_some_and(|variants| variants.contains(&triple[2]))
            {
                self.demanded.insert((triple[1].clone(), triple[2].clone()));
            }
        }
        for pair in segments.windows(2) {
            let Some(owner) = self.aliases.get(&pair[0]) else {
                continue;
            };
            if self
                .variants
                .get(owner)
                .is_some_and(|variants| variants.contains(&pair[1]))
            {
                self.demanded.insert((owner.clone(), pair[1].clone()));
            }
        }
    }
}

impl<'ast> Visit<'ast> for ExternalVariantDemandCollector<'_> {
    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        self.collect_path(&expression.path);
        visit::visit_expr_path(self, expression);
    }

    fn visit_pat(&mut self, _pattern: &'ast syn::Pat) {}

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        }
    }
}

fn collect_enum_variants(items: &[syn::Item]) -> HashMap<String, HashSet<String>> {
    let mut enums = HashMap::<String, HashSet<String>>::new();
    for item in items {
        if let syn::Item::Enum(enum_) = item {
            enums.entry(enum_.ident.to_string()).or_default().extend(
                enum_
                    .variants
                    .iter()
                    .map(|variant| variant.ident.to_string()),
            );
        }
    }
    enums
}

fn collect_trait_methods(items: &[syn::Item]) -> HashMap<String, HashSet<String>> {
    let mut traits = HashMap::<String, HashSet<String>>::new();
    for item in items {
        if let syn::Item::Trait(trait_) = item {
            traits.entry(trait_.ident.to_string()).or_default().extend(
                trait_.items.iter().filter_map(|item| match item {
                    syn::TraitItem::Fn(method) => Some(method.sig.ident.to_string()),
                    _ => None,
                }),
            );
        }
    }
    traits
}

struct MemberDemandCollector<'definitions> {
    enum_variants: &'definitions HashMap<String, HashSet<String>>,
    trait_methods: &'definitions HashMap<String, HashSet<String>>,
    demanded_variants: HashSet<(String, String)>,
    demanded_trait_methods: HashSet<String>,
    impl_owner: Option<String>,
}

impl MemberDemandCollector<'_> {
    fn collect_owner_member(&mut self, owner: &str, member: &str) {
        if self
            .enum_variants
            .get(owner)
            .is_some_and(|variants| variants.contains(member))
        {
            self.demanded_variants
                .insert((owner.to_string(), member.to_string()));
        }
        if self
            .trait_methods
            .get(owner)
            .is_some_and(|methods| methods.contains(member))
        {
            self.demanded_trait_methods.insert(member.to_string());
        }
    }

    fn collect_path(&mut self, path: &syn::Path) {
        let segments = path.segments.iter().collect::<Vec<_>>();
        for pair in segments.windows(2) {
            let owner = if pair[0].ident == "Self" {
                self.impl_owner.clone()
            } else {
                Some(pair[0].ident.to_string())
            };
            let Some(owner) = owner else {
                continue;
            };
            let member = pair[1].ident.to_string();
            self.collect_owner_member(&owner, &member);
        }
    }

    fn collect_macro_token_paths(&mut self, tokens: proc_macro2::TokenStream) {
        let tokens = tokens.into_iter().collect::<Vec<_>>();
        for window in tokens.windows(4) {
            let [
                proc_macro2::TokenTree::Ident(owner),
                proc_macro2::TokenTree::Punct(first_colon),
                proc_macro2::TokenTree::Punct(second_colon),
                proc_macro2::TokenTree::Ident(member),
            ] = window
            else {
                continue;
            };
            if first_colon.as_char() == ':' && second_colon.as_char() == ':' {
                let owner = if owner == "Self" {
                    self.impl_owner.clone()
                } else {
                    Some(owner.to_string())
                };
                if let Some(owner) = owner {
                    self.collect_owner_member(&owner, &member.to_string());
                }
            }
        }
        for token in tokens {
            if let proc_macro2::TokenTree::Group(group) = token {
                self.collect_macro_token_paths(group.stream());
            }
        }
    }
}

impl<'ast> Visit<'ast> for MemberDemandCollector<'_> {
    fn visit_item_enum(&mut self, _item: &'ast syn::ItemEnum) {}

    fn visit_item_trait(&mut self, trait_: &'ast syn::ItemTrait) {
        for item in &trait_.items {
            if let syn::TraitItem::Fn(method) = item
                && let Some(default) = &method.default
            {
                self.visit_block(default);
            }
        }
    }

    fn visit_item_impl(&mut self, impl_: &'ast syn::ItemImpl) {
        let previous = self.impl_owner.replace(type_name(&impl_.self_ty));
        visit::visit_item_impl(self, impl_);
        self.impl_owner = previous;
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let method = call.method.to_string();
        if self
            .trait_methods
            .values()
            .any(|methods| methods.contains(&method))
        {
            self.demanded_trait_methods.insert(method);
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        self.collect_path(&expression.path);
        visit::visit_expr_path(self, expression);
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
            self.collect_macro_token_paths(rust_macro.tokens.clone());
        }
    }
}

struct NonMatchPatternDemandCollector<'definitions> {
    enum_variants: &'definitions HashMap<String, HashSet<String>>,
    demanded_variants: HashSet<(String, String)>,
    impl_owner: Option<String>,
}

impl NonMatchPatternDemandCollector<'_> {
    fn collect_path(&mut self, path: &syn::Path) {
        let segments = path.segments.iter().collect::<Vec<_>>();
        for pair in segments.windows(2) {
            let owner = if pair[0].ident == "Self" {
                self.impl_owner.clone()
            } else {
                Some(pair[0].ident.to_string())
            };
            let Some(owner) = owner else {
                continue;
            };
            let variant = pair[1].ident.to_string();
            if self
                .enum_variants
                .get(&owner)
                .is_some_and(|variants| variants.contains(&variant))
            {
                self.demanded_variants.insert((owner, variant));
            }
        }
    }
}

impl<'ast> Visit<'ast> for NonMatchPatternDemandCollector<'_> {
    fn visit_item_enum(&mut self, _item: &'ast syn::ItemEnum) {}

    fn visit_item_impl(&mut self, impl_: &'ast syn::ItemImpl) {
        let previous = self.impl_owner.replace(type_name(&impl_.self_ty));
        visit::visit_item_impl(self, impl_);
        self.impl_owner = previous;
    }

    fn visit_expr_match(&mut self, match_: &'ast syn::ExprMatch) {
        self.visit_expr(&match_.expr);
        for arm in &match_.arms {
            match &arm.pat {
                syn::Pat::Or(_) => self.visit_pat(&arm.pat),
                syn::Pat::Guard(guard) => {
                    if matches!(guard.pat.as_ref(), syn::Pat::Or(_)) {
                        self.visit_pat(&guard.pat);
                    }
                    self.visit_expr(&guard.guard);
                }
                _ => {}
            }
            self.visit_expr(&arm.body);
        }
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.collect_path(path);
        visit::visit_path(self, path);
    }
}

struct RemovedVariantPatternCleanup<'definitions> {
    removed_variants: &'definitions HashSet<(String, String)>,
    impl_owner: Option<String>,
}

impl RemovedVariantPatternCleanup<'_> {
    fn pattern_uses_removed_variant(&self, pattern: &syn::Pat) -> bool {
        let mut finder = RemovedVariantPatternFinder {
            removed_variants: self.removed_variants,
            impl_owner: self.impl_owner.as_deref(),
            found: false,
        };
        finder.visit_pat(pattern);
        finder.found
    }
}

impl VisitMut for RemovedVariantPatternCleanup<'_> {
    fn visit_item_impl_mut(&mut self, impl_: &mut syn::ItemImpl) {
        let previous = self.impl_owner.replace(type_name(&impl_.self_ty));
        visit_mut::visit_item_impl_mut(self, impl_);
        self.impl_owner = previous;
    }

    fn visit_expr_match_mut(&mut self, match_: &mut syn::ExprMatch) {
        visit_mut::visit_expr_mut(self, &mut match_.expr);
        match_.arms.retain(|arm| {
            matches!(arm.pat, syn::Pat::Or(_)) || !self.pattern_uses_removed_variant(&arm.pat)
        });
        for arm in &mut match_.arms {
            if let syn::Pat::Guard(guard) = &mut arm.pat {
                visit_mut::visit_expr_mut(self, &mut guard.guard);
            }
            visit_mut::visit_expr_mut(self, &mut arm.body);
        }
    }
}

struct ExternalRemovedVariantPatternCleanup<'definitions> {
    module_name: &'definitions str,
    aliases: &'definitions HashMap<String, String>,
    removed_variants: &'definitions HashSet<(String, String)>,
}

impl ExternalRemovedVariantPatternCleanup<'_> {
    fn pattern_uses_removed_variant(&self, pattern: &syn::Pat) -> bool {
        let mut finder = ExternalRemovedVariantPatternFinder {
            module_name: self.module_name,
            aliases: self.aliases,
            removed_variants: self.removed_variants,
            found: false,
        };
        finder.visit_pat(pattern);
        finder.found
    }
}

impl VisitMut for ExternalRemovedVariantPatternCleanup<'_> {
    fn visit_expr_match_mut(&mut self, match_: &mut syn::ExprMatch) {
        visit_mut::visit_expr_mut(self, &mut match_.expr);
        match_.arms.retain(|arm| {
            matches!(arm.pat, syn::Pat::Or(_)) || !self.pattern_uses_removed_variant(&arm.pat)
        });
        for arm in &mut match_.arms {
            if let syn::Pat::Guard(guard) = &mut arm.pat {
                visit_mut::visit_expr_mut(self, &mut guard.guard);
            }
            visit_mut::visit_expr_mut(self, &mut arm.body);
        }
    }
}

struct ExternalRemovedVariantPatternFinder<'definitions> {
    module_name: &'definitions str,
    aliases: &'definitions HashMap<String, String>,
    removed_variants: &'definitions HashSet<(String, String)>,
    found: bool,
}

impl<'ast> Visit<'ast> for ExternalRemovedVariantPatternFinder<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        self.found |= segments.windows(3).any(|triple| {
            triple[0] == self.module_name
                && self
                    .removed_variants
                    .contains(&(triple[1].clone(), triple[2].clone()))
        });
        self.found |= segments.windows(2).any(|pair| {
            self.aliases.get(&pair[0]).is_some_and(|owner| {
                self.removed_variants
                    .contains(&(owner.clone(), pair[1].clone()))
            })
        });
        if !self.found {
            visit::visit_path(self, path);
        }
    }
}

struct RemovedVariantPatternFinder<'definitions, 'owner> {
    removed_variants: &'definitions HashSet<(String, String)>,
    impl_owner: Option<&'owner str>,
    found: bool,
}

impl<'ast> Visit<'ast> for RemovedVariantPatternFinder<'_, '_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let segments = path.segments.iter().collect::<Vec<_>>();
        self.found |= segments.windows(2).any(|pair| {
            let owner = if pair[0].ident == "Self" {
                self.impl_owner.map(str::to_owned)
            } else {
                Some(pair[0].ident.to_string())
            };
            owner.is_some_and(|owner| {
                self.removed_variants
                    .contains(&(owner, pair[1].ident.to_string()))
            })
        });
        if !self.found {
            visit::visit_path(self, path);
        }
    }
}

fn type_name(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(path) => path
            .path
            .segments
            .last()
            .map_or_else(String::new, |segment| segment.ident.to_string()),
        _ => String::new(),
    }
}
