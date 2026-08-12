use std::fmt::Write as _;
use std::path::Path;

const SIFR_GIT_SOURCE: &str = "https://github.com/sifr-lang/sifr.git";

pub(super) fn probe_cargo_toml(
    dependency_name: &str,
    cargo_package_name: &str,
    backend_root: &Path,
    sysroot_runtime_crate: &Path,
    dependency_features: &[String],
    requires_structural_runtime: bool,
) -> String {
    let mut cargo_toml =
        "[package]\nname = \"sifr-rust-probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\n"
            .to_string();
    let mut dependency_features = dependency_features.to_vec();
    if dependency_name == "sifr_runtime"
        && requires_structural_runtime
        && !dependency_features
            .iter()
            .any(|feature| feature == "structural")
    {
        dependency_features.push("structural".to_string());
    }
    cargo_toml.push_str(&dependency_line(
        dependency_name,
        cargo_package_name,
        backend_root,
        &dependency_features,
    ));
    if dependency_name != "sifr_runtime" {
        let path = toml_quote_path(sysroot_runtime_crate);
        if requires_structural_runtime {
            let _ = writeln!(
                cargo_toml,
                "sifr_runtime = {{ path = {path}, features = [\"structural\"] }}"
            );
        } else {
            let _ = writeln!(cargo_toml, "sifr_runtime = {{ path = {path} }}");
        }
    }
    if !matches!(dependency_name, "sifr_runtime" | "sifr_stdlib") {
        let runtime_path = toml_quote_path(sysroot_runtime_crate);
        let _ = writeln!(
            cargo_toml,
            "\n[patch.{source}]\nsifr_runtime = {{ path = {runtime_path} }}",
            source = toml_quote_string(SIFR_GIT_SOURCE),
        );
    }
    cargo_toml
}

pub(super) fn probe_cargo_vendor_args(vendor_dir: Option<&Path>) -> Vec<String> {
    let Some(vendor_dir) = vendor_dir else {
        return Vec::new();
    };
    vec![
        "--config".to_string(),
        "source.crates-io.replace-with=\"sifr-vendor\"".to_string(),
        "--config".to_string(),
        format!(
            "source.sifr-vendor.directory={}",
            toml_quote_string(&vendor_dir.display().to_string())
        ),
    ]
}

fn dependency_line(
    dependency_name: &str,
    cargo_package_name: &str,
    backend_root: &Path,
    features: &[String],
) -> String {
    let backend_root = toml_quote_path(backend_root);
    let package = if dependency_name == cargo_package_name {
        String::new()
    } else {
        format!(", package = {}", toml_quote_string(cargo_package_name))
    };
    let default_features = if dependency_name == "sifr_stdlib" {
        ", default-features = false"
    } else {
        ""
    };
    if features.is_empty() {
        return format!(
            "{dependency_name} = {{ path = {backend_root}{package}{default_features} }}\n"
        );
    }
    let features = features
        .iter()
        .map(|feature| toml_quote_string(feature))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{dependency_name} = {{ path = {backend_root}{package}{default_features}, features = [{features}] }}\n"
    )
}

fn toml_quote_path(path: &Path) -> String {
    toml_quote_string(&path.display().to_string())
}

pub(super) fn toml_quote_string(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            ch => quoted.push(ch),
        }
    }
    quoted.push('"');
    quoted
}
