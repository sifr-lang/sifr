const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");
const WORKSPACE_LOCK: &str = include_str!("../../../Cargo.lock");

#[test]
fn icu_direct_dependencies_match_latest_stable_versions() {
    let manifest: toml::Value =
        toml::from_str(WORKSPACE_MANIFEST).expect("workspace manifest must parse");
    let dependencies = manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(toml::Value::as_table)
        .expect("workspace dependencies must be a table");
    let lock: toml::Value = toml::from_str(WORKSPACE_LOCK).expect("workspace lock must parse");
    let locked_packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .expect("workspace lock packages must be an array");

    for (package, expected_version) in [
        ("icu_collator", "2.3.1"),
        ("icu_datetime", "2.3.0"),
        ("icu_decimal", "2.3.0"),
        ("icu_locale", "2.3.1"),
        ("icu_plurals", "2.3.0"),
    ] {
        assert_eq!(
            dependencies
                .get(package)
                .and_then(|dependency| dependency.get("version"))
                .and_then(toml::Value::as_str),
            Some(expected_version),
            "workspace dependency {package} must select the latest stable version"
        );
        assert_eq!(
            locked_versions(locked_packages, package),
            vec![expected_version],
            "workspace lock must contain exactly one {package} version"
        );
    }
}

#[test]
fn icu_shared_graph_uses_locale_fallback_split() {
    let lock: toml::Value = toml::from_str(WORKSPACE_LOCK).expect("workspace lock must parse");
    let locked_packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .expect("workspace lock packages must be an array");

    for (package, expected_version) in [("icu_locale_fallback", "2.3.0"), ("icu_provider", "2.3.1")]
    {
        assert_eq!(
            locked_versions(locked_packages, package),
            vec![expected_version],
            "workspace lock must contain the patched {package} release"
        );
    }

    let locale_dependencies = package_dependencies(locked_packages, "icu_locale");
    assert!(locale_dependencies.contains(&"icu_locale_fallback"));
    assert!(!locale_dependencies.contains(&"potential_utf"));

    let fallback_dependencies = package_dependencies(locked_packages, "icu_locale_fallback");
    assert!(fallback_dependencies.contains(&"potential_utf"));
}

fn locked_versions<'a>(packages: &'a [toml::Value], name: &str) -> Vec<&'a str> {
    packages
        .iter()
        .filter(|entry| entry.get("name").and_then(toml::Value::as_str) == Some(name))
        .filter_map(|entry| entry.get("version").and_then(toml::Value::as_str))
        .collect()
}

fn package_dependencies<'a>(packages: &'a [toml::Value], name: &str) -> Vec<&'a str> {
    packages
        .iter()
        .find(|entry| entry.get("name").and_then(toml::Value::as_str) == Some(name))
        .unwrap_or_else(|| panic!("{name} must be locked"))
        .get("dependencies")
        .and_then(toml::Value::as_array)
        .unwrap_or_else(|| panic!("{name} dependencies must be an array"))
        .iter()
        .filter_map(toml::Value::as_str)
        .collect()
}
