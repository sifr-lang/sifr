use std::collections::{HashMap, HashSet};

use syn::visit::{self, Visit};

type ItemPath = Vec<String>;
// None is public; a path is the ancestor module allowed by a restricted visibility.
type Audience = Option<ItemPath>;

pub(super) fn publicize_public_enum_field_owners(items: &mut [syn::Item]) {
    let mut owners = HashMap::new();
    collect_owners(items, &[], &mut owners);
    let mut aliases = owners
        .keys()
        .map(|path| (path.join("::"), path.clone()))
        .collect();
    super::super::syntax_cleanup::scoped_imports::expand_items(items, &mut aliases);
    let mut edges = Vec::new();
    let mut demands = owners.clone();
    collect_demands(items, &[], &aliases, &mut demands, &mut edges);
    // Public payloads can expose a chain of associated types (e.g. Deref parents).
    // Solve the declaration graph before changing visibility; item order is irrelevant.
    loop {
        let mut changed = false;
        for (owner, target) in &edges {
            if let Some(audience) = demands.get(owner) {
                let audience = audience.clone();
                changed |= demand(&mut demands, target.clone(), audience);
            }
        }
        if !changed {
            break;
        }
    }
    apply_demands(items, &[], &demands);
}

fn audience(visibility: &syn::Visibility, scope: &[String]) -> Audience {
    match visibility {
        syn::Visibility::Public(_) => None,
        syn::Visibility::Inherited => Some(scope.to_vec()),
        syn::Visibility::Restricted(restricted) => resolve_item_path(scope, &restricted.path),
    }
}

fn demand(demands: &mut HashMap<ItemPath, Audience>, owner: ItemPath, requested: Audience) -> bool {
    match demands.entry(owner) {
        std::collections::hash_map::Entry::Vacant(entry) => {
            entry.insert(requested);
            true
        }
        std::collections::hash_map::Entry::Occupied(mut entry) => {
            let merged = match (entry.get(), requested) {
                (Some(current), Some(requested)) => Some(
                    current
                        .iter()
                        .zip(requested)
                        .take_while(|(a, b)| *a == b)
                        .map(|(a, _)| a.clone())
                        .collect(),
                ),
                _ => None,
            };
            if *entry.get() == merged {
                false
            } else {
                entry.insert(merged);
                true
            }
        }
    }
}

fn collect_owners(items: &[syn::Item], scope: &[String], owners: &mut HashMap<ItemPath, Audience>) {
    for item in items {
        match item {
            syn::Item::Struct(item) => {
                let mut owner = scope.to_vec();
                owner.push(item.ident.to_string());
                owners.insert(owner, audience(&item.vis, scope));
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &module.content {
                    let mut path = scope.to_vec();
                    path.push(module.ident.to_string());
                    collect_owners(nested, &path, owners);
                }
            }
            _ => {}
        }
    }
}

fn owner_path(
    scope: &[String],
    path: &syn::Path,
    aliases: &HashMap<String, ItemPath>,
) -> Option<ItemPath> {
    aliases
        .get(&resolve_item_path(scope, path)?.join("::"))
        .cloned()
}

fn collect_demands(
    items: &[syn::Item],
    scope: &[String],
    aliases: &HashMap<String, ItemPath>,
    demands: &mut HashMap<ItemPath, Audience>,
    edges: &mut Vec<(ItemPath, ItemPath)>,
) {
    for item in items {
        match item {
            syn::Item::Enum(item) if !matches!(item.vis, syn::Visibility::Inherited) => {
                let mut collector = TypePathCollector::new(&item.generics);
                for field in item.variants.iter().flat_map(|variant| &variant.fields) {
                    collector.visit_type(&field.ty);
                }
                for path in collector.paths {
                    if let Some(owner) = owner_path(scope, &path, aliases) {
                        demand(demands, owner, audience(&item.vis, scope));
                    }
                }
            }
            syn::Item::Impl(item) if generated_parent_contract(item) => {
                let syn::Type::Path(self_ty) = item.self_ty.as_ref() else {
                    continue;
                };
                let mut collector = TypePathCollector::new(&item.generics);
                if self_ty.qself.is_some() || collector.shadowed(&self_ty.path) {
                    continue;
                }
                let Some(owner) = owner_path(scope, &self_ty.path, aliases) else {
                    continue;
                };
                for member in &item.items {
                    if let syn::ImplItem::Type(associated) = member {
                        // Associated generic parameters are local to this type declaration.
                        let mut associated_collector = TypePathCollector::new(&item.generics);
                        associated_collector.exclude_generics(&associated.generics);
                        associated_collector.visit_type(&associated.ty);
                        collector.paths.extend(associated_collector.paths);
                    }
                }
                for path in collector.paths {
                    if let Some(target) = owner_path(scope, &path, aliases) {
                        edges.push((owner.clone(), target));
                    }
                }
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &module.content {
                    let mut path = scope.to_vec();
                    path.push(module.ident.to_string());
                    collect_demands(nested, &path, aliases, demands, edges);
                }
            }
            _ => {}
        }
    }
}

// The emitter expresses class ancestry through this standard contract. A
// same-spelled local/private trait does not impose a public parent boundary.
fn generated_parent_contract(item: &syn::ItemImpl) -> bool {
    item.trait_.as_ref().is_some_and(|(path, _)| {
        path.leading_colon.is_some()
            && path
                .segments
                .iter()
                .map(|part| part.ident.to_string())
                .eq(["std", "ops", "Deref"].map(str::to_string))
    })
}

fn apply_demands(items: &mut [syn::Item], scope: &[String], demands: &HashMap<ItemPath, Audience>) {
    for item in items {
        match item {
            syn::Item::Struct(item) => {
                let mut owner = scope.to_vec();
                owner.push(item.ident.to_string());
                if let Some(requested) = demands.get(&owner) {
                    let current = audience(&item.vis, scope);
                    let widen = match (&current, requested) {
                        (Some(current), Some(requested)) => {
                            current.starts_with(requested) && current != requested
                        }
                        (Some(_), None) => true,
                        _ => false,
                    };
                    if widen {
                        item.vis = match requested {
                            None => syn::parse_quote!(pub),
                            Some(path)
                                if scope.len() == path.len() + 1 && scope.starts_with(path) =>
                            {
                                syn::parse_quote!(pub(super))
                            }
                            Some(path) if path.is_empty() => syn::parse_quote!(pub(crate)),
                            Some(path) => {
                                let parts = path
                                    .iter()
                                    .map(|part| {
                                        syn::Ident::new(part, proc_macro2::Span::call_site())
                                    })
                                    .collect::<Vec<_>>();
                                syn::parse_quote!(pub(in crate::#(#parts)::*))
                            }
                        };
                    }
                }
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &mut module.content {
                    let mut path = scope.to_vec();
                    path.push(module.ident.to_string());
                    apply_demands(nested, &path, demands);
                }
            }
            _ => {}
        }
    }
}

fn resolve_item_path(scope: &[String], path: &syn::Path) -> Option<ItemPath> {
    // An absolute Rust path names an external crate, not a same-spelled local item.
    if path.leading_colon.is_some() {
        return None;
    }
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    super::super::syntax_cleanup::scoped_imports::qualified_path(scope, &segments)
}

#[derive(Clone, Default)]
struct TypePathCollector {
    paths: Vec<syn::Path>,
    parameters: HashSet<String>,
}
impl TypePathCollector {
    fn new(generics: &syn::Generics) -> Self {
        let mut value = Self::default();
        value.exclude_generics(generics);
        value
    }
    fn exclude_generics(&mut self, generics: &syn::Generics) {
        self.parameters.extend(
            generics
                .type_params()
                .map(|parameter| parameter.ident.to_string()),
        );
    }
    fn shadowed(&self, path: &syn::Path) -> bool {
        path.segments
            .first()
            .is_some_and(|part| self.parameters.contains(&part.ident.to_string()))
    }
}
impl Visit<'_> for TypePathCollector {
    fn visit_type_path(&mut self, path: &syn::TypePath) {
        if path.qself.is_none() && !self.shadowed(&path.path) {
            self.paths.push(path.path.clone());
        }
        visit::visit_type_path(self, path);
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/generated_rust_canonicalizer/api_cleanup/visibility_cleanup/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../tests/unit/generated_rust_canonicalizer/api_cleanup/visibility_cleanup/boundary_tests.rs"]
mod boundary_tests;
