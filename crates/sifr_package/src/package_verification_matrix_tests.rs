use crate::test_support::TestUnwrap as _;

use serde_json::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[test]
fn package_fixture_matrix_covers_required_categories() {
    let matrix = fixture_matrix();
    let fixtures = matrix["fixtures"]
        .as_array()
        .test_unwrap("fixture matrix must contain fixtures");
    let categories = fixtures
        .iter()
        .map(|fixture| {
            fixture["category"]
                .as_str()
                .test_unwrap("fixture category must be a string")
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
fn readiness_docs_lock_cargo_backed_boundary_and_future_uv_interop() {
    let root = repo_root();
    let package_docs =
        std::fs::read_to_string(root.join("docs/package_management.md")).test_unwrap("read docs");
    assert!(package_docs.contains("Cargo is the package substrate"));
    assert!(package_docs.contains("uv/Python package coordination are future interop work"));

    let audit = std::fs::read_to_string(root.join("crates/sifr_package/DEPENDENCY_AUDIT.md"))
        .test_unwrap("read audit");
    assert!(audit.contains("No `cargo_metadata` crate"));
    assert!(audit.contains("Source ids are opaque Cargo identifiers"));

    let traceability = std::fs::read_to_string(root.join("crates/sifr_package/TRACEABILITY.md"))
        .test_unwrap("read traceability");
    assert!(traceability.contains("Explicit Non-Port Decisions"));
    assert!(traceability.contains("package_e2e_fixture_matrix.json"));
}

#[test]
fn package_demo_subrepos_cover_required_org_repos() {
    let manifest = demo_repository_manifest();
    let repositories = manifest["repositories"]
        .as_array()
        .test_unwrap("demo manifest must contain repositories");
    let repo_ids = repositories
        .iter()
        .map(|repo| {
            repo["id"]
                .as_str()
                .test_unwrap("demo repository id must be a string")
                .to_string()
        })
        .collect::<BTreeSet<_>>();

    for required in [
        "sifr-demo-json",
        "sifr-demo-http",
        "sifr-demo-test-support",
        "sifr-demo-app",
        "sifr-demo-workspace",
    ] {
        assert!(repo_ids.contains(required), "missing demo repo {required}");
    }

    assert_eq!(manifest["checkout_model"], "git_submodule");
    let repository_root = repo_root().join(
        manifest["repository_root"]
            .as_str()
            .test_unwrap("demo manifest must contain repository_root"),
    );
    for repo in repositories {
        let root = repository_root.join(repo["root"].as_str().test_unwrap("repo root"));
        for rel_path in repo["required_paths"]
            .as_array()
            .test_unwrap("repo must contain required_paths")
        {
            let rel_path = rel_path
                .as_str()
                .test_unwrap("required path must be a string");
            assert!(
                root.join(rel_path).exists(),
                "demo repository {:?} is missing {rel_path}",
                repo["id"]
            );
        }
    }
}

fn fixture_matrix() -> Value {
    let path = repo_root()
        .join("verification/areas/package_management/data/package_e2e_fixture_matrix.json");
    let source = std::fs::read_to_string(path).test_unwrap("read fixture matrix");
    serde_json::from_str(&source).test_unwrap("parse fixture matrix")
}

fn demo_repository_manifest() -> Value {
    let path = repo_root()
        .join("verification/areas/package_management/data/package_demo_repositories.json");
    let source = std::fs::read_to_string(path).test_unwrap("read demo repository manifest");
    serde_json::from_str(&source).test_unwrap("parse demo repository manifest")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .test_unwrap("repo root")
        .to_path_buf()
}
