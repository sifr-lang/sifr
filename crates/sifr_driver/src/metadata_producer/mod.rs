//! Driver compatibility exports for the lower metadata producer.
pub use sifr_compiler_services::metadata::{
    PreparedMetadata, ProjectInterfacePayload, ProjectModuleInput, ProjectModuleReferences,
    ProjectTypedArtifact, development_metadata_path, encode_project_results,
    validate_development_metadata,
};
use sifr_identity::CompilerIdentity;
#[cfg(test)]
use sifr_sysroot::metadata as wire;
use std::path::Path;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
pub fn ensure_development_metadata(
    identity: &CompilerIdentity,
    source_root: &Path,
    target: &str,
    cache: &Path,
    cancel: &AtomicBool,
) -> sifr_sysroot::metadata::Result<Arc<PreparedMetadata>> {
    let cancelled = || cancel.load(Ordering::Relaxed);
    sifr_compiler_services::metadata::ensure_development_metadata(
        identity,
        source_root,
        target,
        cache,
        &cancelled,
    )
}
#[cfg(test)]
pub(crate) mod ensure {
    pub(crate) use sifr_compiler_services::metadata::ensure::Stage;
    pub(crate) fn ensure_with_hook(
        identity: &sifr_identity::CompilerIdentity,
        source_root: &std::path::Path,
        target: &str,
        cache: &std::path::Path,
        cancel: &std::sync::atomic::AtomicBool,
        hook: impl Fn(Stage) -> sifr_sysroot::metadata::Result<()>,
    ) -> sifr_sysroot::metadata::Result<std::sync::Arc<super::PreparedMetadata>> {
        let cancelled = || cancel.load(std::sync::atomic::Ordering::Relaxed);
        sifr_compiler_services::metadata::ensure::ensure_with_hook(
            identity,
            source_root,
            target,
            cache,
            &cancelled,
            hook,
        )
    }
}
#[cfg(test)]
pub(crate) mod production {
    pub(crate) use sifr_compiler_services::metadata::production::Inputs;
}
#[cfg(test)]
#[path = "project_results/tests.rs"]
mod project_results_tests;
#[cfg(test)]
mod tests;
