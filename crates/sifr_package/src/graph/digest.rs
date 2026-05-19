use crate::cargo::metadata::NormalizedCargoMetadata;
use crate::graph::derive::SifrPackageGraph;
use serde::Serialize;

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
