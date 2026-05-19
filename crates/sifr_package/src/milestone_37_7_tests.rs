use serde_json::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[test]
fn phase37_fixture_matrix_covers_required_closeout_categories() {
    let matrix = fixture_matrix();
    let fixtures = matrix["fixtures"]
        .as_array()
        .expect("fixture matrix must contain fixtures");
    let categories = fixtures
        .iter()
        .map(|fixture| {
            fixture["category"]
                .as_str()
                .expect("fixture category must be a string")
                .to_string()
        })
        .collect::<BTreeSet<_>>();

    for required in [
        "pure_sifr_cargo_package",
        "rust_backed_sifr_package",
        "workspace_selection",
        "path_dependency",
        "git_dependency",
        "registry_dependency",
        "multiple_version_graph",
        "alias_imports",
        "publishing",
    ] {
        assert!(
            categories.contains(required),
            "missing fixture category {required}"
        );
    }

    for fixture in fixtures {
        assert!(
            fixture["coverage"]
                .as_array()
                .is_some_and(|coverage| !coverage.is_empty()),
            "fixture {:?} must record concrete coverage",
            fixture["category"]
        );
    }
}

#[test]
fn closeout_docs_lock_cargo_backed_boundary_and_future_uv_interop() {
    let root = repo_root();
    let package_docs =
        std::fs::read_to_string(root.join("docs/package_management.md")).expect("read docs");
    assert!(package_docs.contains("Cargo the package substrate"));
    assert!(package_docs.contains("uv/Python package coordination are future interop work"));

    let audit = std::fs::read_to_string(root.join("crates/sifr_package/DEPENDENCY_AUDIT.md"))
        .expect("read audit");
    assert!(audit.contains("No `cargo_metadata` crate"));
    assert!(audit.contains("Source ids are opaque Cargo identifiers"));

    let traceability = std::fs::read_to_string(root.join("crates/sifr_package/TRACEABILITY.md"))
        .expect("read traceability");
    assert!(traceability.contains("Explicit Non-Port Decisions"));
    assert!(traceability.contains("phase37_e2e_fixture_matrix.json"));
}

fn fixture_matrix() -> Value {
    let path = repo_root().join("verification/package_management/phase37_e2e_fixture_matrix.json");
    let source = std::fs::read_to_string(path).expect("read fixture matrix");
    serde_json::from_str(&source).expect("parse fixture matrix")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("repo root")
        .to_path_buf()
}
