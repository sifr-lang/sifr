use crate::{
    create_invocation_workspace, discover_test_root_modules, parse_import_closure_modules,
    DiscoveryDiagnosticStyle,
};
use std::collections::BTreeSet;

#[test]
fn test_create_invocation_workspace_returns_unique_paths() {
    let first =
        create_invocation_workspace("workspace_unique").expect("first workspace should be created");
    let second = create_invocation_workspace("workspace_unique")
        .expect("second workspace should be created");
    assert_ne!(first, second);
    assert!(first.exists());
    assert!(second.exists());

    let _ = std::fs::remove_dir_all(first);
    let _ = std::fs::remove_dir_all(second);
}

#[test]
fn test_discover_test_root_modules_is_deterministic() {
    let unique = format!(
        "sifr_test_root_discovery_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&dir).expect("test dir should be created");
    std::fs::write(dir.join("z_test.sifr"), "def test_z():\n    assert True\n")
        .expect("z_test should be written");
    std::fs::write(dir.join("test_a.sifr"), "def test_a():\n    assert True\n")
        .expect("test_a should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "def helper() -> int:\n    return 1\n",
    )
    .expect("helper should be written");

    let roots = discover_test_root_modules(&dir);
    let names: Vec<String> = roots.keys().cloned().collect();
    assert_eq!(names, vec!["test_a".to_string(), "z_test".to_string()]);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_project_and_test_discovery_share_import_closure_membership() {
    let unique = format!(
        "sifr_discovery_parity_positive_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&dir).expect("project dir should be created");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import value\n\ndef main():\n    print(value())\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("test_parity.sifr"),
        "from helper import value\n\ndef test_value():\n    assert value() == 42\n",
    )
    .expect("test_parity should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "from shared import BASE\n\ndef value() -> int:\n    return BASE\n",
    )
    .expect("helper should be written");
    std::fs::write(dir.join("shared.sifr"), "BASE: int = 42\n").expect("shared should be written");
    std::fs::write(dir.join("unrelated_bad.sifr"), "def unrelated(:\n")
        .expect("unrelated sibling should be written");

    let project_roots = BTreeSet::from(["main".to_string()]);
    let test_roots = BTreeSet::from(["test_parity".to_string()]);
    let project_modules =
        parse_import_closure_modules(&dir, &project_roots, DiscoveryDiagnosticStyle::ModuleName)
            .expect("project closure discovery should succeed");
    let test_modules =
        parse_import_closure_modules(&dir, &test_roots, DiscoveryDiagnosticStyle::ModuleName)
            .expect("test closure discovery should succeed");

    let project_support: BTreeSet<String> = project_modules
        .keys()
        .filter(|name| !project_roots.contains(*name))
        .cloned()
        .collect();
    let test_support: BTreeSet<String> = test_modules
        .keys()
        .filter(|name| !test_roots.contains(*name))
        .cloned()
        .collect();

    assert_eq!(
        project_support,
        BTreeSet::from(["helper".to_string(), "shared".to_string()])
    );
    assert_eq!(project_support, test_support);
    assert!(!project_support.contains("unrelated_bad"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_project_and_test_discovery_parity_reports_reachable_parse_errors() {
    let unique = format!(
        "sifr_discovery_parity_negative_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&dir).expect("project dir should be created");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import value\n\ndef main():\n    print(value())\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("test_parity.sifr"),
        "from helper import value\n\ndef test_value():\n    assert value() == 1\n",
    )
    .expect("test_parity should be written");
    std::fs::write(dir.join("helper.sifr"), "def value(:\n").expect("helper should be written");
    std::fs::write(
        dir.join("unrelated_ok.sifr"),
        "def spare() -> int:\n    return 1\n",
    )
    .expect("unrelated should be written");

    let project_roots = BTreeSet::from(["main".to_string()]);
    let test_roots = BTreeSet::from(["test_parity".to_string()]);

    let project_errors =
        parse_import_closure_modules(&dir, &project_roots, DiscoveryDiagnosticStyle::ModuleName)
            .err()
            .expect("project closure should fail on reachable parse error");
    let test_errors =
        parse_import_closure_modules(&dir, &test_roots, DiscoveryDiagnosticStyle::ModuleName)
            .err()
            .expect("test closure should fail on reachable parse error");

    assert!(project_errors
        .iter()
        .any(|e| e.message.contains("[helper]")));
    assert!(test_errors.iter().any(|e| e.message.contains("[helper]")));

    let _ = std::fs::remove_dir_all(&dir);
}
