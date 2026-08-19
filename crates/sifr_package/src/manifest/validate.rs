use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::manifest::sifr::SifrManifest;
use sifr_frontend::SourceProvider;
use std::path::Path;

pub fn validate_source_root_exists(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    package_root: &Path,
    manifest: &SifrManifest,
    provider: &mut impl SourceProvider,
) -> Result<(), PackageDiagnostic> {
    let source_root = &manifest.source_root;
    let absolute = package_root.join(&source_root.0);
    if !provider.is_dir(&absolute) {
        return Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            "source.root",
            format!("`{}` is not a directory", source_root.0.display()),
        ));
    }
    Ok(())
}
