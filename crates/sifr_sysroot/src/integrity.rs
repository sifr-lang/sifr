//! Explicit package integrity, matching the release archive manifest algorithm.
use crate::{ResolvedSysroot, SysrootMode, sha256_file, sha256_hex};
use std::{
    fs,
    path::{Path, PathBuf},
};
impl ResolvedSysroot {
    pub fn verify_integrity(&self) -> Result<(), String> {
        if self.mode() == SysrootMode::SourceTreeDevelopment {
            return Err("package integrity requires an installed toolchain".into());
        }
        fn collect(path: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
            let metadata = fs::symlink_metadata(path)?;
            if metadata.file_type().is_symlink() {
                return Err(std::io::Error::other(
                    "installed generation contains a symlink",
                ));
            }
            if metadata.is_dir() {
                for entry in fs::read_dir(path)? {
                    collect(&entry?.path(), files)?;
                }
            } else if metadata.is_file() {
                files.push(path.to_owned());
            } else {
                return Err(std::io::Error::other(
                    "installed generation contains a special file",
                ));
            }
            Ok(())
        }
        let mut files = Vec::new();
        for path in [
            "Cargo.toml",
            "Cargo.lock",
            ".cargo/config.toml",
            "crates",
            "lib",
            "vendor",
        ] {
            collect(&self.root.join(path), &mut files).map_err(|e| e.to_string())?;
        }
        // Release manifests use LC_ALL=C sorting of whole path strings.
        // Path::Ord compares components and orders a/z before a.rs instead.
        files.sort_by(|left, right| left.as_os_str().cmp(right.as_os_str()));
        let mut manifest = String::new();
        for path in files {
            let relative = path.strip_prefix(&self.root).map_err(|e| e.to_string())?;
            manifest.push_str(relative.to_str().ok_or("non-UTF8 package path")?);
            manifest.push('\n');
            manifest.push_str(&sha256_file(&path).map_err(|e| e.to_string())?);
            manifest.push('\n');
        }
        let actual = sha256_hex(manifest.as_bytes());
        if actual != self.manifest.sysroot_content_sha256 {
            return Err(format!(
                "installed package integrity mismatch: expected {}, got {actual}; reinstall the selected toolchain",
                self.manifest.sysroot_content_sha256
            ));
        }
        if sha256_file(&self.paths.cargo_lock).map_err(|e| e.to_string())?
            != self.manifest.cargo_lock_sha256
        {
            return Err(
                "installed Cargo.lock integrity mismatch; reinstall the selected toolchain".into(),
            );
        }
        let descriptor: serde_json::Value = serde_json::from_slice(
            &fs::read(self.root.join("lib/sifr/stdlib.metadata.json"))
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let binary_digest = sha256_file(&self.root.join("bin/sifr")).map_err(|e| e.to_string())?;
        if descriptor["compiler_binary_sha256"].as_str() != Some(binary_digest.as_str()) {
            return Err(
                "installed compiler integrity mismatch; reinstall the selected toolchain".into(),
            );
        }
        Ok(())
    }
}
