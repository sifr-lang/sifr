//! Eligibility is decided from the already resolved package authority. Never
//! infer an empty external inventory from the absence of a cached observation.
use crate::PackageEntrypoint;

pub(super) fn identity(entrypoint: &PackageEntrypoint) -> Option<String> {
    if entrypoint.python_runtime.is_some()
        || !entrypoint
            .graph
            .packages
            .contains_key(&entrypoint.package_id)
        || entrypoint
            .graph
            .backend_crates
            .values()
            .any(|crates| !crates.is_empty())
        || entrypoint.graph.packages.values().any(|package| {
            let manifest = &package.manifest;
            manifest.declares_rust_backend()
                || !manifest.compiler_components.is_empty()
                || manifest.sql != Default::default()
                || manifest.python != Default::default()
        })
    {
        return None;
    }
    // Package owners already resolved manifests, source inclusion, versions,
    // aliases, trust and lock/feature selection for this invocation. Bind the
    // complete deterministic projections too, not only their legacy short hashes.
    sifr_frontend::persistence::identity(
        "resolved-pure-package-v1",
        &(
            &entrypoint.package_id.0,
            sifr_package::digest_package_graph(&entrypoint.graph)
                .ok()?
                .hex,
            sifr_package::digest_package_source_map(&entrypoint.source_map)
                .ok()?
                .hex,
            format!("{:?}", (&entrypoint.graph, &entrypoint.source_map)),
            entrypoint.lock_mode.as_str(),
        ),
    )
    .ok()
}
