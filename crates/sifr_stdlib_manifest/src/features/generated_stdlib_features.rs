use std::collections::{BTreeSet, HashSet};

use super::StdlibFeature;

#[must_use]
pub fn planned_sifr_stdlib_features(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> BTreeSet<&'static str> {
    let mut features = BTreeSet::new();
    for module_name in stdlib_modules
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
    {
        features.extend(features_for_module(module_name));
    }
    for feature in required_features.iter().copied().collect::<BTreeSet<_>>() {
        features.extend(features_for_requirement(feature));
    }
    features
}

fn features_for_module(module_name: &str) -> &'static [&'static str] {
    match module_name {
        "sifr.json" | "_sifr.json" => &["json"],
        "sifr.collections" | "_sifr.collections" => &["collections"],
        "sifr.re" | "_sifr.regex" => &["regex"],
        "sifr.uuid" | "_sifr.uuid" => &["uuid"],
        "sifr.hashlib" => &["hash"],
        "sifr.base64" => &["base64"],
        "sifr.html" | "_sifr.html" => &["html"],
        "sifr.calendar" | "_sifr.calendar" => &["calendar"],
        "sifr.env" | "sifr.sys" | "_sifr.sys" => &["sys"],
        "sifr.platform" | "_sifr.platform" => &["platform"],
        "sifr.logging" | "_sifr.logging" => &["logging"],
        "sifr.math" | "_sifr.math" => &["math"],
        "sifr.tomllib" | "_sifr.toml" => &["toml"],
        "sifr.url" | "_sifr.url" => &["url"],
        "sifr.gzip" => &["gzip"],
        "_sifr.compress" => &["gzip", "zipfile"],
        "sifr.zipfile" => &["zipfile"],
        "sifr.unicode" | "_sifr.unicode" => &["unicode"],
        "sifr.i18n" | "_sifr.i18n" => &["i18n"],
        "sifr.net" | "_sifr.net" => &["net"],
        "sifr.tls" | "_sifr.tls" => &["tls"],
        "sifr.http" | "_sifr.http" => &["http"],
        "sifr.python" | "sifr.python_core" | "_sifr.python" => &["python"],
        "sifr.process" | "_sifr.process" => &["process"],
        "sifr.os" | "sifr.shutil" => &["fs", "sys"],
        "sifr.io" | "sifr.pathlib" | "sifr.tempfile" | "_sifr.fs" => &["fs"],
        "sifr.signal" | "_sifr.signal" => &["signals"],
        "sifr.runtime" | "_sifr.runtime" => &["runtime-observability"],
        "sifr.random" => &["random"],
        "_sifr.crypto" => &["base64", "hash", "random"],
        "sifr.time" | "sifr.datetime" | "_sifr.time" | "_sifr.datetime" => &["time"],
        "sifr.encoding" | "_sifr.encoding" => &["encoding"],
        _ => &[],
    }
}

fn features_for_requirement(feature: StdlibFeature) -> &'static [&'static str] {
    match feature {
        StdlibFeature::Base64 => &["base64"],
        StdlibFeature::Blake2 | StdlibFeature::Md5 | StdlibFeature::Sha1 | StdlibFeature::Sha2 => {
            &["hash"]
        }
        StdlibFeature::Flate2 => &["gzip"],
        StdlibFeature::Fs => &["fs"],
        StdlibFeature::Http
        | StdlibFeature::HttpBody
        | StdlibFeature::HttpBodyUtil
        | StdlibFeature::Hyper
        | StdlibFeature::HyperUtil
        | StdlibFeature::H2
        | StdlibFeature::TowerService => &["http"],
        StdlibFeature::IcuCollator
        | StdlibFeature::IcuDatetime
        | StdlibFeature::IcuDecimal
        | StdlibFeature::IcuLocale
        | StdlibFeature::IcuPlurals => &["i18n"],
        StdlibFeature::PercentEncoding | StdlibFeature::Url => &["url"],
        StdlibFeature::PythonRuntime => &["python"],
        StdlibFeature::Regex => &["regex"],
        StdlibFeature::Rustls
        | StdlibFeature::RustlsPemfile
        | StdlibFeature::RustlsPlatformVerifier
        | StdlibFeature::TokioRustls => &["tls"],
        StdlibFeature::SerdeJson => &["json"],
        StdlibFeature::Sys => &["sys"],
        StdlibFeature::Toml => &["toml"],
        StdlibFeature::Tracing | StdlibFeature::Metrics => &["runtime-observability"],
        StdlibFeature::UnicodeNames
        | StdlibFeature::UnicodeNormalization
        | StdlibFeature::UnicodeSegmentation => &["unicode"],
        StdlibFeature::Uuid => &["uuid"],
        StdlibFeature::Zip => &["zipfile"],
        StdlibFeature::Chrono => &["time"],
        StdlibFeature::Rand | StdlibFeature::RandDistr => &["random"],
        StdlibFeature::BigDecimal
        | StdlibFeature::NumBigint
        | StdlibFeature::NumTraits
        | StdlibFeature::Rayon
        | StdlibFeature::RustDecimal => &["numeric"],
        StdlibFeature::EncodingRs => &[],
        StdlibFeature::Bytes
        | StdlibFeature::Cookie
        | StdlibFeature::Ipc
        | StdlibFeature::SifrRuntime
        | StdlibFeature::StructuralRuntime
        | StdlibFeature::Tokio => &[],
    }
}
