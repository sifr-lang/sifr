use super::{
    feature_for_codegen_requirement, planned_sifr_stdlib_features,
    try_generated_cargo_dependencies, try_sysroot_dependency_plan, CargoVendorMode, StdlibFeature,
};
use std::collections::HashSet;

fn generated_cargo_dependencies(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> Vec<String> {
    try_generated_cargo_dependencies(stdlib_modules, required_features)
        .expect("source-tree sysroot dependencies should resolve")
}

#[test]
fn stdlib_module_dependencies_are_deterministic_and_deduplicated() {
    let stdlib_modules = HashSet::from([
        "sifr.json".to_string(),
        "_sifr.json".to_string(),
        "sifr.random".to_string(),
    ]);
    let required_features = HashSet::from([StdlibFeature::SerdeJson, StdlibFeature::Rand]);

    let deps = generated_cargo_dependencies(&stdlib_modules, &required_features);
    assert!(deps[0].contains("features = [\"json\", \"random\"]"));

    assert_eq!(
        deps,
        vec![
            deps[0].clone(),
            "serde_json = { version = \"1.0.149\", features = [\"preserve_order\"] }".to_string(),
            "serde = { version = \"1.0.228\", features = [\"derive\"] }".to_string(),
            "rand = \"0.10.1\"".to_string(),
            "rand_distr = \"0.6.0\"".to_string(),
        ]
    );
}

#[test]
fn unknown_modules_and_empty_features_do_not_emit_dependencies() {
    let stdlib_modules = HashSet::from(["sifr.not_real".to_string()]);
    let required_features = HashSet::new();

    assert!(generated_cargo_dependencies(&stdlib_modules, &required_features).is_empty());
}

#[test]
fn runtime_and_tokio_features_render_owned_dependency_specs() {
    let deps = generated_cargo_dependencies(
        &HashSet::new(),
        &HashSet::from([StdlibFeature::SifrRuntime, StdlibFeature::Tokio]),
    );

    assert!(deps.iter().any(|dep| dep.starts_with("sifr_runtime = ")
        && dep.contains("default-features = false")
        && !dep.contains("features = [")));
    assert!(deps.iter().any(|dep| dep.starts_with("tokio = ")));
}

#[test]
fn ipc_feature_renders_locked_postcard_specs_without_json() {
    let deps = generated_cargo_dependencies(
        &HashSet::from(["sifr.ipc".to_string(), "_sifr.ipc".to_string()]),
        &HashSet::from([StdlibFeature::Ipc]),
    );

    assert_eq!(
        deps,
        vec![
            "postcard = { version = \"1.1.3\", default-features = false, features = [\"use-std\"] }",
            "serde = { version = \"1.0.228\", features = [\"derive\"] }",
        ]
    );
    assert!(!deps.iter().any(|dep| dep.starts_with("serde_json = ")));
    assert_eq!(
        feature_for_codegen_requirement("ipc"),
        Some(StdlibFeature::Ipc)
    );
    assert_eq!(
        feature_for_codegen_requirement("postcard"),
        Some(StdlibFeature::Ipc)
    );
}

#[test]
fn unicode_module_emits_runtime_and_unicode_dependencies() {
    let deps = generated_cargo_dependencies(
        &HashSet::from(["sifr.unicode".to_string()]),
        &HashSet::new(),
    );

    assert!(deps
        .iter()
        .any(|dep| dep.starts_with("sifr_runtime = ") && dep.contains("features = [\"unicode\"]")));
    assert!(deps.iter().any(|dep| dep.starts_with("sifr_stdlib = ")
        && dep.contains("default-features = false")
        && dep.contains("features = [\"unicode\"]")));
    assert!(deps.contains(&"unicode_names2 = \"3.1.0\"".to_string()));
    assert!(deps.contains(&"unicode-normalization = \"0.1.25\"".to_string()));
    assert!(deps.contains(&"unicode-segmentation = \"1.13.3\"".to_string()));
}

#[test]
fn unicode_intrinsic_features_enable_runtime_unicode_feature() {
    let deps = generated_cargo_dependencies(
        &HashSet::new(),
        &HashSet::from([
            StdlibFeature::SifrRuntime,
            StdlibFeature::UnicodeNames,
            StdlibFeature::UnicodeNormalization,
            StdlibFeature::UnicodeSegmentation,
        ]),
    );

    assert!(deps
        .iter()
        .any(|dep| dep.starts_with("sifr_runtime = ") && dep.contains("features = [\"unicode\"]")));
}

#[test]
fn i18n_module_emits_runtime_and_icu_dependencies() {
    let deps =
        generated_cargo_dependencies(&HashSet::from(["sifr.i18n".to_string()]), &HashSet::new());

    assert!(deps
        .iter()
        .any(|dep| dep.starts_with("sifr_runtime = ") && dep.contains("features = [\"i18n\"]")));
    assert!(deps.iter().any(|dep| dep.starts_with("sifr_stdlib = ")
        && dep.contains("default-features = false")
        && dep.contains("features = [\"i18n\"]")));
    assert!(deps.contains(&"icu_collator = \"2.2.0\"".to_string()));
    assert!(deps.contains(&"icu_datetime = \"2.2.0\"".to_string()));
    assert!(deps.contains(&"icu_decimal = \"2.2.0\"".to_string()));
    assert!(deps.contains(&"icu_locale = \"2.2.0\"".to_string()));
    assert!(deps.contains(&"icu_plurals = \"2.2.0\"".to_string()));
}

#[test]
fn runtime_dependency_can_enable_unicode_and_i18n_together() {
    let deps = generated_cargo_dependencies(
        &HashSet::new(),
        &HashSet::from([
            StdlibFeature::SifrRuntime,
            StdlibFeature::IcuLocale,
            StdlibFeature::UnicodeNormalization,
        ]),
    );

    assert!(deps.iter().any(|dep| dep.starts_with("sifr_runtime = ")
        && dep.contains("features = [\"i18n\", \"unicode\"]")));
}

#[test]
fn python_runtime_feature_enables_sifr_runtime_python_feature() {
    let module_deps =
        generated_cargo_dependencies(&HashSet::from(["sifr.python".to_string()]), &HashSet::new());
    assert!(module_deps
        .iter()
        .any(|dep| dep.starts_with("sifr_runtime = ") && dep.contains("features = [\"python\"]")));

    let deps = generated_cargo_dependencies(
        &HashSet::new(),
        &HashSet::from([StdlibFeature::PythonRuntime]),
    );

    assert!(deps
        .iter()
        .any(|dep| dep.starts_with("sifr_runtime = ") && dep.contains("features = [\"python\"]")));
    assert_eq!(
        feature_for_codegen_requirement("python-runtime"),
        Some(StdlibFeature::PythonRuntime)
    );
}

#[test]
fn planned_sysroot_stdlib_features_are_minimal_for_representative_modules() {
    let cases = [
        ("sifr.re", &["regex"][..], &["json", "http", "python"][..]),
        ("sifr.json", &["json"][..], &["regex", "http", "python"][..]),
        ("sifr.html", &["html"][..], &["json", "regex", "http"][..]),
        (
            "sifr.calendar",
            &["calendar"][..],
            &["json", "regex", "http"][..],
        ),
        ("sifr.uuid", &["uuid"][..], &["json", "regex", "http"][..]),
        ("sifr.http", &["http"][..], &["json", "regex", "python"][..]),
        (
            "sifr.platform",
            &["platform"][..],
            &["json", "regex", "http"][..],
        ),
        (
            "sifr.python",
            &["python"][..],
            &["json", "regex", "http"][..],
        ),
        (
            "sifr.math",
            &["math"][..],
            &["json", "regex", "http", "python"][..],
        ),
        (
            "sifr.hashlib",
            &["hash"][..],
            &["json", "regex", "http", "python"][..],
        ),
    ];

    for (module, expected, must_not_include) in cases {
        let planned =
            planned_sifr_stdlib_features(&HashSet::from([module.to_string()]), &HashSet::new());
        let expected = expected.iter().copied().collect();
        assert_eq!(planned, expected, "unexpected features for {module}");
        for unexpected in must_not_include {
            assert!(
                !planned.contains(unexpected),
                "{module} unexpectedly enabled {unexpected}"
            );
        }
    }
}

#[test]
fn stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies() {
    for (module, expected_feature) in [
        ("sifr.html", "html"),
        ("sifr.platform", "platform"),
        ("sifr.calendar", "calendar"),
        ("sifr.uuid", "uuid"),
        ("sifr.math", "math"),
        ("sifr.hashlib", "hash"),
    ] {
        let deps =
            generated_cargo_dependencies(&HashSet::from([module.to_string()]), &HashSet::new());
        assert_eq!(deps.len(), 1, "{module} should only emit sifr_stdlib");
        assert!(
            deps[0].starts_with("sifr_stdlib = "),
            "{module} dependency: {}",
            deps[0]
        );
        assert!(
            deps[0].contains("default-features = false")
                && deps[0].contains(&format!("features = [\"{expected_feature}\"]")),
            "{module} dependency: {}",
            deps[0]
        );
    }
}

#[test]
fn planned_sysroot_stdlib_features_include_codegen_requirements_without_umbrellas() {
    let planned = planned_sifr_stdlib_features(
        &HashSet::new(),
        &HashSet::from([
            StdlibFeature::Regex,
            StdlibFeature::SerdeJson,
            StdlibFeature::PythonRuntime,
        ]),
    );

    assert_eq!(planned, ["json", "python", "regex"].into_iter().collect());
    assert!(!planned.contains("text-data"));
    assert!(!planned.contains("network-stack"));
}

#[test]
fn sysroot_dependency_plan_captures_identity_features_and_vendor_mode() {
    let plan = try_sysroot_dependency_plan(
        &HashSet::from(["sifr.json".to_string()]),
        &HashSet::new(),
        CargoVendorMode::SysrootOnly,
    )
    .expect("source-tree sysroot should resolve");

    assert_eq!(plan.cargo_vendor_mode, CargoVendorMode::SysrootOnly);
    assert_eq!(plan.sysroot_content_sha256.len(), 64);
    assert!(plan.cargo_config.ends_with(".cargo/config.toml"));
    assert!(plan.vendor_dir.ends_with("vendor"));
    assert!(plan.cache_fingerprint.contains("vendor_mode=sysroot-only"));
    assert!(plan
        .cache_fingerprint
        .contains(&format!("content_sha256={}", plan.sysroot_content_sha256)));
    assert!(plan.cargo_dependency_lines().iter().any(|dep| {
        dep.starts_with("sifr_stdlib = ")
            && dep.contains("default-features = false")
            && dep.contains("features = [\"json\"]")
    }));
}
