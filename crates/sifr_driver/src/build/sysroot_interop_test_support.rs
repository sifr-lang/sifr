use sifr_sysroot::{ResolvedSysroot, SysrootManifest, SysrootPaths};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) struct TempSysroot {
    pub(super) path: PathBuf,
    private_sources: RefCell<HashMap<String, String>>,
}

impl TempSysroot {
    pub(super) fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let path = std::env::temp_dir().join(format!(
            "sifr_sysroot_interop_{label}_{}_{}",
            std::process::id(),
            nonce
        ));
        fs::create_dir_all(path.join("stdlib/_sifr")).expect("private sources");
        fs::create_dir_all(path.join("stdlib/sifr")).expect("public sources");
        fs::create_dir_all(path.join("crates/sifr_stdlib/src")).expect("stdlib crate");
        fs::create_dir_all(path.join("crates/sifr_runtime/src")).expect("runtime crate");
        fs::create_dir_all(path.join(".cargo")).expect("cargo config dir");
        fs::create_dir_all(path.join("vendor")).expect("vendor dir");
        fs::write(
            path.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/sifr_runtime\", \"crates/sifr_stdlib\"]\nresolver = \"3\"\n",
        )
        .expect("workspace manifest");
        fs::write(path.join("Cargo.lock"), "# test lock\n").expect("lockfile");
        fs::write(
            path.join(".cargo/config.toml"),
            "[source.crates-io]\nreplace-with = \"sifr-vendor\"\n[source.sifr-vendor]\ndirectory = \"vendor\"\n",
        )
        .expect("cargo config");
        fs::write(
            path.join("crates/sifr_runtime/Cargo.toml"),
            "[package]\nname = \"sifr_runtime\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
        )
        .expect("runtime manifest");
        fs::write(
            path.join("crates/sifr_runtime/src/lib.rs"),
            "pub mod interop {}\n",
        )
        .expect("runtime lib");
        fs::write(
            path.join("crates/sifr_stdlib/Cargo.toml"),
            "[package]\nname = \"sifr_stdlib\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
        )
        .expect("stdlib manifest");
        fs::write(path.join("sysroot.toml"), "# test sysroot\n").expect("sysroot manifest");
        Self {
            path,
            private_sources: RefCell::new(HashMap::new()),
        }
    }

    pub(super) fn write_private(&self, module: &str, source: &str) {
        fs::write(self.private_path(module), source).expect("private source");
        self.private_sources
            .borrow_mut()
            .insert(module.to_string(), source.to_string());
    }

    pub(super) fn private_source(&self, module: &str) -> String {
        self.private_sources
            .borrow()
            .get(module)
            .cloned()
            .expect("private source should be registered")
    }

    pub(super) fn write_stdlib_crate(&self, source: &str) {
        fs::write(self.path.join("crates/sifr_stdlib/src/lib.rs"), source).expect("stdlib lib");
    }

    pub(super) fn private_path(&self, module: &str) -> PathBuf {
        let filename = module
            .strip_prefix("_sifr.")
            .expect("private module prefix");
        self.path.join(format!("stdlib/_sifr/{filename}.sifr"))
    }

    pub(super) fn resolved(&self) -> ResolvedSysroot {
        ResolvedSysroot {
            root: self.path.clone(),
            manifest: SysrootManifest {
                schema_version: 1,
                sifr_version: "0.0.0-test".to_string(),
                target_triple: "test-target".to_string(),
                built_by_compiler_commit: "test".to_string(),
                sysroot_content_sha256:
                    "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                cargo_lock_sha256:
                    "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            },
            paths: SysrootPaths::from_root(&self.path),
            cargo_lock_content_sha256: "2".repeat(64),
        }
    }
}

impl Drop for TempSysroot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
