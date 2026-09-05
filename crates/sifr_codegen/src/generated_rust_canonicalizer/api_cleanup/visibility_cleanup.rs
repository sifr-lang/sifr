use std::collections::HashMap;

use syn::visit::{self, Visit};

type ItemPath = Vec<String>;

pub(super) fn publicize_public_enum_field_owners(items: &mut [syn::Item]) {
    let mut demands = HashMap::<ItemPath, syn::Visibility>::new();
    collect_visibility_demands(items, &[], &mut demands);
    apply_visibility_demands(items, &[], &demands);
}

fn collect_visibility_demands(
    items: &[syn::Item],
    module_path: &[String],
    demands: &mut HashMap<ItemPath, syn::Visibility>,
) {
    for item in items {
        match item {
            syn::Item::Enum(item_enum) if !matches!(item_enum.vis, syn::Visibility::Inherited) => {
                let mut collector = TypePathCollector::default();
                for field in item_enum
                    .variants
                    .iter()
                    .flat_map(|variant| &variant.fields)
                {
                    collector.visit_type(&field.ty);
                }
                for path in collector.paths {
                    let Some(owner) = resolve_item_path(module_path, &path) else {
                        continue;
                    };
                    demands
                        .entry(owner)
                        .and_modify(|visibility| {
                            if matches!(item_enum.vis, syn::Visibility::Public(_)) {
                                *visibility = item_enum.vis.clone();
                            }
                        })
                        .or_insert_with(|| item_enum.vis.clone());
                }
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &module.content {
                    let mut nested_path = module_path.to_vec();
                    nested_path.push(module.ident.to_string());
                    collect_visibility_demands(nested, &nested_path, demands);
                }
            }
            _ => {}
        }
    }
}

fn apply_visibility_demands(
    items: &mut [syn::Item],
    module_path: &[String],
    demands: &HashMap<ItemPath, syn::Visibility>,
) {
    for item in items {
        match item {
            syn::Item::Struct(item_struct)
                if matches!(item_struct.vis, syn::Visibility::Inherited) =>
            {
                let mut owner = module_path.to_vec();
                owner.push(item_struct.ident.to_string());
                if let Some(visibility) = demands.get(&owner) {
                    item_struct.vis = visibility.clone();
                }
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &mut module.content {
                    let mut nested_path = module_path.to_vec();
                    nested_path.push(module.ident.to_string());
                    apply_visibility_demands(nested, &nested_path, demands);
                }
            }
            _ => {}
        }
    }
}

fn resolve_item_path(module_path: &[String], path: &syn::Path) -> Option<ItemPath> {
    if path.segments.is_empty() {
        return None;
    }
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let mut resolved = if path.leading_colon.is_some() || segments[0] == "crate" {
        Vec::new()
    } else {
        module_path.to_vec()
    };
    let mut index = usize::from(segments[0] == "crate" || segments[0] == "self");
    while segments
        .get(index)
        .is_some_and(|segment| segment == "super")
    {
        resolved.pop()?;
        index += 1;
    }
    resolved.extend(segments[index..].iter().cloned());
    Some(resolved)
}

#[derive(Default)]
struct TypePathCollector {
    paths: Vec<syn::Path>,
}

impl Visit<'_> for TypePathCollector {
    fn visit_type_path(&mut self, path: &syn::TypePath) {
        if path.qself.is_none() {
            self.paths.push(path.path.clone());
        }
        visit::visit_type_path(self, path);
    }
}
