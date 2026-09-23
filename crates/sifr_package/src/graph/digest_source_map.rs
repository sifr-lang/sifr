use super::digest::{GraphDigest, digest_serializable};
use crate::imports::source_map::PackageSourceMap;
use serde::Serialize;
use std::fs;
use std::io;

pub fn digest_package_source_map(source_map: &PackageSourceMap) -> Result<GraphDigest, String> {
    let canonical = CanonicalSourceMap::from(source_map);
    digest_serializable("package-source-map", &canonical)
}

/// Digest the bytes of every discovered source module with its package/module
/// identity. The canonical serialized records preserve field boundaries.
pub fn digest_package_source_snapshot(source_map: &PackageSourceMap) -> io::Result<GraphDigest> {
    let modules = source_map
        .modules
        .values()
        .map(|module| {
            Ok(CanonicalSourceContents {
                package_id: &module.package_id.0,
                module_path: &module.module_path.0,
                contents: fs::read(&module.file_path)?,
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    let ambiguous_modules = source_map
        .ambiguous_modules
        .values()
        .flat_map(|modules| modules.iter())
        .map(|module| {
            Ok(CanonicalSourceContents {
                package_id: &module.package_id.0,
                module_path: &module.module_path.0,
                contents: fs::read(&module.file_path)?,
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    digest_serializable(
        "package-source-snapshot",
        &CanonicalSourceSnapshot {
            source_map: CanonicalSourceMap::from(source_map),
            modules,
            ambiguous_modules,
        },
    )
    .map_err(io::Error::other)
}

#[derive(Serialize)]
struct CanonicalSourceSnapshot<'a> {
    source_map: CanonicalSourceMap<'a>,
    modules: Vec<CanonicalSourceContents<'a>>,
    ambiguous_modules: Vec<CanonicalSourceContents<'a>>,
}

#[derive(Serialize)]
struct CanonicalSourceMap<'a> {
    roots: Vec<CanonicalSourceRoot<'a>>,
    modules: Vec<CanonicalSourceModule<'a>>,
    ambiguous_modules: Vec<CanonicalSourceModule<'a>>,
    public_apis: Vec<(
        &'a str,
        &'a str,
        &'a crate::imports::namespace_api::NamespaceApi,
    )>,
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
    cargo_package_id: &'a str,
    module_path: &'a str,
    file_path: String,
    source_root: String,
}

#[derive(Serialize)]
struct CanonicalSourceContents<'a> {
    package_id: &'a str,
    module_path: &'a str,
    contents: Vec<u8>,
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
                    cargo_package_id: &module.cargo_package_id.0,
                    module_path: &module.module_path.0,
                    file_path: module.file_path.display().to_string(),
                    source_root: module.source_root.display().to_string(),
                })
                .collect(),
            public_apis: source_map
                .public_apis
                .iter()
                .map(|(key, api)| (key.package_id.0.as_str(), key.module_path.0.as_str(), api))
                .collect(),
            ambiguous_modules: source_map
                .ambiguous_modules
                .values()
                .flat_map(|modules| modules.iter())
                .map(|module| CanonicalSourceModule {
                    package_id: &module.package_id.0,
                    cargo_package_id: &module.cargo_package_id.0,
                    module_path: &module.module_path.0,
                    file_path: module.file_path.display().to_string(),
                    source_root: module.source_root.display().to_string(),
                })
                .collect(),
        }
    }
}
