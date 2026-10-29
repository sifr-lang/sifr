//! Independent source fixtures share one real Git owner outside the product tree.
//!
//! Keeping the owner outside the checkout prevents standalone fixtures from
//! discovering the product's Cargo/Sifr manifests. Only native dependencies are
//! reused: project paths, outputs and runtime processes remain independent.
use sha2::{Digest as _, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

fn owner() -> &'static Path {
    static OWNER: OnceLock<PathBuf> = OnceLock::new();
    OWNER.get_or_init(|| {
        let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
            .canonicalize()
            .expect("CLI source owner");
        let key = format!("{:x}", Sha256::digest(checkout.as_os_str().as_encoded_bytes()));
        let parent = std::env::temp_dir().join("sifr-cli-fixture-owners").join(key);
        std::fs::create_dir_all(&parent).expect("CLI fixture owner directory");
        let parent = parent.canonicalize().expect("canonical CLI fixture owner");
        let status = Command::new("git")
            .args(["init", "--quiet", "--template=", "--initial-branch=sifr-fixtures"])
            .arg(&parent)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .status()
            .expect("initialize independent fixture repository");
        assert!(status.success(), "initialize independent fixture repository");
        parent
    })
}

pub fn project(prefix: &str) -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(owner())
        .expect("independent CLI fixture")
}
