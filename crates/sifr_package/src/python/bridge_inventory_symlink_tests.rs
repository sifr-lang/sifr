use super::bridge_inventory_tests::BridgeFixture;
use super::discover_python_bridge_inventory;
use std::fs;
use std::os::unix::fs::symlink;

#[test]
fn symbolic_link_bridge_source_is_rejected() {
    let fixture = BridgeFixture::new("symlink");
    fixture.write_at("outside.py", "VALUE = 1\n");
    symlink(
        fixture.root.join("outside.py"),
        fixture.root.join("src/python_bridges/linked.py"),
    )
    .expect("create symlink");

    let diagnostics = discover_python_bridge_inventory(&fixture.package)
        .expect_err("bridge source symlinks must fail");
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("symbolic links")));
}

#[test]
fn symbolic_link_bridge_root_is_rejected() {
    let fixture = BridgeFixture::new("symlink_root");
    fixture.write_at("outside_bridge_sources/adapter.py", "VALUE = 1\n");
    fs::remove_dir_all(fixture.root.join("src/python_bridges"))
        .expect("remove canonical bridge root");
    symlink(
        fixture.root.join("outside_bridge_sources"),
        fixture.root.join("src/python_bridges"),
    )
    .expect("create bridge root symlink");

    let diagnostics = discover_python_bridge_inventory(&fixture.package)
        .expect_err("a symlinked bridge root must fail");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains("symbolic links")
            && matches!(
                diagnostic.origin.as_ref(),
                crate::diag::PackageDiagnosticOrigin::PythonBridgeSource { path, .. }
                    if path.ends_with("src/python_bridges")
            )
    }));
}
