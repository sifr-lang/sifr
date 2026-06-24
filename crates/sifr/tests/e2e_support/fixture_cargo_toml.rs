use super::*;

const POSTCARD_DEP: &str =
    "postcard = { version = \"1.1.3\", default-features = false, features = [\"use-std\"] }";
const BYTES_DEP: &str = "bytes = \"1.11.1\"";
const H2_DEP: &str = "h2 = \"0.4.14\"";
const HTTP_DEP: &str = "http = \"1.4.1\"";
const HTTP_BODY_DEP: &str = "http-body = \"1.0.1\"";
const HTTP_BODY_UTIL_DEP: &str =
    "http-body-util = { version = \"0.1.3\", default-features = false }";
const HYPER_DEP: &str =
    "hyper = { version = \"1.10.1\", default-features = false, features = [\"client\", \"http1\", \"http2\", \"server\"] }";
const HYPER_UTIL_DEP: &str =
    "hyper-util = { version = \"0.1.20\", default-features = false, features = [\"tokio\"] }";
const PERCENT_ENCODING_DEP: &str = "percent-encoding = \"2.3.2\"";
const SERDE_DEP: &str = "serde = { version = \"1.0.228\", features = [\"derive\"] }";
const SERDE_JSON_DEP: &str =
    "serde_json = { version = \"1.0.149\", features = [\"preserve_order\"] }";
const TOWER_SERVICE_DEP: &str = "tower-service = \"0.3.3\"";
const TRACING_DEP: &str =
    "tracing = { version = \"0.1.44\", default-features = false, features = [\"std\"] }";
const URL_DEP: &str = "url = \"2.5.8\"";

pub(crate) fn generate_cargo_toml(
    stdlib_modules: &BTreeSet<String>,
    required_crates: &BTreeSet<String>,
    package_name: &str,
) -> String {
    let mut contents = format!(
        "[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n"
    );
    let mut deps = BTreeSet::new();
    for module_name in stdlib_modules {
        match module_name.as_str() {
            "sifr.json" | "sifr.collections" | "_sifr.json" | "_sifr.collections" => {
                deps.insert(SERDE_JSON_DEP.to_string());
                deps.insert(SERDE_DEP.to_string());
            }
            "sifr.time" | "_sifr.time" | "sifr.datetime" | "_sifr.datetime" => {
                deps.insert("chrono = \"0.4.44\"".to_string());
            }
            "sifr.random" | "_sifr.crypto" => {
                deps.insert("rand = \"0.10.1\"".to_string());
                deps.insert("rand_distr = \"0.6.0\"".to_string());
            }
            "sifr.re" | "_sifr.regex" => {
                deps.insert("regex = \"1.12.3\"".to_string());
            }
            "sifr.hash" | "sifr.hashlib" => {
                deps.insert("sha2 = \"0.11.0\"".to_string());
                deps.insert("md5 = \"0.8.0\"".to_string());
                deps.insert("sha1 = \"0.11.0\"".to_string());
                deps.insert("blake2 = \"0.10.6\"".to_string());
            }
            "sifr.encoding" | "_sifr.encoding" => {
                deps.insert("encoding_rs = \"0.8.35\"".to_string());
            }
            "sifr.unicode" | "_sifr.unicode" => {
                deps.insert("unicode_names2 = \"3.1.0\"".to_string());
                deps.insert("unicode-normalization = \"0.1.25\"".to_string());
                deps.insert("unicode-segmentation = \"1.13.3\"".to_string());
            }
            "sifr.i18n" | "_sifr.i18n" => {
                deps.insert("icu_collator = \"2.2.0\"".to_string());
                deps.insert("icu_datetime = \"2.2.0\"".to_string());
                deps.insert("icu_decimal = \"2.2.0\"".to_string());
                deps.insert("icu_locale = \"2.2.0\"".to_string());
                deps.insert("icu_plurals = \"2.2.0\"".to_string());
            }
            "sifr.base64" => {
                deps.insert("base64 = \"0.22.1\"".to_string());
            }
            "sifr.parallel" => {
                deps.insert("rayon = \"1.12.0\"".to_string());
            }
            "sifr.runtime" | "_sifr.runtime" => {
                deps.insert("metrics = \"0.24.6\"".to_string());
                deps.insert(TRACING_DEP.to_string());
            }
            "sifr.ipc" | "_sifr.ipc" => {
                deps.insert(POSTCARD_DEP.to_string());
                deps.insert(SERDE_DEP.to_string());
            }
            "sifr.tomllib" | "_sifr.toml" => {
                deps.insert(
                    "toml = { version = \"1.1.2\", features = [\"preserve_order\"] }".to_string(),
                );
            }
            "sifr.url" | "_sifr.url" => {
                deps.insert(URL_DEP.to_string());
                deps.insert(PERCENT_ENCODING_DEP.to_string());
            }
            "sifr.http" | "_sifr.http" => {
                deps.insert(HTTP_DEP.to_string());
            }
            "sifr.http_transport" => {
                deps.insert(BYTES_DEP.to_string());
                deps.insert(H2_DEP.to_string());
                deps.insert(HTTP_DEP.to_string());
                deps.insert(HTTP_BODY_DEP.to_string());
                deps.insert(HTTP_BODY_UTIL_DEP.to_string());
                deps.insert(HYPER_DEP.to_string());
                deps.insert(HYPER_UTIL_DEP.to_string());
                deps.insert("rustls = \"=0.23.35\"".to_string());
                deps.insert("rustls-pemfile = \"2.2.0\"".to_string());
                deps.insert(
                    "rustls-platform-verifier = { version = \"0.7.0\", default-features = false }"
                        .to_string(),
                );
                deps.insert(sifr_runtime_dependency_spec_with_features(&[
                    "net", "tls", "http",
                ]));
                deps.insert(tokio_dependency_spec());
                deps.insert("tokio-rustls = \"0.26.4\"".to_string());
                deps.insert(TOWER_SERVICE_DEP.to_string());
                deps.insert(TRACING_DEP.to_string());
            }
            "sifr.gzip" | "sifr.zipfile" | "_sifr.compress" => {
                deps.insert("flate2 = \"1.1.9\"".to_string());
                deps.insert("zip = \"8.6.0\"".to_string());
            }
            "_bigint" => {
                deps.insert("num-bigint = \"0.4.6\"".to_string());
                deps.insert("num-traits = \"0.2.19\"".to_string());
            }
            _ => {}
        }
    }

    if needs_sifr_runtime_module_dependency(stdlib_modules)
        && !deps
            .iter()
            .any(|dependency| dependency.starts_with("sifr_runtime = "))
    {
        deps.insert(sifr_runtime_dependency_spec_for_modules(stdlib_modules));
    }

    if needs_sifr_stdlib_module_dependency(stdlib_modules)
        && !deps
            .iter()
            .any(|dependency| dependency.starts_with("sifr_stdlib = "))
    {
        deps.insert(sifr_stdlib_dependency_spec_for_modules(stdlib_modules));
    }

    if needs_sifr_runtime_http_dependency(required_crates)
        && !deps
            .iter()
            .any(|dependency| dependency.starts_with("sifr_runtime = "))
    {
        deps.insert(sifr_runtime_dependency_spec_with_features(&[
            "net", "tls", "http",
        ]));
    }

    for crate_name in required_crates {
        match crate_name.as_str() {
            "bytes" => {
                deps.insert(BYTES_DEP.to_string());
            }
            "h2" => {
                deps.insert(H2_DEP.to_string());
            }
            "http-body" | "http_body" => {
                deps.insert(HTTP_BODY_DEP.to_string());
            }
            "http-body-util" | "http_body_util" => {
                deps.insert(HTTP_BODY_UTIL_DEP.to_string());
            }
            "hyper" => {
                deps.insert(HYPER_DEP.to_string());
            }
            "hyper-util" | "hyper_util" => {
                deps.insert(HYPER_UTIL_DEP.to_string());
            }
            "tower-service" | "tower_service" => {
                deps.insert(TOWER_SERVICE_DEP.to_string());
            }
            _ => {}
        }
    }

    for crate_name in required_crates {
        match crate_name.as_str() {
            "serde_json" => {
                deps.insert(SERDE_JSON_DEP.to_string());
                deps.insert(SERDE_DEP.to_string());
            }
            "postcard" | "ipc" => {
                deps.insert(POSTCARD_DEP.to_string());
                deps.insert(SERDE_DEP.to_string());
            }
            "chrono" => {
                deps.insert("chrono = \"0.4.44\"".to_string());
            }
            "rand" => {
                deps.insert("rand = \"0.10.1\"".to_string());
            }
            "rand_distr" => {
                deps.insert("rand_distr = \"0.6.0\"".to_string());
            }
            "regex" => {
                deps.insert("regex = \"1.12.3\"".to_string());
            }
            "sha2" => {
                deps.insert("sha2 = \"0.11.0\"".to_string());
            }
            "md5" => {
                deps.insert("md5 = \"0.8.0\"".to_string());
            }
            "sha1" => {
                deps.insert("sha1 = \"0.11.0\"".to_string());
            }
            "uuid" => {
                deps.insert(
                    "uuid = { version = \"1.23.1\", features = [\"v3\", \"v5\"] }".to_string(),
                );
            }
            "blake2" => {
                deps.insert("blake2 = \"0.10.6\"".to_string());
            }
            "base64" => {
                deps.insert("base64 = \"0.22.1\"".to_string());
            }
            "toml" => {
                deps.insert(
                    "toml = { version = \"1.1.2\", features = [\"preserve_order\"] }".to_string(),
                );
            }
            "url" => {
                deps.insert(URL_DEP.to_string());
            }
            "percent-encoding" | "percent_encoding" => {
                deps.insert(PERCENT_ENCODING_DEP.to_string());
            }
            "http" => {
                deps.insert(HTTP_DEP.to_string());
            }
            "cookie" => {}
            "flate2" => {
                deps.insert("flate2 = \"1.1.9\"".to_string());
            }
            "zip" => {
                deps.insert("zip = \"8.6.0\"".to_string());
            }
            "num-bigint" => {
                deps.insert("num-bigint = \"0.4.6\"".to_string());
            }
            "num-traits" => {
                deps.insert("num-traits = \"0.2.19\"".to_string());
            }
            "rust_decimal" => {
                deps.insert(
                    "rust_decimal = { version = \"1.41.0\", features = [\"maths\", \"serde-with-str\"] }".to_string(),
                );
            }
            "bigdecimal" => {
                deps.insert(
                    "bigdecimal = { version = \"0.4.10\", features = [\"serde\"] }".to_string(),
                );
            }
            "rayon" => {
                deps.insert("rayon = \"1.12.0\"".to_string());
            }
            "sifr_runtime" | "sifr-runtime" => {
                if !deps
                    .iter()
                    .any(|dependency| dependency.starts_with("sifr_runtime = "))
                {
                    deps.insert(sifr_runtime_dependency_spec_with_features(&[]));
                }
            }
            "tokio" => {
                deps.insert(tokio_dependency_spec());
            }
            "tokio-rustls" | "tokio_rustls" => {
                deps.insert("tokio-rustls = \"0.26.4\"".to_string());
            }
            "rustls" => {
                deps.insert("rustls = \"=0.23.35\"".to_string());
            }
            "rustls-pemfile" | "rustls_pemfile" => {
                deps.insert("rustls-pemfile = \"2.2.0\"".to_string());
            }
            "rustls-platform-verifier" | "rustls_platform_verifier" => {
                deps.insert(
                    "rustls-platform-verifier = { version = \"0.7.0\", default-features = false }"
                        .to_string(),
                );
            }
            "metrics" => {
                deps.insert("metrics = \"0.24.6\"".to_string());
            }
            "tracing" => {
                deps.insert(TRACING_DEP.to_string());
            }
            _ => {}
        }
    }

    if !deps.is_empty() {
        contents.push_str("[dependencies]\n");
        for dep in deps {
            contents.push_str(&dep);
            contents.push('\n');
        }
    }

    // Keep generated grouped crates outside of the parent workspace to avoid Cargo
    // interpreting them as non-members of the existing workspace.
    contents.push_str("\n[workspace]\n");

    contents
}

fn needs_sifr_runtime_module_dependency(stdlib_modules: &BTreeSet<String>) -> bool {
    stdlib_modules.iter().any(|module| {
        matches!(
            module.as_str(),
            "sifr.encoding"
                | "_sifr.encoding"
                | "sifr.unicode"
                | "_sifr.unicode"
                | "sifr.i18n"
                | "_sifr.i18n"
                | "sifr.net"
                | "_sifr.net"
                | "sifr.tls"
                | "_sifr.tls"
        )
    })
}

fn needs_sifr_stdlib_module_dependency(stdlib_modules: &BTreeSet<String>) -> bool {
    stdlib_modules.iter().any(|module| {
        matches!(
            module.as_str(),
            "sifr.html"
                | "_sifr.html"
                | "sifr.calendar"
                | "_sifr.calendar"
                | "sifr.uuid"
                | "_sifr.uuid"
                | "sifr.math"
                | "_sifr.math"
                | "sifr.platform"
                | "_sifr.platform"
        )
    })
}

fn needs_sifr_runtime_http_dependency(required_crates: &BTreeSet<String>) -> bool {
    required_crates.iter().any(|crate_name| {
        matches!(
            crate_name.as_str(),
            "h2" | "http-body"
                | "http_body"
                | "http-body-util"
                | "http_body_util"
                | "hyper"
                | "hyper-util"
                | "hyper_util"
                | "tower-service"
                | "tower_service"
        )
    })
}
