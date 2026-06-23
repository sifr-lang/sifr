use super::{feature_for_codegen_requirement, generated_cargo_dependencies, StdlibFeature};
use std::collections::HashSet;

#[test]
fn stdlib_module_dependencies_are_deterministic_and_deduplicated() {
    let stdlib_modules = HashSet::from([
        "sifr.json".to_string(),
        "_sifr.json".to_string(),
        "sifr.random".to_string(),
    ]);
    let required_features = HashSet::from([StdlibFeature::SerdeJson, StdlibFeature::Rand]);

    let deps = generated_cargo_dependencies(&stdlib_modules, &required_features);

    assert_eq!(
        deps,
        vec![
            "serde_json = { version = \"1.0.149\", features = [\"preserve_order\"] }",
            "serde = { version = \"1.0.228\", features = [\"derive\"] }",
            "rand = \"0.10.1\"",
            "rand_distr = \"0.6.0\"",
        ]
    );
}

#[test]
fn unknown_modules_and_empty_features_do_not_emit_dependencies() {
    let stdlib_modules = HashSet::from(["sifr.io".to_string()]);
    let required_features = HashSet::new();

    assert!(generated_cargo_dependencies(&stdlib_modules, &required_features).is_empty());
}

#[test]
fn runtime_and_tokio_features_render_owned_dependency_specs() {
    let deps = generated_cargo_dependencies(
        &HashSet::new(),
        &HashSet::from([StdlibFeature::SifrRuntime, StdlibFeature::Tokio]),
    );

    assert!(deps
        .iter()
        .any(|dep| dep.starts_with("sifr_runtime = ") && !dep.contains("features")));
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
