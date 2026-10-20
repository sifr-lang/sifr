use ruff_text_size::{TextRange, TextSize};

pub(super) fn span() -> TextRange {
    TextRange::new(TextSize::from(0), TextSize::from(18))
}

/// Owned native inputs for tests that require complete Cargo qualification.
/// Synthetic planning/error fixtures deliberately do not use this helper.
pub(super) struct CargoFixture {
    root: tempfile::TempDir,
}

impl CargoFixture {
    pub(super) fn new(name: &str, source: &str) -> Self {
        let fixture = Self {
            root: tempfile::Builder::new()
                .prefix("sifr-interop-fixture-")
                .tempdir()
                .expect("create owned Cargo fixture"),
        };
        fixture.write("Cargo.toml", &format!(
            "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[workspace]\n"
        ));
        fixture.write("src/lib.rs", source);
        fixture
    }

    pub(super) fn bridge() -> Self {
        let fixture = Self::new("sifr-app", "pub mod bridges;\n");
        fixture.write("src/bridges/mod.rs", concat!(
            "pub fn hash() {}\npub fn codec() {}\npub fn user_noop() {}\n",
            "pub mod consumer { pub struct Consumer; impl Consumer { pub fn poll(&self) {} } }\n",
        ));
        fixture
    }

    pub(super) fn path(&self) -> &std::path::Path {
        self.root.path()
    }

    pub(super) fn write(&self, relative: &str, content: &str) {
        let path = self.path().join(relative);
        std::fs::create_dir_all(path.parent().expect("fixture parent"))
            .expect("create fixture directory");
        std::fs::write(path, content).expect("write fixture input");
    }

    pub(super) fn bind(
        &self,
        mut context: super::rust_interop::PackageRustInteropContext,
    ) -> super::rust_interop::PackageRustInteropContext {
        let package = context
            .graph
            .packages
            .get_mut(&context.package_id)
            .expect("fixture package");
        package.package_root = self.path().to_path_buf();
        package.sifr_manifest = self.path().join("sifr.toml");
        context
    }
}
