use super::{DottedModulePath, PackageModuleKey, PackageModuleSource, PackageSourceMap};
use crate::manifest::sifr::ImportRoot;
use std::collections::BTreeMap;

pub(super) fn matching_scoped_import<'a>(
    imports: &'a BTreeMap<ImportRoot, crate::ScopedImport>,
    import_path: &DottedModulePath,
) -> Option<&'a crate::ScopedImport> {
    imports
        .iter()
        .filter(|(root, _)| import_root_matches(root, import_path))
        .max_by_key(|(root, _)| root.0.split('.').count())
        .map(|(_, import)| import)
}

fn import_root_matches(root: &ImportRoot, import_path: &DottedModulePath) -> bool {
    import_path.0 == root.0
        || import_path
            .0
            .strip_prefix(&root.0)
            .is_some_and(|suffix| suffix.starts_with('.'))
}

pub(super) fn remap_import_path(
    import_path: &DottedModulePath,
    import_root: &ImportRoot,
    target_export_root: &ImportRoot,
) -> DottedModulePath {
    if import_path.0 == import_root.0 {
        return DottedModulePath(target_export_root.0.clone());
    }
    let suffix = import_path
        .0
        .strip_prefix(&format!("{}.", import_root.0))
        .unwrap_or_default();
    DottedModulePath(format!("{}.{}", target_export_root.0, suffix))
}

pub(super) fn is_private_dependency_module(
    source_map: &PackageSourceMap,
    module: &PackageModuleSource,
    module_path: &DottedModulePath,
) -> bool {
    if source_map.public_apis.contains_key(&PackageModuleKey {
        package_id: module.package_id.clone(),
        module_path: module_path.clone(),
    }) {
        return false;
    }
    true
}
