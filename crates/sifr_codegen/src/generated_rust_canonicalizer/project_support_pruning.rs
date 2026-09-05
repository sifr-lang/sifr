use super::impl_self_type_name;
use super::method_demand::{demanded_inherent_method_names, prune_inherent_methods};
use std::collections::HashSet;
use syn::visit::{self, Visit};

pub(crate) fn prune_generated_project_owners(
    prelude_source: &str,
    support_source: &str,
    body_consumers: &[&str],
) -> Result<(String, String), String> {
    let mut prelude =
        prune_generated_project_prelude_for_consumers(prelude_source, body_consumers)?;
    let mut support = String::new();
    for _ in 0..16 {
        let support_consumers = std::iter::once(prelude.as_str())
            .chain(body_consumers.iter().copied())
            .collect::<Vec<_>>();
        let next_support =
            prune_generated_support_for_consumers(support_source, &support_consumers)?;
        let prelude_consumers = body_consumers
            .iter()
            .copied()
            .chain((!next_support.trim().is_empty()).then_some(next_support.as_str()))
            .collect::<Vec<_>>();
        let next_prelude =
            prune_generated_project_prelude_for_consumers(prelude_source, &prelude_consumers)?;
        if next_prelude == prelude && next_support == support {
            return Ok((next_prelude, next_support));
        }
        prelude = next_prelude;
        support = next_support;
    }
    Err("generated project support ownership did not reach a fixed point".to_string())
}

pub(crate) fn prune_generated_support_for_consumers(
    support_source: &str,
    consumer_sources: &[&str],
) -> Result<String, String> {
    let support_file = syn::parse_file(support_source)
        .map_err(|error| format!("failed to parse generated project support: {error}"))?;
    let support_names = crate::stdlib_filter::rust_source_defined_item_names(support_source);
    let mut consumer_roots = HashSet::new();
    let mut consumer_items = Vec::new();
    for source in consumer_sources {
        consumer_roots.extend(crate::stdlib_filter::rust_source_referenced_item_names(
            source,
            &support_names,
        ));
        consumer_roots.extend(crate::stdlib_filter::rust_source_required_trait_names(
            source,
            support_source,
        )?);
        let file = syn::parse_file(source)
            .map_err(|error| format!("failed to parse generated support consumer: {error}"))?;
        consumer_items.extend(file.items);
    }

    let mut seed_file = support_file.clone();
    remove_inherent_impls(&mut seed_file.items);
    let seed_source = prettyplease::unparse(&seed_file);
    let selected_seed =
        crate::stdlib_filter::filter_stdlib_ir_to_needed(&seed_source, &consumer_roots);
    let mut selected_names = crate::stdlib_filter::rust_source_defined_item_names(&selected_seed);
    selected_names.extend(crate::error_refs::collect_source_builtin_error_classes(
        &selected_seed,
        crate::BUILTIN_ERROR_CLASSES,
    ));

    let mut demand_file = support_file.clone();
    demand_file.items.retain(|item| {
        matches!(item, syn::Item::Impl(item_impl) if item_impl.trait_.is_none())
            || support_item_name(item).is_none_or(|name| selected_names.contains(&name))
    });
    demand_file.items.extend(consumer_items);
    let demanded_methods = demanded_inherent_method_names(&demand_file);

    let mut pruned_support = support_file;
    prune_inherent_methods(&mut pruned_support.items, &demanded_methods);
    let pruned_source = prettyplease::unparse(&pruned_support);
    let filtered =
        crate::stdlib_filter::filter_stdlib_ir_to_needed(&pruned_source, &selected_names);
    if crate::stdlib_filter::rust_source_defined_item_names(&filtered).is_empty() {
        Ok(String::new())
    } else {
        Ok(filtered)
    }
}

pub(crate) fn prune_generated_project_prelude_for_consumers(
    prelude_source: &str,
    consumer_sources: &[&str],
) -> Result<String, String> {
    let mut prelude = syn::parse_file(prelude_source)
        .map_err(|error| format!("failed to parse generated project prelude: {error}"))?;
    let mut demand_file = prelude.clone();
    for source in consumer_sources {
        let consumer = syn::parse_file(source)
            .map_err(|error| format!("failed to parse generated project consumer: {error}"))?;
        demand_file.items.extend(consumer.items);
    }
    let demanded_methods = demanded_inherent_method_names(&demand_file);
    prune_inherent_methods(&mut prelude.items, &demanded_methods);

    let nominal_index = prelude.items.iter().position(
        |item| matches!(item, syn::Item::Mod(module) if module.ident == "__sifr_project_nominals"),
    );
    let Some(nominal_index) = nominal_index else {
        return Ok(prettyplease::unparse(&prelude));
    };
    let nested_source = match &prelude.items[nominal_index] {
        syn::Item::Mod(module) => module
            .content
            .as_ref()
            .map(|(_, items)| {
                prettyplease::unparse(&syn::File {
                    shebang: None,
                    frontmatter: None,
                    attrs: Vec::new(),
                    items: items.clone(),
                })
            })
            .unwrap_or_default(),
        _ => String::new(),
    };
    let nominal_names = crate::stdlib_filter::rust_source_defined_item_names(&nested_source);
    let mut root_source = prelude.clone();
    if let syn::Item::Mod(module) = &mut root_source.items[nominal_index]
        && let Some((_, items)) = &mut module.content
    {
        items.clear();
    }
    root_source
        .items
        .retain(|item| nominal_reexport_name(item).is_none());
    let mut required_nominals = crate::stdlib_filter::rust_source_referenced_item_names(
        &prettyplease::unparse(&root_source),
        &nominal_names,
    );
    for source in consumer_sources {
        required_nominals.extend(crate::stdlib_filter::rust_source_referenced_item_names(
            source,
            &nominal_names,
        ));
    }
    prelude.items.retain(|item| {
        nominal_reexport_name(item).is_none_or(|name| required_nominals.contains(&name))
    });
    let nominal_index = prelude.items.iter().position(
        |item| matches!(item, syn::Item::Mod(module) if module.ident == "__sifr_project_nominals"),
    );
    let Some(nominal_index) = nominal_index else {
        return Ok(prettyplease::unparse(&prelude));
    };
    let mut outer = prelude.clone();
    if let syn::Item::Mod(module) = &mut outer.items[nominal_index]
        && let Some((_, items)) = &mut module.content
    {
        items.clear();
    }
    let outer_source = prettyplease::unparse(&outer);
    let nested_consumers = std::iter::once(outer_source.as_str())
        .chain(consumer_sources.iter().copied())
        .collect::<Vec<_>>();
    let pruned_nested = prune_generated_support_for_consumers(&nested_source, &nested_consumers)?;
    let nested_file = syn::parse_file(&pruned_nested)
        .map_err(|error| format!("failed to parse pruned project nominals: {error}"))?;
    if let syn::Item::Mod(module) = &mut prelude.items[nominal_index]
        && let Some((_, items)) = &mut module.content
    {
        *items = nested_file.items;
    }
    let nominal_module_is_empty = matches!(
        &prelude.items[nominal_index],
        syn::Item::Mod(module)
            if module.content.as_ref().is_none_or(|(_, items)| items.is_empty())
    );
    let has_nominal_reexports = prelude
        .items
        .iter()
        .any(|item| nominal_reexport_name(item).is_some());
    if nominal_module_is_empty && !has_nominal_reexports {
        prelude.items.remove(nominal_index);
    }
    Ok(prettyplease::unparse(&prelude))
}

fn nominal_reexport_name(item: &syn::Item) -> Option<String> {
    let syn::Item::Use(item_use) = item else {
        return None;
    };
    let syn::UseTree::Path(module) = &item_use.tree else {
        return None;
    };
    if module.ident != "__sifr_project_nominals" {
        return None;
    }
    match module.tree.as_ref() {
        syn::UseTree::Name(name) => Some(name.ident.to_string()),
        syn::UseTree::Rename(rename) => Some(rename.rename.to_string()),
        _ => None,
    }
}

pub(crate) fn import_generated_support_in_project_nominals(
    prelude_source: &str,
    required: &HashSet<String>,
) -> Result<String, String> {
    let mut prelude = syn::parse_file(prelude_source)
        .map_err(|error| format!("failed to parse generated project prelude: {error}"))?;
    let support_import = syn::parse_str::<syn::Item>(&render_generated_support_import(required))
        .map_err(|error| format!("failed to build generated support import: {error}"))?;
    for item in &mut prelude.items {
        if let syn::Item::Mod(module) = item
            && module.ident == "__sifr_project_nominals"
            && let Some((_, items)) = &mut module.content
        {
            items.insert(0, support_import);
            break;
        }
    }
    Ok(prettyplease::unparse(&prelude))
}

pub(crate) fn import_project_bindings_in_project_nominals(
    prelude_source: &str,
    candidates: &HashSet<String>,
) -> Result<String, String> {
    let mut prelude = syn::parse_file(prelude_source)
        .map_err(|error| format!("failed to parse generated project prelude: {error}"))?;
    for item in &mut prelude.items {
        let syn::Item::Mod(module) = item else {
            continue;
        };
        if module.ident != "__sifr_project_nominals" {
            continue;
        }
        let Some((_, items)) = &mut module.content else {
            continue;
        };
        let nested_source = prettyplease::unparse(&syn::File {
            shebang: None,
            frontmatter: None,
            attrs: Vec::new(),
            items: items.clone(),
        });
        let required =
            crate::stdlib_filter::rust_source_referenced_item_names(&nested_source, candidates);
        if required.is_empty() {
            break;
        }
        let mut names = required.into_iter().collect::<Vec<_>>();
        names.sort();
        let import = syn::parse_str::<syn::Item>(&format!("use crate::{{{}}};", names.join(",")))
            .map_err(|error| format!("failed to build project binding import: {error}"))?;
        items.insert(0, import);
        break;
    }
    Ok(prettyplease::unparse(&prelude))
}

pub(crate) fn render_generated_support_import(required: &HashSet<String>) -> String {
    let mut required = required.iter().collect::<Vec<_>>();
    required.sort();
    format!(
        "use crate::__sifr_generated_support::{{{}}};",
        required.into_iter().cloned().collect::<Vec<_>>().join(",")
    )
}

pub(crate) fn import_project_prelude_bindings_in_generated_support(
    prelude_source: &str,
    support_source: &str,
) -> Result<String, String> {
    let prelude = syn::parse_file(prelude_source)
        .map_err(|error| format!("failed to parse generated project prelude: {error}"))?;
    let mut root_bindings = HashSet::new();
    for item in &prelude.items {
        match item {
            syn::Item::Use(item_use) if !matches!(item_use.vis, syn::Visibility::Inherited) => {
                collect_use_binding_names(&item_use.tree, &mut root_bindings);
            }
            _ => {
                if let Some(name) = support_item_name(item) {
                    root_bindings.insert(name);
                }
            }
        }
    }

    let mut support = syn::parse_file(support_source)
        .map_err(|error| format!("failed to parse generated project support: {error}"))?;
    let support_names = crate::stdlib_filter::rust_source_defined_item_names(support_source);
    root_bindings.retain(|name| !support_names.contains(name));
    let mut collector = UnqualifiedRootBindingCollector {
        candidates: &root_bindings,
        referenced: HashSet::new(),
    };
    collector.visit_file(&support);
    if collector.referenced.is_empty() {
        return Ok(support_source.to_string());
    }

    let mut referenced = collector.referenced.into_iter().collect::<Vec<_>>();
    referenced.sort();
    let import =
        syn::parse_str::<syn::ItemUse>(&format!("use crate::{{{}}};", referenced.join(",")))
            .map_err(|error| format!("failed to build generated support root imports: {error}"))?;
    support.items.insert(0, syn::Item::Use(import));
    Ok(prettyplease::unparse(&support))
}

fn collect_use_binding_names(tree: &syn::UseTree, names: &mut HashSet<String>) {
    match tree {
        syn::UseTree::Name(name) => {
            names.insert(name.ident.to_string());
        }
        syn::UseTree::Rename(rename) => {
            names.insert(rename.rename.to_string());
        }
        syn::UseTree::Path(path) => collect_use_binding_names(&path.tree, names),
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_use_binding_names(item, names);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

struct UnqualifiedRootBindingCollector<'a> {
    candidates: &'a HashSet<String>,
    referenced: HashSet<String>,
}

impl<'ast> Visit<'ast> for UnqualifiedRootBindingCollector<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path.leading_colon.is_none()
            && let Some(first) = path.segments.first()
        {
            let name = first.ident.to_string();
            if self.candidates.contains(&name) {
                self.referenced.insert(name);
            }
        }
        visit::visit_path(self, path);
    }
}

fn remove_inherent_impls(items: &mut Vec<syn::Item>) {
    for item in items.iter_mut() {
        if let syn::Item::Mod(module) = item
            && let Some((_, nested)) = &mut module.content
        {
            remove_inherent_impls(nested);
        }
    }
    items.retain(|item| !matches!(item, syn::Item::Impl(item_impl) if item_impl.trait_.is_none()));
}

fn support_item_name(item: &syn::Item) -> Option<String> {
    match item {
        syn::Item::Const(item) => Some(item.ident.to_string()),
        syn::Item::Enum(item) => Some(item.ident.to_string()),
        syn::Item::Fn(item) => Some(item.sig.ident.to_string()),
        syn::Item::Impl(item) => impl_self_type_name(item.self_ty.as_ref()),
        syn::Item::Static(item) => Some(item.ident.to_string()),
        syn::Item::Struct(item) => Some(item.ident.to_string()),
        syn::Item::Trait(item) => Some(item.ident.to_string()),
        syn::Item::Type(item) => Some(item.ident.to_string()),
        syn::Item::Union(item) => Some(item.ident.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        import_project_prelude_bindings_in_generated_support, prune_generated_project_owners,
    };

    #[test]
    fn project_nominal_methods_retain_their_transitive_support_helpers() {
        let prelude = r#"
            mod __sifr_project_nominals {
                pub struct Child;
                impl Child {
                    pub async fn wait(&self) -> i64 {
                        process_async_wait().await
                    }
                }
            }
            pub use __sifr_project_nominals::Child;
        "#;
        let support = r#"
            async fn process_async_wait() -> i64 {
                status_from_parts()
            }
            fn status_from_parts() -> i64 {
                0
            }
            fn unused_support() {}
        "#;
        let body = "async fn run(child: Child) -> i64 { child.wait().await }";

        let (pruned_prelude, pruned_support) =
            prune_generated_project_owners(prelude, support, &[body])
                .expect("generated owners should prune");

        assert!(pruned_prelude.contains("pub async fn wait"));
        assert!(pruned_support.contains("async fn process_async_wait"));
        assert!(pruned_support.contains("fn status_from_parts"));
        assert!(!pruned_support.contains("fn unused_support"));
    }

    #[test]
    fn consumer_method_calls_retain_support_trait_contracts() {
        let support = r#"
            struct Wrapper;
            trait RenderSupport {
                fn render(&self) -> i64;
            }
            impl RenderSupport for Wrapper {
                fn render(&self) -> i64 { 1 }
            }
            fn unused_support() {}
        "#;
        let body = "fn run(value: Wrapper) -> i64 { value.render() }";

        let (_, pruned_support) = prune_generated_project_owners("", support, &[body])
            .expect("generated owners should prune");

        assert!(pruned_support.contains("trait RenderSupport"));
        assert!(pruned_support.contains("impl RenderSupport for Wrapper"));
        assert!(!pruned_support.contains("fn unused_support"));
    }

    #[test]
    fn support_imports_only_referenced_crate_root_reexports() {
        let prelude = "mod nominals { pub struct ParseError; pub struct Other; } pub use nominals::{ParseError, Other};";
        let support = "pub(crate) fn parse() -> ParseError { ParseError }";

        let imported = import_project_prelude_bindings_in_generated_support(prelude, support)
            .expect("generated owners should parse");

        assert!(imported.contains("use crate::ParseError;"));
        assert!(!imported.contains("crate::Other"));
    }

    #[test]
    fn support_does_not_import_already_qualified_root_reexports() {
        let prelude = "mod nominals { pub struct ParseError; } pub use nominals::ParseError;";
        let support = "pub(crate) fn parse() -> crate::ParseError { crate::ParseError }";

        let imported = import_project_prelude_bindings_in_generated_support(prelude, support)
            .expect("generated owners should parse");

        assert_eq!(imported, support);
    }
}
