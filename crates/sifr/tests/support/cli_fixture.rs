//! Fresh source projects share the checkout's native dependency owner.
//!
//! Project paths, generated outputs and runtime processes remain independent.
//! Tests that exercise cache isolation still select their own explicit cache.
use std::path::Path;

pub fn project(prefix: &str) -> tempfile::TempDir {
    let parent = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/sifr_cli_fixtures");
    std::fs::create_dir_all(&parent).expect("CLI fixture parent");
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(parent)
        .expect("independent CLI fixture")
}
