use super::*;

#[test]
fn package_capture_inventory_configuration_and_declared_external_inputs() {
    use sifr_frontend::SourceProvider;
    use sifr_frontend::persistence::{CapturingSourceProvider, Observation};
    let temp = TestWorkspace::new("dx12_capture");
    let app = app_package(&temp);
    let json = metadata_json(&temp.root, &[&app], &[]);
    let lock = temp.root.join("Cargo.lock");
    fs::write(&lock, "# resolved lock A\n").test_unwrap("write lock");
    let mut disk = DiskSourceProvider::new();
    let mut capture = CapturingSourceProvider::new(&mut disk);
    let result = crate::semantic_capture::capture_package_resolution(&json, &lock, &mut capture)
        .test_unwrap("capture actual package resolver");
    assert_eq!(result.graph.packages.len(), 1);
    assert!(
        result
            .observations
            .iter()
            .any(|o| matches!(o, Observation::Directory { .. }))
    );
    assert!(
        result
            .sources
            .iter()
            .any(|source| source.path.ends_with("sifr.toml"))
    );
    let source = result
        .source_map
        .modules
        .values()
        .next()
        .test_unwrap("module");
    let text = capture
        .read_file(&source.file_path)
        .test_unwrap("captured module");
    fs::write(&source.file_path, "CHANGED: int = 9\n").test_unwrap("edit module");
    assert_eq!(
        capture
            .read_file(&source.file_path)
            .test_unwrap("pinned module"),
        text
    );
    assert!(!capture.unchanged());
    let mut fresh = CapturingSourceProvider::new(&mut disk);
    let next = crate::semantic_capture::capture_package_resolution(&json, &lock, &mut fresh)
        .test_unwrap("fresh package resolver");
    assert_eq!(
        result.cargo_resolution_identity,
        next.cargo_resolution_identity
    );
    fs::write(&lock, "# resolved lock B\n").test_unwrap("change lock");
    assert!(!fresh.unchanged());
    let mut fresh = CapturingSourceProvider::new(&mut disk);
    let changed = crate::semantic_capture::capture_package_resolution(&json, &lock, &mut fresh)
        .test_unwrap("changed lock");
    assert_ne!(
        result.cargo_resolution_identity,
        changed.cargo_resolution_identity
    );
    let parent = source.file_path.parent().test_unwrap("parent");
    fs::write(parent.join("added.sifr"), "VALUE: int = 1\n").test_unwrap("new module");
    assert!(!fresh.unchanged());
    let mut fresh = CapturingSourceProvider::new(&mut disk);
    let _ = crate::semantic_capture::capture_package_resolution(&json, &lock, &mut fresh)
        .test_unwrap("capture before source-root change");
    let manifest = app.root.join("sifr.toml");
    let content = fs::read_to_string(&manifest).test_unwrap("manifest");
    fs::write(
        &manifest,
        content.replace("root = \"src\"", "root = \"other\""),
    )
    .test_unwrap("switch inclusion root");
    write_module_under(&app.root, "other", "__init__");
    write_module_under(&app.root, "other", "main");
    assert!(!fresh.unchanged());
    let mut changed_capture = CapturingSourceProvider::new(&mut disk);
    let changed =
        crate::semantic_capture::capture_package_resolution(&json, &lock, &mut changed_capture)
            .test_unwrap("changed source root resolver");
    assert!(
        changed
            .source_map
            .modules
            .values()
            .all(|module| module.file_path.starts_with(app.root.join("other")))
    );
}
