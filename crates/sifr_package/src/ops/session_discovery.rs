use crate::cargo::metadata::CargoPackageId;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn find_manifest(start: &Path) -> Option<PathBuf> {
    let mut current = if start.is_file() {
        start.parent()?
    } else {
        start
    };
    loop {
        let candidate = current.join("sifr.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        if is_cargo_workspace_root(current) {
            return None;
        }
        current = current.parent()?;
    }
}

fn is_cargo_workspace_root(root: &Path) -> bool {
    let manifest = root.join("Cargo.toml");
    let Ok(source) = fs::read_to_string(manifest) else {
        return false;
    };
    source
        .parse::<toml::Table>()
        .is_ok_and(|table| table.contains_key("workspace"))
}

pub(super) fn session_cargo_id(root: &Path) -> CargoPackageId {
    CargoPackageId(format!("path+file://{}#session", root.display()))
}
