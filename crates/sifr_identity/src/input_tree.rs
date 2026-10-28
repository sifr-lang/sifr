//! Deterministic byte inventory and Cargo watches for declared input roots.
#![allow(clippy::print_stdout)]
use crate::IdentityEncoder;
use std::{fs, io, path::Path};

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
            let name = child.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if matches!(name, ".git" | "target" | "__pycache__") {
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
