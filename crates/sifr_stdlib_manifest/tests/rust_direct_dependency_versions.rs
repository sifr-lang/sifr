mod support;
use support::TestUnwrap as _;
#[allow(dead_code)]
#[path = "support/cargo_edges.rs"]
mod cargo_edges;
#[allow(dead_code)]
#[path = "support/cargo_inventory.rs"]
mod cargo_inventory;

#[path = "support/registry_audit.rs"]
mod registry_audit;

use std::collections::BTreeSet;
const AUDIT: &str = include_str!("data/rust_latest_stable_registry.json");

#[test]
fn every_maintained_rust_direct_declaration_matches_the_registry_audit() {
    let audit: serde_json::Value = serde_json::from_str(AUDIT).test_unwrap("audit JSON");
    assert_eq!(audit["schema_version"].as_u64(), Some(2));
    assert_eq!(audit["audited_at"].as_str(), Some("2026-09-09"));
    assert_eq!(
        audit["source"].as_str(),
        Some("https://index.crates.io/{prefix}/{crate}")
    );
    let manifests = cargo_inventory::maintained_paths("Cargo.toml");
    assert_eq!(
        audit["maintained_manifests"].as_u64(),
        Some(manifests.len() as u64)
    );
    let declarations = cargo_inventory::declarations();
    assert_eq!(
        audit["direct_declarations"].as_u64(),
        Some(declarations.len() as u64)
    );
    registry_audit::exact_packages(&audit, &declarations).test_unwrap("exact package inventory");
    let releases = registry_audit::releases(&audit).test_unwrap("registry releases");
    let mut expected = Vec::new();
    let mut stale = Vec::new();
    for (name, release) in releases {
        assert!(
            release["source_url"]
                .as_str()
                .is_some_and(|url| url.starts_with("https://index.crates.io/"))
        );
        assert!(
            release["response_sha256"]
                .as_str()
                .is_some_and(registry_audit::checksum)
        );
        let latest = release["latest_stable"]
            .as_str()
            .test_unwrap("latest version");
        for row in release["declarations"]
            .as_array()
            .test_unwrap("declaration owner array")
        {
            let declaration = cargo_inventory::Declaration {
                path: row["path"].as_str().test_unwrap("owner path").into(),
                table: row["table"].as_str().test_unwrap("owner table").into(),
                alias: row["alias"].as_str().test_unwrap("owner alias").into(),
                package: row["package"].as_str().test_unwrap("owner package").into(),
                requirement: row["requirement"]
                    .as_str()
                    .test_unwrap("owner requirement")
                    .into(),
            };
            assert_eq!(declaration.package, name, "audit owner identity");
            if !registry_audit::requirement_matches_latest(&declaration.requirement, latest) {
                stale.push(format!(
                    "{} {} {}: {} selects {:?}; official latest is {latest}",
                    declaration.path,
                    declaration.table,
                    declaration.alias,
                    name,
                    declaration.requirement
                ));
            }
            expected.push(declaration);
        }
    }
    expected.sort();
    assert_eq!(
        declarations, expected,
        "exact path/table/alias/package/requirement inventory"
    );
    assert!(
        stale.is_empty(),
        "release-owner updates required:\n{}",
        stale.join("\n")
    );
}

#[test]
fn audited_checksums_match_every_maintained_lock_identity() {
    let audit: serde_json::Value = serde_json::from_str(AUDIT).test_unwrap("audit JSON");
    for path in cargo_inventory::maintained_paths("Cargo.lock") {
        let lock = cargo_inventory::read_toml(&cargo_inventory::root().join(&path));
        let packages = cargo_edges::packages(&lock).test_unwrap("lock identities");
        registry_audit::lock_checksums(&audit, packages)
            .unwrap_or_else(|error| panic!("{path}: {error}"));
    }
}

#[test]
fn tracked_inventory_rejects_removed_manifest_lock_and_core_fixture() {
    let inventory = cargo_inventory::inventory();
    for key in ["manifests", "locks"] {
        let paths = inventory[key]
            .as_array()
            .test_unwrap("path inventory")
            .iter()
            .map(|path| path.as_str().test_unwrap("path").to_owned())
            .collect::<BTreeSet<_>>();
        for removed in &paths {
            let mut changed = paths.clone();
            changed.remove(removed);
            assert!(
                cargo_inventory::check_inventory(&changed, &inventory[key]).is_err(),
                "{removed}"
            );
        }
    }
}

#[test]
fn tracked_discovery_excludes_untracked_vendor_and_gitlink_inputs() {
    // The supplied index contains no entry for the extra on-disk Cargo.toml.
    let index = b"100644 abc 0\tCargo.toml\0100644 abc 0\tverification/areas/core_language/fixture/Cargo.toml\0100644 abc 0\tvendor/crate/Cargo.toml\0100644 abc 0\tthird_party/crate/Cargo.toml\0160000 abc 0\texternal\0";
    let actual =
        cargo_inventory::select_tracked_paths(index, "Cargo.toml").test_unwrap("tracked listing");
    assert_eq!(
        actual,
        [
            "Cargo.toml".to_owned(),
            "verification/areas/core_language/fixture/Cargo.toml".to_owned()
        ]
        .into()
    );
    assert!(!actual.contains("untracked/Cargo.toml"));
    assert!(
        cargo_inventory::select_tracked_paths(b"120000 abc 0\tCargo.toml\0", "Cargo.toml").is_err()
    );
    assert!(
        cargo_inventory::select_tracked_paths(b"100644 abc 2\tCargo.toml\0", "Cargo.toml").is_err()
    );
}

#[test]
fn an_extra_untracked_manifest_does_not_enter_live_git_discovery() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .test_unwrap("clock")
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("sifr-item49-index-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&directory).test_unwrap("isolated index fixture");
    std::fs::write(directory.join("Cargo.toml"), "[workspace]\n").test_unwrap("tracked manifest");
    for args in [vec!["init", "--quiet"], vec!["add", "--", "Cargo.toml"]] {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(&directory)
            .output()
            .test_unwrap("fixture git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let before =
        cargo_inventory::tracked_paths(&directory, "Cargo.toml").test_unwrap("initial index");
    std::fs::create_dir(directory.join("untracked")).test_unwrap("untracked directory");
    std::fs::write(directory.join("untracked/Cargo.toml"), "[workspace]\n")
        .test_unwrap("untracked manifest");
    let after =
        cargo_inventory::tracked_paths(&directory, "Cargo.toml").test_unwrap("unchanged index");
    std::fs::remove_dir_all(&directory).test_unwrap("remove owned index fixture");
    assert_eq!(before, ["Cargo.toml".to_owned()].into());
    assert_eq!(after, before);
}

#[test]
fn registry_rows_reject_missing_extra_duplicate_and_malformed_releases() {
    let audit: serde_json::Value = serde_json::from_str(AUDIT).test_unwrap("audit JSON");
    let declarations = cargo_inventory::declarations();
    let mut missing = audit.clone();
    missing["packages"]
        .as_array_mut()
        .test_unwrap("rows")
        .remove(0);
    assert!(registry_audit::exact_packages(&missing, &declarations).is_err());
    let mut extra = audit.clone();
    let mut row = extra["packages"][0].clone();
    row["name"] = "not-a-declared-package".into();
    extra["packages"]
        .as_array_mut()
        .test_unwrap("rows")
        .push(row);
    assert!(registry_audit::exact_packages(&extra, &declarations).is_err());
    let mut duplicate = audit.clone();
    let row = duplicate["packages"][0].clone();
    duplicate["packages"]
        .as_array_mut()
        .test_unwrap("rows")
        .push(row);
    assert!(registry_audit::releases(&duplicate).is_err());
    for (field, value) in [
        ("latest_stable", "1.0.0-rc.1"),
        ("latest_stable", "01.0.0"),
        ("checksum", "bad"),
    ] {
        let mut invalid = audit.clone();
        invalid["packages"][0][field] = value.into();
        assert!(registry_audit::releases(&invalid).is_err());
    }
}

#[test]
fn registry_aliases_preserve_package_identity_and_source_boundary() {
    let registry: toml::Value = toml::from_str(
        r#"renamed = { package = "zip", version = "=8.6.0" }
local_probe = { package = "bindgen", path = "rust/bindgen" }
inherited = { workspace = true }"#,
    )
    .test_unwrap("alias manifest");
    assert_eq!(
        cargo_inventory::registry_requirement("renamed", &registry["renamed"]),
        Ok(Some(("zip".into(), "=8.6.0".into())))
    );
    assert_eq!(
        cargo_inventory::registry_requirement("local_probe", &registry["local_probe"]),
        Ok(None)
    );
    assert_eq!(
        cargo_inventory::registry_requirement("inherited", &registry["inherited"]),
        Ok(None)
    );
    let mut changed = registry["renamed"].clone();
    changed["package"] = "different".into();
    assert_ne!(
        cargo_inventory::registry_requirement("renamed", &changed),
        cargo_inventory::registry_requirement("renamed", &registry["renamed"])
    );
}

#[test]
fn lock_identity_resolver_rejects_missing_ambiguous_duplicate_and_wrong_sources() {
    let lock: toml::Value = toml::from_str(
        r#"[[package]]
name = "syn"
version = "3.0.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
[[package]]
name = "syn"
version = "2.0.117"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#,
    )
    .test_unwrap("lock fixture");
    let packages = cargo_edges::packages(&lock).test_unwrap("packages");
    assert!(cargo_edges::current_edge(packages, "syn 3.0.5", "syn", "3.0.5").is_ok());
    for edge in [
        "syn",
        "syn 3.0.4",
        "syn 3.0.5 (git+https://example.invalid/syn)",
        "syn 3.0.5 extra",
        "",
    ] {
        assert!(cargo_edges::resolve(packages, edge).is_err(), "{edge}");
    }
    let mut changed = lock.clone();
    let duplicate = changed["package"][0].clone();
    changed["package"]
        .as_array_mut()
        .test_unwrap("packages")
        .push(duplicate);
    assert!(cargo_edges::packages(&changed).is_err());
    let mut source = packages[0].clone();
    source["source"] = "git+https://example.invalid/syn".into();
    assert!(cargo_edges::current_edge(&[source], "syn", "syn", "3.0.5").is_err());
}

#[test]
fn checksum_audit_rejects_full_version_source_and_checksum_drift() {
    let audit: serde_json::Value = serde_json::from_str(AUDIT).test_unwrap("audit");
    let lock = cargo_inventory::read_toml(&cargo_inventory::root().join("Cargo.lock"));
    let package = cargo_edges::packages(&lock)
        .test_unwrap("packages")
        .iter()
        .find(|package| {
            package["name"].as_str() == Some("syn") && package["version"].as_str() == Some("3.0.5")
        })
        .test_unwrap("selected Syn");
    for (field, value) in [
        ("version", "3.0.5+different"),
        ("source", "git+https://example.invalid/syn"),
        ("checksum", "bad"),
    ] {
        let mut changed = package.clone();
        changed[field] = value.into();
        assert!(
            registry_audit::lock_checksums(&audit, &[changed]).is_err(),
            "{field}"
        );
    }
}
