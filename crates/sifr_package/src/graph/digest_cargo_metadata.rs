use super::digest::{GraphDigest, digest_serializable};
use crate::cargo::metadata::NormalizedCargoMetadata;
use serde::Serialize;

#[must_use]
pub fn digest_graph_inputs(metadata: &NormalizedCargoMetadata) -> GraphDigest {
    let canonical = CanonicalMetadata::from(metadata);
    digest_serializable(&canonical)
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
    links: Option<&'a str>,
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
                    links: package.links.as_deref(),
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
