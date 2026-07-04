use sifr_stdlib_model::{try_generated_cargo_dependencies, StdlibFeature};
use std::collections::HashSet;

fn normalize_runtime_dependency(dependency: &str) -> String {
    normalize_path_dependency(
        normalize_path_dependency(
            dependency.to_string(),
            "sifr_runtime",
            "<sifr_runtime_path>",
        ),
        "sifr_stdlib",
        "<sifr_stdlib_path>",
    )
}

fn normalize_path_dependency(dependency: String, package: &str, placeholder: &str) -> String {
    if !dependency.starts_with(&format!("{package} = ")) {
        return dependency;
    }
    let Some(path_start) = dependency.find("path = \"") else {
        return dependency;
    };
    let value_start = path_start + "path = \"".len();
    let Some(value_end_offset) = dependency[value_start..].find('"') else {
        return dependency;
    };
    let value_end = value_start + value_end_offset;
    format!(
        "{}{}{}",
        &dependency[..value_start],
        placeholder,
        &dependency[value_end..]
    )
}

fn generated_cargo_dependencies(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> Vec<String> {
    try_generated_cargo_dependencies(stdlib_modules, required_features)
        .expect("source-tree sysroot dependencies should resolve")
}

#[test]
fn text_i18n_feature_dependency_snapshots_cover_feature_combinations() {
    let cases = [
        (
            "encoding",
            HashSet::from(["sifr.encoding".to_string()]),
            vec![
                "sifr_stdlib = { path = \"<sifr_stdlib_path>\", default-features = false, features = [\"encoding\"] }",
            ],
        ),
        (
            "unicode",
            HashSet::from(["sifr.unicode".to_string()]),
            vec![
                "sifr_runtime = { path = \"<sifr_runtime_path>\", default-features = false, features = [\"unicode\"] }",
                "sifr_stdlib = { path = \"<sifr_stdlib_path>\", default-features = false, features = [\"unicode\"] }",
                "unicode_names2 = \"3.1.0\"",
                "unicode-normalization = \"0.1.25\"",
                "unicode-segmentation = \"1.13.3\"",
            ],
        ),
        (
            "i18n",
            HashSet::from(["sifr.i18n".to_string()]),
            vec![
                "sifr_runtime = { path = \"<sifr_runtime_path>\", default-features = false, features = [\"i18n\"] }",
                "sifr_stdlib = { path = \"<sifr_stdlib_path>\", default-features = false, features = [\"i18n\"] }",
                "icu_collator = \"2.2.0\"",
                "icu_datetime = \"2.2.0\"",
                "icu_decimal = \"2.2.0\"",
                "icu_locale = \"2.2.0\"",
                "icu_plurals = \"2.2.0\"",
            ],
        ),
        (
            "encoding-and-unicode",
            HashSet::from(["sifr.encoding".to_string(), "sifr.unicode".to_string()]),
            vec![
                "sifr_runtime = { path = \"<sifr_runtime_path>\", default-features = false, features = [\"unicode\"] }",
                "sifr_stdlib = { path = \"<sifr_stdlib_path>\", default-features = false, features = [\"encoding\", \"unicode\"] }",
                "unicode_names2 = \"3.1.0\"",
                "unicode-normalization = \"0.1.25\"",
                "unicode-segmentation = \"1.13.3\"",
            ],
        ),
        (
            "encoding-and-i18n",
            HashSet::from(["sifr.encoding".to_string(), "sifr.i18n".to_string()]),
            vec![
                "sifr_runtime = { path = \"<sifr_runtime_path>\", default-features = false, features = [\"i18n\"] }",
                "sifr_stdlib = { path = \"<sifr_stdlib_path>\", default-features = false, features = [\"encoding\", \"i18n\"] }",
                "icu_collator = \"2.2.0\"",
                "icu_datetime = \"2.2.0\"",
                "icu_decimal = \"2.2.0\"",
                "icu_locale = \"2.2.0\"",
                "icu_plurals = \"2.2.0\"",
            ],
        ),
        (
            "unicode-and-i18n",
            HashSet::from(["sifr.unicode".to_string(), "sifr.i18n".to_string()]),
            vec![
                "sifr_runtime = { path = \"<sifr_runtime_path>\", default-features = false, features = [\"i18n\", \"unicode\"] }",
                "sifr_stdlib = { path = \"<sifr_stdlib_path>\", default-features = false, features = [\"i18n\", \"unicode\"] }",
                "icu_collator = \"2.2.0\"",
                "icu_datetime = \"2.2.0\"",
                "icu_decimal = \"2.2.0\"",
                "icu_locale = \"2.2.0\"",
                "icu_plurals = \"2.2.0\"",
                "unicode_names2 = \"3.1.0\"",
                "unicode-normalization = \"0.1.25\"",
                "unicode-segmentation = \"1.13.3\"",
            ],
        ),
        (
            "encoding-unicode-and-i18n",
            HashSet::from([
                "sifr.encoding".to_string(),
                "sifr.i18n".to_string(),
                "sifr.unicode".to_string(),
            ]),
            vec![
                "sifr_runtime = { path = \"<sifr_runtime_path>\", default-features = false, features = [\"i18n\", \"unicode\"] }",
                "sifr_stdlib = { path = \"<sifr_stdlib_path>\", default-features = false, features = [\"encoding\", \"i18n\", \"unicode\"] }",
                "icu_collator = \"2.2.0\"",
                "icu_datetime = \"2.2.0\"",
                "icu_decimal = \"2.2.0\"",
                "icu_locale = \"2.2.0\"",
                "icu_plurals = \"2.2.0\"",
                "unicode_names2 = \"3.1.0\"",
                "unicode-normalization = \"0.1.25\"",
                "unicode-segmentation = \"1.13.3\"",
            ],
        ),
    ];

    for (name, modules, expected) in cases {
        let deps = generated_cargo_dependencies(&modules, &HashSet::new())
            .iter()
            .map(|dependency| normalize_runtime_dependency(dependency))
            .collect::<Vec<_>>();

        assert_eq!(deps, expected, "{name}");
    }
}
