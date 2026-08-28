#![no_main]

use libfuzzer_sys::fuzz_target;
use sifr_frontend::DiskSourceProvider;
use sifr_package::{derive_package_graph, parse_metadata_json};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

thread_local! {
    static FIXTURE: RefCell<Option<Fixture>> = RefCell::new(create_fixture());
}

struct Fixture(PathBuf);

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fuzz_target!(|data: &[u8]| {
    FIXTURE.with(|fixture| {
        let fixture = fixture.borrow();
        let Some(fixture) = fixture.as_ref() else {
            return;
        };
        let json = metadata_json(&fixture.0, data);
        if let Ok(metadata) = parse_metadata_json(&json) {
            let _ = derive_package_graph(metadata, &mut DiskSourceProvider::new());
        }
    });
});

fn create_fixture() -> Option<Fixture> {
    let root = std::env::temp_dir().join(format!(
        "sifr_project_graph_fuzz_{}_{}",
        std::process::id(),
        NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let fixture = Fixture(root);
    prepare_fixture(&fixture.0).ok()?;
    Some(fixture)
}

fn prepare_fixture(root: &Path) -> std::io::Result<()> {
    fs::create_dir_all(root.join("src"))?;
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fuzz-package\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n",
    )?;
    fs::write(
        root.join("sifr.toml"),
        "[package]\nname = \"fuzz_package\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n",
    )?;
    fs::write(
        root.join("src/lib.rs"),
        "// Pure Sifr package marker. Sifr source lives in the sifr.toml source root.\n",
    )?;
    fs::write(root.join("src/__init__.sifr"), "")
}

fn metadata_json(root: &Path, data: &[u8]) -> String {
    let extra_feature = if data.first().copied().unwrap_or_default() & 1 == 0 {
        "{}"
    } else {
        r#"{"generated":[]}"#
    };
    format!(
        r#"{{
            "packages":[{{
                "id":"path+file://{root}#fuzz-package@0.1.0",
                "name":"fuzz-package",
                "version":"0.1.0",
                "source":null,
                "manifest_path":"{root}/Cargo.toml",
                "dependencies":[],
                "targets":[{{
                    "name":"fuzz_package",
                    "kind":["lib"],
                    "crate_types":["lib"],
                    "src_path":"{root}/src/lib.rs"
                }}],
                "features":{extra_feature},
                "metadata":{{"sifr":{{"manifest":"sifr.toml"}}}}
            }}],
            "workspace_members":["path+file://{root}#fuzz-package@0.1.0"],
            "target_directory":"{root}/target",
            "workspace_root":"{root}"
        }}"#,
        root = root.display(),
    )
}
