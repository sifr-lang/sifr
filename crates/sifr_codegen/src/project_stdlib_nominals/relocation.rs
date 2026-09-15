use super::ProjectStdlibNominalPlan;
use crate::stdlib_filter::{
    partition_relocated_rust_items_by_name, rust_source_references_item_name,
};
use std::collections::{BTreeMap, HashSet};

#[cfg(test)]
#[path = "relocation_tests.rs"]
mod tests;

/// Imported contracts are emitted in consumers but implemented exactly once,
/// beside the shared nominal definition. Never retain duplicate local impls.
#[derive(Default)]
pub(crate) struct RelocatedStructuralImplementations {
    by_owner_and_trait: BTreeMap<(String, String), String>,
}

impl RelocatedStructuralImplementations {
    fn collect(&mut self, items: Vec<syn::Item>) {
        for item in items {
            let syn::Item::Impl(implementation) = &item else {
                continue;
            };
            let Some((path, _)) = &implementation.trait_ else {
                continue;
            };
            let segments = path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>();
            if path.leading_colon.is_none()
                || segments.len() != 4
                || segments[..3] != ["sifr_runtime", "interop", "structural"]
                || !matches!(
                    segments[3].as_str(),
                    "StructuralType" | "StructuralConstruct" | "StructuralProject"
                )
            {
                continue;
            }
            let syn::Type::Path(owner) = implementation.self_ty.as_ref() else {
                continue;
            };
            // Keep complete generic self types distinct, not just basenames.
            let key = (quote::quote!(#owner).to_string(), segments[3].clone());
            let source = prettyplease::unparse(&syn::File {
                shebang: None,
                frontmatter: None,
                attrs: Vec::new(),
                items: vec![item],
            });
            if let Some(previous) = self.by_owner_and_trait.get(&key) {
                assert_eq!(
                    previous, &source,
                    "inconsistent structural contract for shared nominal {}",
                    key.0
                );
            } else {
                self.by_owner_and_trait.insert(key, source);
            }
        }
    }

    pub(crate) fn append_to_support(&self, support: &str) -> String {
        if self.by_owner_and_trait.is_empty() {
            return support.to_string();
        }
        std::iter::once(support)
            .chain(self.by_owner_and_trait.values().map(String::as_str))
            .filter(|source| !source.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub(crate) fn relocate_project_stdlib_nominals(
    source: &str,
    module_name: &str,
    plan: &ProjectStdlibNominalPlan,
    crate_root_modules: &HashSet<&str>,
    local_class_rust_names: &HashSet<String>,
    implementations: &mut RelocatedStructuralImplementations,
) -> String {
    let names = plan
        .registry
        .shared_rust_names
        .iter()
        .chain(&plan.registry.crate_root_rust_names)
        .cloned()
        .collect::<HashSet<_>>();
    relocate_project_stdlib_nominals_owned_by(
        source,
        module_name,
        crate_root_modules,
        local_class_rust_names,
        &names,
        implementations,
    )
}

pub(crate) fn relocate_project_stdlib_nominals_owned_by(
    source: &str,
    module_name: &str,
    crate_root_modules: &HashSet<&str>,
    local_class_rust_names: &HashSet<String>,
    owned_names: &HashSet<String>,
    implementations: &mut RelocatedStructuralImplementations,
) -> String {
    if owned_names.is_empty() {
        return source.to_string();
    }
    let relocatable_names = owned_names
        .iter()
        .filter(|name| !local_class_rust_names.contains(*name))
        .collect::<HashSet<_>>();
    let names = relocatable_names.iter().map(|name| name.as_str()).collect();
    let (stripped, relocated) =
        partition_relocated_rust_items_by_name(source, &names, local_class_rust_names);
    implementations.collect(relocated);
    if crate_root_modules.contains(module_name) {
        return stripped;
    }
    let mut ordered_names = owned_names.iter().collect::<Vec<_>>();
    ordered_names.sort();
    let mut imports = String::new();
    for name in ordered_names {
        if local_class_rust_names.contains(name) {
            continue;
        }
        if !rust_source_references_item_name(&stripped, name) {
            continue;
        }
        imports.push_str("use crate::");
        imports.push_str(name);
        imports.push_str(";\n");
    }
    format!("{imports}\n{stripped}")
}
