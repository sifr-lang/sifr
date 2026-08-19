use crate::manifest::sifr::SifrManifest;
use std::path::{Path, PathBuf};

#[must_use]
pub fn source_root_path(package_root: &Path, manifest: &SifrManifest) -> PathBuf {
    package_root.join(&manifest.source_root.0)
}
