//! Deterministic byte inventory and Cargo watches for declared input roots.
#![allow(clippy::print_stdout)]
use crate::IdentityEncoder;
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};

fn excluded(name: &std::ffi::OsStr) -> bool {
    matches!(name.to_str(), Some(".git" | "target" | "__pycache__"))
}

/// A calculation owns one union of declared roots. Shared package inputs are
/// read once, while each invocation still observes their current bytes.
#[derive(Default)]
pub(super) struct InputInventory {
    roots: BTreeSet<PathBuf>,
}

impl InputInventory {
    pub(super) fn add(&mut self, path: &Path) {
        if self.roots.iter().any(|root| covers(root, path)) {
            return;
        }
        self.roots.retain(|root| !covers(path, root));
        self.roots.insert(path.to_path_buf());
    }

    pub(super) fn encode(&self, root: &Path, hash: &mut IdentityEncoder) -> io::Result<()> {
        for path in &self.roots {
            visit(root, path, hash)?;
        }
        Ok(())
    }
}

fn covers(root: &Path, path: &Path) -> bool {
    path.strip_prefix(root).is_ok_and(|relative| {
        // Explicit declarations can include paths omitted by an ancestor's
        // recursive scan. Keep those roots rather than silently dropping input.
        !relative.components().any(|part| excluded(part.as_os_str()))
    })
}

pub(super) fn visit(root: &Path, path: &Path, hash: &mut IdentityEncoder) -> io::Result<()> {
    // Cargo recursively watches directory roots, including additions/removals and symlinks.
    // Emitting every descendant repeats the same scan for each ancestor and bloats Cargo metadata.
    println!("cargo:rerun-if-changed={}", path.display());
    hash_tree(root, path, hash)
}

pub(super) fn hash_tree(root: &Path, path: &Path, hash: &mut IdentityEncoder) -> io::Result<()> {
    if path.is_dir() {
        let mut children = fs::read_dir(path)?
            .map(|e| e.map(|e| e.path()))
            .collect::<io::Result<Vec<_>>>()?;
        children.sort();
        for child in children {
            if child.file_name().is_some_and(excluded) {
                continue;
            }
            hash_tree(root, &child, hash)?;
        }
    } else if path.is_file() {
        let relative = path.strip_prefix(root).map_err(io::Error::other)?;
        let name = relative
            .to_str()
            .ok_or_else(|| io::Error::other("non-UTF8 compiler input"))?;
        hash.field(name, &fs::read(path)?);
    }
    Ok(())
}
