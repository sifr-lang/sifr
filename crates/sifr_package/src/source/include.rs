use crate::manifest::sifr::SifrManifest;
use std::path::PathBuf;

#[must_use]
pub fn required_manifest_relative_entries(manifest: &SifrManifest) -> Vec<PathBuf> {
    let mut entries = vec![PathBuf::from("sifr.toml"), manifest.source_root.0.clone()];
    entries.extend(
        manifest
            .sql
            .profiles
            .values()
            .flat_map(|profile| profile.sources.iter().cloned()),
    );
    entries.sort();
    entries.dedup();
    entries
}
