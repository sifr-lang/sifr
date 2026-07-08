use super::*;

pub(crate) fn sifr_runtime_dependency_spec_for_modules(
    stdlib_modules: &BTreeSet<String>,
) -> String {
    let mut features = Vec::new();
    let has_module = |names: &[&str]| {
        stdlib_modules
            .iter()
            .any(|module| names.contains(&module.as_str()))
    };
    let needs_net = has_module(&["sifr.net", "_sifr.net"]);
    let needs_tls = has_module(&["sifr.tls", "_sifr.tls"]);
    if needs_net || needs_tls {
        features.push("net");
    }
    if needs_tls {
        features.push("tls");
    }
    sifr_runtime_dependency_spec_with_features(&features)
}

pub(crate) fn sifr_runtime_dependency_spec_with_features(features: &[&str]) -> String {
    let runtime_path = compile_time_sifr_runtime_path();
    let escaped_path = runtime_path
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    if features.is_empty() {
        return format!("sifr_runtime = {{ path = \"{escaped_path}\" }}");
    }
    let rendered_features = features
        .iter()
        .map(|feature| format!("\"{feature}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("sifr_runtime = {{ path = \"{escaped_path}\", features = [{rendered_features}] }}")
}

pub(crate) fn sifr_stdlib_dependency_spec_for_modules(stdlib_modules: &BTreeSet<String>) -> String {
    let mut features = Vec::new();
    let has_module = |names: &[&str]| {
        stdlib_modules
            .iter()
            .any(|module| names.contains(&module.as_str()))
    };
    if has_module(&["sifr.html", "_sifr.html"]) {
        features.push("html");
    }
    if has_module(&["sifr.calendar", "_sifr.calendar"]) {
        features.push("calendar");
    }
    if has_module(&["sifr.platform", "_sifr.platform"]) {
        features.push("platform");
    }
    if has_module(&["sifr.logging", "_sifr.logging"]) {
        features.push("logging");
    }
    if has_module(&["sifr.uuid", "_sifr.uuid"]) {
        features.push("uuid");
    }
    if has_module(&["sifr.collections", "_sifr.collections"]) {
        features.push("collections");
    }
    if has_module(&["sifr.math", "_sifr.math"]) {
        features.push("math");
    }
    if has_module(&["sifr.random", "_sifr.crypto"]) {
        features.push("random");
    }
    if has_module(&["sifr.hash", "sifr.hashlib", "_sifr.crypto"]) {
        features.push("hash");
    }
    if has_module(&["sifr.base64", "_sifr.crypto"]) {
        features.push("base64");
    }
    if has_module(&["sifr.bytes", "sifr.base64", "sifr.hashlib", "_sifr.bytes"]) {
        features.push("bytes");
    }
    if has_module(&["sifr.re", "_sifr.regex"]) {
        features.push("regex");
    }
    if has_module(&["sifr.json", "_sifr.json"]) {
        features.push("json");
    }
    if has_module(&["sifr.tomllib", "_sifr.toml"]) {
        features.push("toml");
    }
    if has_module(&["sifr.url", "_sifr.url"]) {
        features.push("url");
    }
    if has_module(&["sifr.encoding", "_sifr.encoding"]) {
        features.push("encoding");
    }
    if has_module(&["sifr.io", "sifr.pathlib", "sifr.tempfile", "_sifr.fs"]) {
        features.push("fs");
    }
    if has_module(&["sifr.i18n", "_sifr.i18n"]) {
        features.push("i18n");
    }
    if has_module(&["sifr.unicode", "_sifr.unicode"]) {
        features.push("unicode");
    }
    if has_module(&["sifr.time", "_sifr.time", "sifr.datetime", "_sifr.datetime"]) {
        features.push("time");
    }
    if has_module(&["sifr.gzip", "_sifr.compress"]) {
        features.push("gzip");
    }
    if has_module(&["sifr.zipfile", "_sifr.compress"]) {
        features.push("zipfile");
    }
    sifr_stdlib_dependency_spec_with_features(&features)
}

pub(crate) fn sifr_stdlib_dependency_spec_with_features(features: &[&str]) -> String {
    let stdlib_path = compile_time_sifr_stdlib_path();
    let escaped_path = stdlib_path
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    if features.is_empty() {
        return format!("sifr_stdlib = {{ path = \"{escaped_path}\", default-features = false }}");
    }
    let rendered_features = features
        .iter()
        .map(|feature| format!("\"{feature}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "sifr_stdlib = {{ path = \"{escaped_path}\", default-features = false, features = [{rendered_features}] }}"
    )
}

pub(crate) fn tokio_dependency_spec() -> String {
    "tokio = { version = \"1.52.3\", features = [\"io-util\", \"macros\", \"net\", \"process\", \"rt\", \"signal\", \"sync\", \"time\"] }"
        .to_string()
}

pub(crate) fn compile_time_sifr_stdlib_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("sifr crate manifest directory must have workspace parent")
        .join("sifr_stdlib")
}

pub(crate) fn compile_time_sifr_runtime_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("sifr crate manifest directory must have workspace parent")
        .join("sifr_runtime")
}
