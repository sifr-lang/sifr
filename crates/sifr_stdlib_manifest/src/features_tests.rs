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
fn stdlib_module_dependencies_use_only_sysroot_crates_for_json_and_collections() {
    let stdlib_modules = HashSet::from([
        "sifr.collections".to_string(),
        "_sifr.collections".to_string(),
        "sifr.random".to_string(),
    ]);
    let required_features = HashSet::from([StdlibFeature::SerdeJson, StdlibFeature::Rand]);

    let deps = generated_cargo_dependencies(&stdlib_modules, &required_features);
    assert!(deps[0].contains("features = [\"collections\", \"json\", \"random\"]"));

    assert_eq!(deps.len(), 1);
    assert!(deps[0].starts_with("sifr_stdlib = "));
    assert!(!deps.iter().any(|dep| dep.starts_with("serde_json = ")));
    assert!(!deps.iter().any(|dep| dep.starts_with("serde = ")));
}

#[test]
fn random_module_emits_only_sysroot_stdlib_dependency() {
    let deps =
        generated_cargo_dependencies(&HashSet::from(["sifr.random".to_string()]), &HashSet::new());

    assert_eq!(deps.len(), 1);
    assert!(deps[0].starts_with("sifr_stdlib = "));
    assert!(deps[0].contains("features = [\"random\"]"));
    assert!(!deps[0].contains("sifr_runtime"));
    assert!(!deps.iter().any(|dep| dep.starts_with("rand = ")));
    assert!(!deps.iter().any(|dep| dep.starts_with("rand_distr = ")));
}

#[test]
fn process_private_module_emits_process_stdlib_feature() {
    for module in ["sifr.process", "_sifr.process"] {
        let deps =
            generated_cargo_dependencies(&HashSet::from([module.to_string()]), &HashSet::new());

        assert_eq!(deps.len(), 1, "{module} should only need sifr_stdlib");
        assert!(deps[0].starts_with("sifr_stdlib = "), "{module}");
        assert!(deps[0].contains("default-features = false"), "{module}");
        assert!(
            deps[0].contains("features = [\"process\"]"),
            "{module} should enable the process stdlib feature"
        );
    }
}

#[test]
fn unknown_modules_and_empty_features_do_not_emit_dependencies() {
    let stdlib_modules = HashSet::from(["sifr.not_real".to_string()]);
    let required_features = HashSet::new();

    assert!(generated_cargo_dependencies(&stdlib_modules, &required_features).is_empty());
}

#[test]
fn runtime_and_tokio_features_render_retained_glue_dependency_specs() {
    let deps = generated_cargo_dependencies(
        &HashSet::new(),
        &HashSet::from([StdlibFeature::SifrRuntime, StdlibFeature::Tokio]),
    );

    assert_eq!(deps.len(), 2);
    assert!(
        deps[0].starts_with("sifr_runtime = ")
            && deps[0].contains("default-features = false")
            && !deps[0].contains("features = [")
    );
    assert!(deps.iter().any(|dep| dep.starts_with("tokio = ")));
}

#[test]
fn ipc_feature_renders_sysroot_specs_without_json() {
    let deps = generated_cargo_dependencies(
        &HashSet::from(["sifr.ipc".to_string(), "_sifr.ipc".to_string()]),
        &HashSet::from([StdlibFeature::Ipc]),
    );

    assert!(deps.is_empty());
    assert!(!deps.iter().any(|dep| dep.starts_with("serde_json = ")));
    assert!(!deps.iter().any(|dep| dep.starts_with("serde = ")));
    assert!(!deps.iter().any(|dep| dep.starts_with("postcard = ")));
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
fn unicode_module_emits_only_sysroot_stdlib_dependency() {
    let deps = generated_cargo_dependencies(
        &HashSet::from(["sifr.unicode".to_string()]),
        &HashSet::new(),
    );

    assert_eq!(deps.len(), 1);
    assert!(deps[0].starts_with("sifr_stdlib = "));
    assert!(deps[0].contains("default-features = false"));
    assert!(deps[0].contains("features = [\"unicode\"]"));
    assert!(!deps[0].starts_with("sifr_runtime = "));
}

#[test]
fn logging_module_emits_only_sysroot_stdlib_dependency() {
    let deps = generated_cargo_dependencies(
        &HashSet::from(["sifr.logging".to_string(), "_sifr.logging".to_string()]),
        &HashSet::new(),
    );

    assert_eq!(deps.len(), 1);
    assert!(deps[0].starts_with("sifr_stdlib = "));
    assert!(deps[0].contains("default-features = false"));
    assert!(deps[0].contains("features = [\"logging\"]"));
    assert!(!deps[0].starts_with("sifr_runtime = "));
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
fn i18n_module_emits_only_stdlib_feature_dependency() {
    let deps =
        generated_cargo_dependencies(&HashSet::from(["sifr.i18n".to_string()]), &HashSet::new());

    assert!(!deps.iter().any(|dep| dep.starts_with("sifr_runtime = ")));
    assert!(deps.iter().any(|dep| dep.starts_with("sifr_stdlib = ")
        && dep.contains("default-features = false")
        && dep.contains("features = [\"i18n\"]")));
    assert!(!deps.iter().any(|dep| dep.starts_with("icu_")));
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
            "sifr.env",
            &["sys"][..],
            &["json", "regex", "http", "platform"][..],
        ),
        (
            "sifr.sys",
            &["sys"][..],
            &["json", "regex", "http", "platform"][..],
        ),
        (
            "sifr.os",
            &["fs", "sys"][..],
            &["json", "regex", "http", "platform"][..],
        ),
        (
            "sifr.shutil",
            &["fs", "sys"][..],
            &["json", "regex", "http", "platform"][..],
        ),
        (
            "sifr.python",
            &["python"][..],
            &["json", "regex", "http"][..],
        ),
        ("sifr.pathlib", &["fs"][..], &["json", "regex", "http"][..]),
        (
            "sifr.math",
            &["math"][..],
            &["json", "regex", "http", "python"][..],
        ),
        (
            "sifr.hashlib",
            &["bytes", "hash"][..],
            &["json", "regex", "http", "python"][..],
        ),
        (
            "sifr.base64",
            &["base64", "bytes"][..],
            &["json", "regex", "http", "python"][..],
        ),
        (
            "sifr.bytes",
            &["bytes"][..],
            &["json", "regex", "http", "python"][..],
        ),
        (
            "sifr.tomllib",
            &["toml"][..],
            &["json", "regex", "http", "python"][..],
        ),
        (
            "sifr.encoding",
            &["encoding"][..],
            &["json", "regex", "http", "python"][..],
        ),
        (
            "sifr.unicode",
            &["unicode"][..],
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
    for (module, expected_features) in [
        ("sifr.html", &["html"][..]),
        ("_sifr.html", &["html"][..]),
        ("sifr.platform", &["platform"][..]),
        ("_sifr.platform", &["platform"][..]),
        ("sifr.env", &["sys"][..]),
        ("sifr.sys", &["sys"][..]),
        ("_sifr.sys", &["sys"][..]),
        ("sifr.os", &["fs", "sys"][..]),
        ("sifr.shutil", &["fs", "sys"][..]),
        ("sifr.calendar", &["calendar"][..]),
        ("_sifr.calendar", &["calendar"][..]),
        ("sifr.uuid", &["uuid"][..]),
        ("_sifr.uuid", &["uuid"][..]),
        ("sifr.math", &["math"][..]),
        ("_sifr.math", &["math"][..]),
        ("sifr.hashlib", &["bytes", "hash"][..]),
        ("sifr.base64", &["base64", "bytes"][..]),
        ("sifr.bytes", &["bytes"][..]),
        ("_sifr.bytes", &["bytes"][..]),
        ("sifr.collections", &["collections"][..]),
        ("_sifr.collections", &["collections"][..]),
        ("sifr.re", &["regex"][..]),
        ("_sifr.regex", &["regex"][..]),
        ("sifr.pathlib", &["fs"][..]),
        ("sifr.url", &["url"][..]),
        ("_sifr.url", &["url"][..]),
        ("sifr.tomllib", &["toml"][..]),
        ("_sifr.toml", &["toml"][..]),
        ("sifr.encoding", &["encoding"][..]),
        ("_sifr.encoding", &["encoding"][..]),
        ("sifr.unicode", &["unicode"][..]),
        ("_sifr.unicode", &["unicode"][..]),
        ("_sifr.fs", &["fs"][..]),
    ] {
        let deps =
            generated_cargo_dependencies(&HashSet::from([module.to_string()]), &HashSet::new());
        assert_eq!(deps.len(), 1, "{module} should only emit sifr_stdlib");
        assert!(
            deps[0].starts_with("sifr_stdlib = "),
            "{module} dependency: {}",
            deps[0]
        );
        let expected_feature_list = expected_features
            .iter()
            .map(|feature| format!("\"{feature}\""))
            .collect::<Vec<_>>()
            .join(", ");
        assert!(
            deps[0].contains("default-features = false")
                && deps[0].contains(&format!("features = [{expected_feature_list}]")),
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
        &HashSet::from([StdlibFeature::SerdeJson]),
        CargoVendorMode::SysrootOnly,
    )
    .expect("source-tree sysroot should resolve");

    assert_eq!(plan.cargo_vendor_mode, CargoVendorMode::SysrootOnly);
    assert_eq!(
        plan.stdlib_modules,
        ["sifr.json".to_string()].into_iter().collect()
    );
    assert_eq!(
        plan.required_features,
        [StdlibFeature::SerdeJson].into_iter().collect()
    );
    assert_eq!(
        plan.dependency_input_fingerprint(),
        "[stdlib]\nsifr.json\n[features]\nserde_json\n"
    );
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
