use crate::manifest::sifr::{PackageSourceRoot, SifrManifest};
use std::path::{Path, PathBuf};

#[must_use]
pub fn source_root_paths(package_root: &Path, manifest: &SifrManifest) -> Vec<PathBuf> {
    manifest
        .source_roots
        .iter()
        .map(|PackageSourceRoot(root)| package_root.join(root))
        .collect()
}
