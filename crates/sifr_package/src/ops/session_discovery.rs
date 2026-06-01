use crate::cargo::metadata::CargoPackageId;
use sifr_frontend::{DiskSourceProvider, SourceProvider};
use std::path::{Path, PathBuf};

pub(super) fn find_manifest(start: &Path) -> Option<PathBuf> {
    let mut provider = DiskSourceProvider::new();
    let mut current = if provider.is_file(start) {
        start.parent()?
    } else {
        start
    };
    loop {
        let candidate = current.join("sifr.toml");
        if provider.is_file(&candidate) {
            return Some(candidate);
        }
        if is_cargo_workspace_root(current, &mut provider) {
            return None;
        }
        current = current.parent()?;
    }
}

fn is_cargo_workspace_root(root: &Path, provider: &mut impl SourceProvider) -> bool {
    let manifest = root.join("Cargo.toml");
    let Ok(source) = provider.read_file(&manifest) else {
        return false;
    };
    source
        .as_str()
        .parse::<toml::Table>()
        .is_ok_and(|table| table.contains_key("workspace"))
}

pub(super) fn session_cargo_id(root: &Path) -> CargoPackageId {
    CargoPackageId(format!("path+file://{}#session", root.display()))
}
