use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::manifest::sifr::SifrManifest;
use std::path::Path;

pub fn validate_source_roots_exist(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    package_root: &Path,
    manifest: &SifrManifest,
) -> Result<(), PackageDiagnostic> {
    for source_root in &manifest.source_roots {
        let absolute = package_root.join(&source_root.0);
        if !absolute.is_dir() {
            return Err(PackageDiagnostic::invalid_sifr_manifest(
                cargo_package_id,
                manifest_path.to_path_buf(),
                if manifest.production_schema {
                    "source.root"
                } else {
                    "source.roots"
                },
                format!("`{}` is not a directory", source_root.0.display()),
            ));
        }
    }
    Ok(())
}

pub fn validate_exports_match_sources(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    package_root: &Path,
    manifest: &SifrManifest,
) -> Result<(), PackageDiagnostic> {
    if manifest.production_schema {
        return Ok(());
    }
    for export in &manifest.exports {
        let export_path = export.0.replace('.', "/");
        let found = manifest.source_roots.iter().any(|source_root| {
            let root = package_root.join(&source_root.0);
            root.join(format!("{export_path}.sifr")).is_file()
                || root.join(&export_path).join("__init__.sifr").is_file()
        });
        if !found {
            return Err(PackageDiagnostic::invalid_sifr_manifest(
                cargo_package_id,
                manifest_path.to_path_buf(),
                "exports.modules",
                format!(
                    "export `{}` does not resolve to a .sifr file or package __init__.sifr",
                    export.0
                ),
            ));
        }
    }
    Ok(())
}
