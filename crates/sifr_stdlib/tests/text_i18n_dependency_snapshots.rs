use sifr_stdlib::generated_cargo_dependencies;
use std::collections::HashSet;

fn normalize_runtime_dependency(dependency: &str) -> String {
    if dependency.starts_with("sifr_runtime = ") {
        if dependency.contains("features = [\"i18n\", \"unicode\"]") {
            return "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"i18n\", \"unicode\"] }"
                .to_string();
        }
        if dependency.contains("features = [\"i18n\"]") {
            return "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"i18n\"] }"
                .to_string();
        }
        if dependency.contains("features = [\"unicode\"]") {
            return "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"unicode\"] }"
                .to_string();
        }
        return "sifr_runtime = { path = \"<sifr_runtime_path>\" }".to_string();
    }
    dependency.to_string()
}

#[test]
fn text_i18n_feature_dependency_snapshots_cover_feature_combinations() {
    let cases = [
        (
            "encoding",
            HashSet::from(["sifr.encoding".to_string()]),
            vec![
                "encoding_rs = \"0.8.35\"",
                "sifr_runtime = { path = \"<sifr_runtime_path>\" }",
            ],
        ),
        (
            "unicode",
            HashSet::from(["sifr.unicode".to_string()]),
            vec![
                "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"unicode\"] }",
                "unicode_names2 = \"3.1.0\"",
                "unicode-normalization = \"0.1.25\"",
                "unicode-segmentation = \"1.13.3\"",
            ],
        ),
        (
            "i18n",
            HashSet::from(["sifr.i18n".to_string()]),
            vec![
                "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"i18n\"] }",
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
                "encoding_rs = \"0.8.35\"",
                "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"unicode\"] }",
                "unicode_names2 = \"3.1.0\"",
                "unicode-normalization = \"0.1.25\"",
                "unicode-segmentation = \"1.13.3\"",
            ],
        ),
        (
            "encoding-and-i18n",
            HashSet::from(["sifr.encoding".to_string(), "sifr.i18n".to_string()]),
            vec![
                "encoding_rs = \"0.8.35\"",
                "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"i18n\"] }",
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
                "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"i18n\", \"unicode\"] }",
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
                "encoding_rs = \"0.8.35\"",
                "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"i18n\", \"unicode\"] }",
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
