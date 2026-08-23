use super::digest::{GraphDigest, digest_serializable};
use crate::python::{PythonEnvironmentProbe, PythonEnvironmentProbeRequest};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageBuildCacheInputs {
    pub cargo_lock_digest: Option<String>,
    pub cargo_metadata_digest: Option<String>,
    pub package_graph_digest: Option<String>,
    pub package_source_map_digest: Option<String>,
    pub python_probe_digest: Option<String>,
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
pub fn digest_python_environment_probe(
    request: &PythonEnvironmentProbeRequest,
    probe: &PythonEnvironmentProbe,
) -> GraphDigest {
    let canonical = CanonicalPythonEnvironmentProbe { request, probe };
    digest_serializable(&canonical)
}

/// Stable identity for authoring artifacts that depend on the selected
/// interpreter, ABI, and locked environment but not on the particular set of
/// import roots currently reachable from one Sifr entrypoint.
#[must_use]
pub fn digest_python_authoring_environment_probe(
    request: &PythonEnvironmentProbeRequest,
    probe: &PythonEnvironmentProbe,
) -> GraphDigest {
    let mut request = request.clone();
    request.required_imports.clear();
    request.declared_imports.clear();
    request.native_imports.clear();
    let mut probe = probe.clone();
    probe.imports.clear();
    probe.native_imports.clear();
    digest_python_environment_probe(&request, &probe)
}

#[derive(Serialize)]
struct CanonicalPackageBuildCacheInputs<'a> {
    cargo_lock_digest: Option<&'a str>,
    cargo_metadata_digest: Option<&'a str>,
    package_graph_digest: Option<&'a str>,
    package_source_map_digest: Option<&'a str>,
    python_probe_digest: Option<&'a str>,
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
            python_probe_digest: inputs.python_probe_digest.as_deref(),
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
struct CanonicalPythonEnvironmentProbe<'a> {
    request: &'a PythonEnvironmentProbeRequest,
    probe: &'a PythonEnvironmentProbe,
}
