use crate::manifest::sifr::SifrManifest;
use std::path::PathBuf;

#[must_use]
pub fn required_manifest_relative_entries(manifest: &SifrManifest) -> Vec<PathBuf> {
    let mut entries = vec![PathBuf::from("sifr.toml")];
    entries.extend(manifest.source_roots.iter().map(|root| root.0.clone()));
    entries.sort();
    entries.dedup();
    entries
}
