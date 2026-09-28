use super::*;
use sifr_frontend::{DiskSourceProvider, SourceProvider, persistence::CapturingSourceProvider};
use std::fs;

#[test]
fn changed_source_or_metadata_rejects_restored_check() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("main.sifr");
    fs::write(&file, "def main() -> None:\n    pass\n").unwrap();
    let compiler = CompilerContext::for_test();
    let inputs = manifestless_inputs(&compiler, "metadata-one", &file).unwrap();
    let mut disk = DiskSourceProvider::new();
    let mut capture = CapturingSourceProvider::new(&mut disk);
    capture.read_file(&file).unwrap();
    let record =
        sifr_frontend::persistence::CompletedCheck::capture(&file, inputs.clone(), &capture, &[])
            .unwrap();
    assert!(record.validate(&inputs, &mut DiskSourceProvider::new()));
    let different_metadata = manifestless_inputs(&compiler, "metadata-two", &file).unwrap();
    assert!(!record.validate(&different_metadata, &mut DiskSourceProvider::new()));
    let mut different_product = inputs.clone();
    different_product.compiler = "different-product".into();
    assert!(!record.validate(&different_product, &mut DiskSourceProvider::new()));
    fs::write(&file, "def main() -> None:\n    print(1)\n").unwrap();
    assert!(!record.validate(&inputs, &mut DiskSourceProvider::new()));
}

#[test]
fn corrupt_or_missing_check_is_a_miss() {
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("workspace");
    let cache = root.path().join("cache");
    fs::create_dir(&workspace).unwrap();
    assert!(saved_checks::load_generation(&cache, &workspace, &"a".repeat(64)).is_err());
    let generation = root.path().join("generation");
    fs::create_dir(&generation).unwrap();
    fs::write(
        generation.join("manifest.json"),
        b"{\"schema\":2,\"records\":[]}",
    )
    .unwrap();
    assert!(
        saved_checks::read_complete_generation(
            &generation,
            &"b".repeat(64),
            &"a".repeat(64),
            &"c".repeat(64),
        )
        .is_err()
    );
}

#[test]
fn failed_preview_has_no_partial_output() {
    let compiler = CompilerContext::for_test();
    let result = generated_rust_preview(&compiler, "def main(:\n    pass\n");
    assert!(
        matches!(result, PreviewResult::Unavailable { diagnostic_count } if diagnostic_count > 0)
    );
}
