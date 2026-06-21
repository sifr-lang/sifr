use super::digest::{digest_serializable, GraphDigest};
use crate::imports::source_map::PackageSourceMap;
use serde::Serialize;

#[must_use]
pub fn digest_package_source_map(source_map: &PackageSourceMap) -> GraphDigest {
    let canonical = CanonicalSourceMap::from(source_map);
    digest_serializable(&canonical)
}

#[derive(Serialize)]
struct CanonicalSourceMap<'a> {
    roots: Vec<CanonicalSourceRoot<'a>>,
    modules: Vec<CanonicalSourceModule<'a>>,
    ambiguous_modules: Vec<CanonicalSourceModule<'a>>,
}

#[derive(Serialize)]
struct CanonicalSourceRoot<'a> {
    package_id: &'a str,
    import_root: &'a str,
    path: String,
}

#[derive(Serialize)]
struct CanonicalSourceModule<'a> {
    package_id: &'a str,
    module_path: &'a str,
    file_path: String,
    source_root: String,
}

impl<'a> From<&'a PackageSourceMap> for CanonicalSourceMap<'a> {
    fn from(source_map: &'a PackageSourceMap) -> Self {
        Self {
            roots: source_map
                .roots
                .iter()
                .map(|((package_id, import_root), path)| CanonicalSourceRoot {
                    package_id: &package_id.0,
                    import_root: &import_root.0,
                    path: path.display().to_string(),
                })
                .collect(),
            modules: source_map
                .modules
                .values()
                .map(|module| CanonicalSourceModule {
                    package_id: &module.package_id.0,
                    module_path: &module.module_path.0,
                    file_path: module.file_path.display().to_string(),
                    source_root: module.source_root.display().to_string(),
                })
                .collect(),
            ambiguous_modules: source_map
                .ambiguous_modules
                .values()
                .flat_map(|modules| modules.iter())
                .map(|module| CanonicalSourceModule {
                    package_id: &module.package_id.0,
                    module_path: &module.module_path.0,
                    file_path: module.file_path.display().to_string(),
                    source_root: module.source_root.display().to_string(),
                })
                .collect(),
        }
    }
}
