use sifr_stdlib::{generated_cargo_dependencies, StdlibFeature};
use std::collections::HashSet;

fn modules(names: &[&str]) -> HashSet<String> {
    names.iter().map(|name| (*name).to_string()).collect()
}

fn features(features: &[StdlibFeature]) -> HashSet<StdlibFeature> {
    features.iter().copied().collect()
}

fn normalize_runtime_dependency(dependency: &str) -> String {
    if dependency.starts_with("sifr_runtime = ") {
        return "sifr_runtime = { path = \"<sifr_runtime_path>\" }".to_string();
    }
    dependency.to_string()
}

#[test]
fn concurrency_runtime_dependency_snapshots_cover_m7_feature_combinations() {
    let cases = [
        (
            "tokio-runtime",
            modules(&[]),
            features(&[StdlibFeature::SifrRuntime, StdlibFeature::Tokio]),
            vec![
                "sifr_runtime = { path = \"<sifr_runtime_path>\" }",
                "tokio = { version = \"1.52.3\", features = [\"io-util\", \"macros\", \"process\", \"rt\", \"signal\", \"sync\", \"time\"] }",
            ],
        ),
        (
            "parallel",
            modules(&["sifr.parallel"]),
            features(&[]),
            vec!["rayon = \"1.12.0\""],
        ),
        (
            "runtime-diagnostics",
            modules(&["sifr.runtime"]),
            features(&[]),
            vec![
                "metrics = \"0.24.6\"",
                "tracing = { version = \"0.1.44\", default-features = false, features = [\"std\"] }",
            ],
        ),
        (
            "ipc",
            modules(&["sifr.ipc"]),
            features(&[]),
            vec![
                "postcard = { version = \"1.1.3\", default-features = false, features = [\"use-std\"] }",
                "serde = { version = \"1.0.228\", features = [\"derive\"] }",
            ],
        ),
        (
            "full-concurrency-runtime",
            modules(&["sifr.ipc", "sifr.parallel", "sifr.runtime"]),
            features(&[StdlibFeature::SifrRuntime, StdlibFeature::Tokio]),
            vec![
                "postcard = { version = \"1.1.3\", default-features = false, features = [\"use-std\"] }",
                "serde = { version = \"1.0.228\", features = [\"derive\"] }",
                "rayon = \"1.12.0\"",
                "metrics = \"0.24.6\"",
                "tracing = { version = \"0.1.44\", default-features = false, features = [\"std\"] }",
                "sifr_runtime = { path = \"<sifr_runtime_path>\" }",
                "tokio = { version = \"1.52.3\", features = [\"io-util\", \"macros\", \"process\", \"rt\", \"signal\", \"sync\", \"time\"] }",
            ],
        ),
    ];

    for (name, stdlib_modules, required_features, expected) in cases {
        let deps = generated_cargo_dependencies(&stdlib_modules, &required_features)
            .iter()
            .map(|dependency| normalize_runtime_dependency(dependency))
            .collect::<Vec<_>>();
        assert_eq!(deps, expected, "{name}");
    }
}
