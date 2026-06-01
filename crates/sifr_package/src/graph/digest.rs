use crate::cargo::metadata::NormalizedCargoMetadata;
use crate::graph::derive::SifrPackageGraph;
use crate::imports::source_map::PackageSourceMap;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphDigest {
    pub algorithm: &'static str,
    pub hex: String,
}

#[must_use]
pub fn digest_graph_inputs(metadata: &NormalizedCargoMetadata) -> GraphDigest {
    let canonical = CanonicalMetadata::from(metadata);
    digest_serializable(&canonical)
}

#[must_use]
pub fn digest_package_graph(graph: &SifrPackageGraph) -> GraphDigest {
    let canonical = CanonicalGraph::from(graph);
    digest_serializable(&canonical)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageBuildCacheInputs {
    pub cargo_lock_digest: Option<String>,
    pub cargo_metadata_digest: Option<String>,
    pub package_graph_digest: Option<String>,
    pub package_source_map_digest: Option<String>,
    pub sifr_metadata_digests: BTreeMap<String, String>,
    pub sifr_source_digests: BTreeMap<String, String>,
    pub compiler_version: String,
    pub target: Option<String>,
    pub profile: String,
    pub features: Vec<String>,
    pub selectors: Vec<String>,
}

#[must_use]
pub fn digest_package_build_cache_inputs(inputs: &PackageBuildCacheInputs) -> GraphDigest {
    let canonical = CanonicalPackageBuildCacheInputs::from(inputs);
    digest_serializable(&canonical)
}

#[must_use]
pub fn digest_package_source_map(source_map: &PackageSourceMap) -> GraphDigest {
    let canonical = CanonicalSourceMap::from(source_map);
    digest_serializable(&canonical)
}

fn digest_serializable<T: Serialize>(value: &T) -> GraphDigest {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    GraphDigest {
        algorithm: "fnv1a64",
        hex: format!("{:016x}", fnv1a64(&bytes)),
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[derive(Serialize)]
struct CanonicalMetadata<'a> {
    packages: Vec<CanonicalPackage<'a>>,
    resolve_edges: Vec<CanonicalResolveEdge<'a>>,
    workspace_members: Vec<&'a str>,
    target_directory: String,
    workspace_root: String,
}

#[derive(Serialize)]
struct CanonicalPackage<'a> {
    id: &'a str,
    name: &'a str,
    version: &'a str,
    source: Option<&'a str>,
    manifest_path: String,
    dependency_names: Vec<&'a str>,
    feature_names: Vec<&'a str>,
    has_sifr_metadata: bool,
}

#[derive(Serialize)]
struct CanonicalResolveEdge<'a> {
    from: &'a str,
    dependency_name: &'a str,
    to: &'a str,
}

impl<'a> From<&'a NormalizedCargoMetadata> for CanonicalMetadata<'a> {
    fn from(metadata: &'a NormalizedCargoMetadata) -> Self {
        Self {
            packages: metadata
                .packages
                .values()
                .map(|package| CanonicalPackage {
                    id: &package.id.0,
                    name: &package.name,
                    version: &package.version,
                    source: package.source.as_deref(),
                    manifest_path: package.manifest_path.display().to_string(),
                    dependency_names: package
                        .dependencies
                        .iter()
                        .map(|dependency| dependency.name.as_str())
                        .collect(),
                    feature_names: package.features.keys().map(String::as_str).collect(),
                    has_sifr_metadata: package.sifr_metadata.is_some(),
                })
                .collect(),
            resolve_edges: metadata
                .resolve_edges
                .iter()
                .map(|edge| CanonicalResolveEdge {
                    from: &edge.from.0,
                    dependency_name: &edge.dependency_name,
                    to: &edge.to.0,
                })
                .collect(),
            workspace_members: metadata
                .workspace_members
                .iter()
                .map(|member| member.0.as_str())
                .collect(),
            target_directory: metadata.target_directory.display().to_string(),
            workspace_root: metadata.workspace_root.display().to_string(),
        }
    }
}

#[derive(Serialize)]
struct CanonicalGraph<'a> {
    packages: Vec<CanonicalGraphPackage<'a>>,
    edges: Vec<(&'a str, Vec<&'a str>)>,
    scopes: Vec<CanonicalScope<'a>>,
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

#[derive(Serialize)]
struct CanonicalPackageBuildCacheInputs<'a> {
    cargo_lock_digest: Option<&'a str>,
    cargo_metadata_digest: Option<&'a str>,
    package_graph_digest: Option<&'a str>,
    package_source_map_digest: Option<&'a str>,
    sifr_metadata_digests: Vec<(&'a str, &'a str)>,
    sifr_source_digests: Vec<(&'a str, &'a str)>,
    compiler_version: &'a str,
    target: Option<&'a str>,
    profile: &'a str,
    features: Vec<&'a str>,
    selectors: Vec<&'a str>,
}

impl<'a> From<&'a PackageBuildCacheInputs> for CanonicalPackageBuildCacheInputs<'a> {
    fn from(inputs: &'a PackageBuildCacheInputs) -> Self {
        let mut features = inputs
            .features
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        features.sort_unstable();
        let mut selectors = inputs
            .selectors
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        selectors.sort_unstable();

        Self {
            cargo_lock_digest: inputs.cargo_lock_digest.as_deref(),
            cargo_metadata_digest: inputs.cargo_metadata_digest.as_deref(),
            package_graph_digest: inputs.package_graph_digest.as_deref(),
            package_source_map_digest: inputs.package_source_map_digest.as_deref(),
            sifr_metadata_digests: inputs
                .sifr_metadata_digests
                .iter()
                .map(|(path, digest)| (path.as_str(), digest.as_str()))
                .collect(),
            sifr_source_digests: inputs
                .sifr_source_digests
                .iter()
                .map(|(path, digest)| (path.as_str(), digest.as_str()))
                .collect(),
            compiler_version: &inputs.compiler_version,
            target: inputs.target.as_deref(),
            profile: &inputs.profile,
            features,
            selectors,
        }
    }
}

#[derive(Serialize)]
struct CanonicalGraphPackage<'a> {
    package_id: &'a str,
    cargo_package_id: &'a str,
    sifr_name: &'a str,
    exports: Vec<&'a str>,
}

#[derive(Serialize)]
struct CanonicalScope<'a> {
    package_id: &'a str,
    imports: Vec<(&'a str, &'a str)>,
}

impl<'a> From<&'a SifrPackageGraph> for CanonicalGraph<'a> {
    fn from(graph: &'a SifrPackageGraph) -> Self {
        Self {
            packages: graph
                .packages
                .values()
                .map(|package| CanonicalGraphPackage {
                    package_id: &package.package_id.0,
                    cargo_package_id: &package.cargo_package_id.0,
                    sifr_name: &package.sifr_name.0,
                    exports: package
                        .manifest
                        .exports
                        .iter()
                        .map(|root| root.0.as_str())
                        .collect(),
                })
                .collect(),
            edges: graph
                .cargo_edges
                .iter()
                .map(|(from, to)| {
                    (
                        from.0.as_str(),
                        to.iter().map(|package_id| package_id.0.as_str()).collect(),
                    )
                })
                .collect(),
            scopes: graph
                .direct_dependency_scopes
                .iter()
                .map(|(package_id, scope)| CanonicalScope {
                    package_id: &package_id.0,
                    imports: scope
                        .imports
                        .iter()
                        .map(|(root, import)| (root.0.as_str(), import.package_id.0.as_str()))
                        .collect(),
                })
                .collect(),
        }
    }
}
