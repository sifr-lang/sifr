use super::{Result, wire};
use sifr_identity::CompilerIdentity;
use std::path::Path;
pub struct Inputs(sifr_semantic_services::metadata::inputs::Inputs);
impl std::ops::Deref for Inputs {
    type Target = sifr_semantic_services::metadata::inputs::Inputs;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
fn fail(error: impl std::fmt::Display) -> wire::MetadataError {
    wire::MetadataError(error.to_string())
}
impl Inputs {
    pub fn capture(identity: &CompilerIdentity, source_root: &Path, target: &str) -> Result<Self> {
        sifr_semantic_services::metadata::inputs::Inputs::capture(identity, source_root, target)
            .map(Self)
    }
    pub fn produce(&self) -> Result<Vec<u8>> {
        let compiled =
            crate::stdlib::compile_stdlib_sources_with_sysroot(&self.sources, self.sysroot.clone())
                .map_err(|errors| {
                    fail(
                        errors
                            .iter()
                            .map(|e| e.message.as_str())
                            .collect::<Vec<_>>()
                            .join("\n"),
                    )
                })?;
        super::project::project(
            &compiled,
            &self.sources,
            self.compatibility,
            &self.sysroot.paths.stdlib_root,
        )
    }
}
