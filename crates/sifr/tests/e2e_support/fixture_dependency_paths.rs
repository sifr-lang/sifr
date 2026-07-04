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
    if has_module(&["sifr.i18n", "_sifr.i18n"]) {
        features.push("i18n");
    }
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
    let runtime_path = discover_sifr_runtime_path().unwrap_or_else(compile_time_sifr_runtime_path);
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
    if has_module(&["sifr.uuid", "_sifr.uuid"]) {
        features.push("uuid");
    }
    if has_module(&["sifr.math", "_sifr.math"]) {
        features.push("math");
    }
    if has_module(&["sifr.hash", "sifr.hashlib", "_sifr.crypto"]) {
        features.push("hash");
    }
    if has_module(&["sifr.base64", "_sifr.crypto"]) {
        features.push("base64");
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
    if has_module(&["sifr.i18n", "_sifr.i18n"]) {
        features.push("i18n");
    }
    if has_module(&["sifr.unicode", "_sifr.unicode"]) {
        features.push("unicode");
    }
    sifr_stdlib_dependency_spec_with_features(&features)
}

pub(crate) fn sifr_stdlib_dependency_spec_with_features(features: &[&str]) -> String {
    let stdlib_path = discover_sifr_stdlib_path().unwrap_or_else(compile_time_sifr_stdlib_path);
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

pub(crate) fn discover_sifr_stdlib_path() -> Option<PathBuf> {
    env::var_os("SIFR_STDLIB_PATH")
        .map(PathBuf::from)
        .filter(|path| path.join("Cargo.toml").is_file())
        .or_else(discover_sifr_stdlib_path_from_current_dir)
        .or_else(discover_sifr_stdlib_path_from_current_exe)
}

pub(crate) fn discover_sifr_stdlib_path_from_current_dir() -> Option<PathBuf> {
    env::current_dir()
        .ok()
        .and_then(|path| discover_sifr_stdlib_path_from_ancestors(&path))
}

pub(crate) fn discover_sifr_stdlib_path_from_current_exe() -> Option<PathBuf> {
    env::current_exe()
        .ok()
        .and_then(|path| discover_sifr_stdlib_path_from_ancestors(&path))
}

pub(crate) fn discover_sifr_stdlib_path_from_ancestors(start: &Path) -> Option<PathBuf> {
    start.ancestors().find_map(|ancestor| {
        let candidate = ancestor.join("crates").join("sifr_stdlib");
        candidate.join("Cargo.toml").is_file().then_some(candidate)
    })
}

pub(crate) fn compile_time_sifr_stdlib_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")))
        .join("sifr_stdlib")
}

pub(crate) fn discover_sifr_runtime_path() -> Option<PathBuf> {
    env::var_os("SIFR_RUNTIME_PATH")
        .map(PathBuf::from)
        .filter(|path| path.join("Cargo.toml").is_file())
        .or_else(discover_sifr_runtime_path_from_current_dir)
        .or_else(discover_sifr_runtime_path_from_current_exe)
}

pub(crate) fn discover_sifr_runtime_path_from_current_dir() -> Option<PathBuf> {
    env::current_dir()
        .ok()
        .and_then(|path| discover_sifr_runtime_path_from_ancestors(&path))
}

pub(crate) fn discover_sifr_runtime_path_from_current_exe() -> Option<PathBuf> {
    env::current_exe()
        .ok()
        .and_then(|path| discover_sifr_runtime_path_from_ancestors(&path))
}

pub(crate) fn discover_sifr_runtime_path_from_ancestors(start: &Path) -> Option<PathBuf> {
    start.ancestors().find_map(|ancestor| {
        let candidate = ancestor.join("crates").join("sifr_runtime");
        candidate.join("Cargo.toml").is_file().then_some(candidate)
    })
}

pub(crate) fn compile_time_sifr_runtime_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")))
        .join("sifr_runtime")
}
