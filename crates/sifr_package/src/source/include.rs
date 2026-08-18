use crate::manifest::sifr::SifrManifest;
use std::path::PathBuf;

#[must_use]
pub fn required_manifest_relative_entries(manifest: &SifrManifest) -> Vec<PathBuf> {
    vec![PathBuf::from("sifr.toml"), manifest.source_root.0.clone()]
}
