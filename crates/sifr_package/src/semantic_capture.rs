//! Captures package/configuration and complete source-inventory observations
//! through the same provider that derives the actual resolver state.
use crate::{PackageSourceMap, SifrPackageGraph};
use sifr_frontend::SourceProvider;
use std::path::Path;

use sifr_frontend::persistence::{CapturedSource, CapturingSourceProvider, Observation, identity};

pub struct CapturedPackageResolution {
    pub graph: SifrPackageGraph,
    pub source_map: PackageSourceMap,
    pub observations: Vec<Observation>,
    pub sources: Vec<CapturedSource>,
    pub cargo_resolution_identity: String,
}
pub fn capture_package_resolution(
    metadata_json: &str,
    lock_path: &Path,
    capture: &mut CapturingSourceProvider<'_>,
) -> Result<CapturedPackageResolution, String> {
    // Canonical JSON preserves resolved feature/target/configuration observations,
    // rather than replacing Cargo's effective graph with a manifest-only guess.
    let metadata_value: serde_json::Value =
        serde_json::from_str(metadata_json).map_err(|error| error.to_string())?;
    let lock = if capture.is_file(lock_path) {
        Some(
            capture
                .read_file(lock_path)
                .map_err(|error| error.to_string())?
                .as_str()
                .to_owned(),
        )
    } else {
        None
    };
    let cargo_resolution_identity =
        identity("resolved-cargo-semantic-v1", &(&metadata_value, lock))
            .map_err(|error| error.to_string())?;
    let metadata =
        crate::parse_metadata_json(metadata_json).map_err(|error| format!("{error:?}"))?;
    let graph =
        crate::derive_package_graph(metadata, capture).map_err(|errors| format!("{errors:?}"))?;
    let source_map =
        PackageSourceMap::build(&graph, capture).map_err(|errors| format!("{errors:?}"))?;
    Ok(CapturedPackageResolution {
        graph,
        source_map,
        observations: capture.observations().to_vec(),
        sources: capture.sources(),
        cargo_resolution_identity,
    })
}
