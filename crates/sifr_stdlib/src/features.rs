use std::collections::{BTreeSet, HashSet};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StdlibFeature {
    Base64,
    BigDecimal,
    Blake2,
    Bytes,
    Chrono,
    Cookie,
    EncodingRs,
    Flate2,
    H2,
    Http,
    HttpBody,
    HttpBodyUtil,
    Hyper,
    HyperUtil,
    IcuCollator,
    IcuDatetime,
    IcuDecimal,
    IcuLocale,
    IcuPlurals,
    Ipc,
    Md5,
    Metrics,
    NumBigint,
    NumTraits,
    PercentEncoding,
    PythonRuntime,
    Rand,
    RandDistr,
    Rayon,
    Regex,
    Rustls,
    RustlsPemfile,
    RustlsPlatformVerifier,
    RustDecimal,
    SerdeJson,
    Sha1,
    Sha2,
    SifrRuntime,
    Tokio,
    TokioRustls,
    Toml,
    TowerService,
    Tracing,
    UnicodeNames,
    UnicodeNormalization,
    UnicodeSegmentation,
    Url,
    Uuid,
    Zip,
}

impl StdlibFeature {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Base64 => "base64",
            Self::BigDecimal => "bigdecimal",
            Self::Blake2 => "blake2",
            Self::Bytes => "bytes",
            Self::Chrono => "chrono",
            Self::Cookie => "cookie",
            Self::EncodingRs => "encoding_rs",
            Self::Flate2 => "flate2",
            Self::H2 => "h2",
            Self::Http => "http",
            Self::HttpBody => "http-body",
            Self::HttpBodyUtil => "http-body-util",
            Self::Hyper => "hyper",
            Self::HyperUtil => "hyper-util",
            Self::IcuCollator => "icu_collator",
            Self::IcuDatetime => "icu_datetime",
            Self::IcuDecimal => "icu_decimal",
            Self::IcuLocale => "icu_locale",
            Self::IcuPlurals => "icu_plurals",
            Self::Ipc => "ipc",
            Self::Md5 => "md5",
            Self::Metrics => "metrics",
            Self::NumBigint => "num-bigint",
            Self::NumTraits => "num-traits",
            Self::PercentEncoding => "percent-encoding",
            Self::PythonRuntime => "sifr_runtime/python",
            Self::Rand => "rand",
            Self::RandDistr => "rand_distr",
            Self::Rayon => "rayon",
            Self::Regex => "regex",
            Self::Rustls => "rustls",
            Self::RustlsPemfile => "rustls-pemfile",
            Self::RustlsPlatformVerifier => "rustls-platform-verifier",
            Self::RustDecimal => "rust_decimal",
            Self::SerdeJson => "serde_json",
            Self::Sha1 => "sha1",
            Self::Sha2 => "sha2",
            Self::SifrRuntime => "sifr_runtime",
            Self::Tokio => "tokio",
            Self::TokioRustls => "tokio-rustls",
            Self::Toml => "toml",
            Self::TowerService => "tower-service",
            Self::Tracing => "tracing",
            Self::UnicodeNames => "unicode_names2",
            Self::UnicodeNormalization => "unicode-normalization",
            Self::UnicodeSegmentation => "unicode-segmentation",
            Self::Url => "url",
            Self::Uuid => "uuid",
            Self::Zip => "zip",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GeneratedCargoDependency {
    pub package: &'static str,
    pub spec: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StdlibFeatureSpec {
    pub feature: StdlibFeature,
    pub cargo_dependencies: &'static [GeneratedCargoDependency],
}

const BASE64_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "base64",
    spec: "base64 = \"0.22.1\"",
}];
const BIGDECIMAL_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "bigdecimal",
    spec: "bigdecimal = { version = \"0.4.10\", features = [\"serde\"] }",
}];
const BLAKE2_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "blake2",
    spec: "blake2 = \"0.10.6\"",
}];
const BYTES_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "bytes",
    spec: "bytes = \"1.11.1\"",
}];
const CHRONO_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "chrono",
    spec: "chrono = \"0.4.44\"",
}];
// Cookie-header helpers are Sifr-owned string/header validation; no cookie jar
// or signing dependency is emitted for the URL-header-cookie substrate.
const COOKIE_DEPS: &[GeneratedCargoDependency] = &[];
const ENCODING_RS_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "encoding_rs",
    spec: "encoding_rs = \"0.8.35\"",
}];
const FLATE2_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "flate2",
    spec: "flate2 = \"1.1.9\"",
}];
const H2_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "h2",
    spec: "h2 = \"0.4.14\"",
}];
const HTTP_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "http",
    spec: "http = \"1.4.1\"",
}];
const HTTP_BODY_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "http-body",
    spec: "http-body = \"1.0.1\"",
}];
const HTTP_BODY_UTIL_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "http-body-util",
    spec: "http-body-util = { version = \"0.1.3\", default-features = false }",
}];
const HYPER_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "hyper",
    spec: "hyper = { version = \"1.10.1\", default-features = false, features = [\"client\", \"http1\", \"http2\", \"server\"] }",
}];
const HYPER_UTIL_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "hyper-util",
    spec: "hyper-util = { version = \"0.1.20\", default-features = false, features = [\"tokio\"] }",
}];
const ICU_COLLATOR_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "icu_collator",
    spec: "icu_collator = \"2.2.0\"",
}];
const ICU_DATETIME_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "icu_datetime",
    spec: "icu_datetime = \"2.2.0\"",
}];
const ICU_DECIMAL_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "icu_decimal",
    spec: "icu_decimal = \"2.2.0\"",
}];
const ICU_LOCALE_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "icu_locale",
    spec: "icu_locale = \"2.2.0\"",
}];
const ICU_PLURALS_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "icu_plurals",
    spec: "icu_plurals = \"2.2.0\"",
}];
const IPC_DEPS: &[GeneratedCargoDependency] = &[
    GeneratedCargoDependency {
        package: "postcard",
        spec:
            "postcard = { version = \"1.1.3\", default-features = false, features = [\"use-std\"] }",
    },
    GeneratedCargoDependency {
        package: "serde",
        spec: "serde = { version = \"1.0.228\", features = [\"derive\"] }",
    },
];
const MD5_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "md5",
    spec: "md5 = \"0.8.0\"",
}];
const METRICS_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "metrics",
    spec: "metrics = \"0.24.6\"",
}];
const NUM_BIGINT_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "num-bigint",
    spec: "num-bigint = \"0.4.6\"",
}];
const NUM_TRAITS_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "num-traits",
    spec: "num-traits = \"0.2.19\"",
}];
const PERCENT_ENCODING_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "percent-encoding",
    spec: "percent-encoding = \"2.3.2\"",
}];
const PYTHON_RUNTIME_DEPS: &[GeneratedCargoDependency] = SIFR_RUNTIME_DEPS;
const RAND_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "rand",
    spec: "rand = \"0.10.1\"",
}];
const RAND_DISTR_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "rand_distr",
    spec: "rand_distr = \"0.6.0\"",
}];
const RAYON_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "rayon",
    spec: "rayon = \"1.12.0\"",
}];
const REGEX_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "regex",
    spec: "regex = \"1.12.3\"",
}];
const RUSTLS_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "rustls",
    spec: "rustls = \"=0.23.35\"",
}];
const RUSTLS_PEMFILE_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "rustls-pemfile",
    spec: "rustls-pemfile = \"2.2.0\"",
}];
const RUSTLS_PLATFORM_VERIFIER_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "rustls-platform-verifier",
    spec: "rustls-platform-verifier = { version = \"0.7.0\", default-features = false }",
}];
const RUST_DECIMAL_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "rust_decimal",
    spec: "rust_decimal = { version = \"1.41.0\", features = [\"maths\", \"serde-with-str\"] }",
}];
const SERDE_JSON_DEPS: &[GeneratedCargoDependency] = &[
    GeneratedCargoDependency {
        package: "serde_json",
        spec: "serde_json = { version = \"1.0.149\", features = [\"preserve_order\"] }",
    },
    GeneratedCargoDependency {
        package: "serde",
        spec: "serde = { version = \"1.0.228\", features = [\"derive\"] }",
    },
];
const SHA1_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "sha1",
    spec: "sha1 = \"0.11.0\"",
}];
const SHA2_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "sha2",
    spec: "sha2 = \"0.11.0\"",
}];
const SIFR_RUNTIME_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "sifr_runtime",
    spec: "sifr_runtime",
}];
const TOKIO_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "tokio",
    spec: "tokio = { version = \"1.52.3\", features = [\"io-util\", \"macros\", \"process\", \"rt\", \"signal\", \"sync\", \"time\"] }",
}];
const TOKIO_RUSTLS_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "tokio-rustls",
    spec: "tokio-rustls = \"0.26.4\"",
}];
const TOML_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "toml",
    spec: "toml = { version = \"1.1.2\", features = [\"preserve_order\"] }",
}];
const TOWER_SERVICE_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "tower-service",
    spec: "tower-service = \"0.3.3\"",
}];
const TRACING_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "tracing",
    spec: "tracing = { version = \"0.1.44\", default-features = false, features = [\"std\"] }",
}];
const UNICODE_NAMES_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "unicode_names2",
    spec: "unicode_names2 = \"3.1.0\"",
}];
const UNICODE_NORMALIZATION_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "unicode-normalization",
    spec: "unicode-normalization = \"0.1.25\"",
}];
const UNICODE_SEGMENTATION_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "unicode-segmentation",
    spec: "unicode-segmentation = \"1.13.3\"",
}];
const URL_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "url",
    spec: "url = \"2.5.8\"",
}];
const UUID_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "uuid",
    spec: "uuid = { version = \"1.23.1\", features = [\"v3\", \"v5\"] }",
}];
const ZIP_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "zip",
    spec: "zip = \"8.6.0\"",
}];

pub const STDLIB_FEATURE_SPECS: &[StdlibFeatureSpec] = &[
    StdlibFeatureSpec {
        feature: StdlibFeature::Base64,
        cargo_dependencies: BASE64_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::BigDecimal,
        cargo_dependencies: BIGDECIMAL_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Blake2,
        cargo_dependencies: BLAKE2_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Bytes,
        cargo_dependencies: BYTES_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Chrono,
        cargo_dependencies: CHRONO_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Cookie,
        cargo_dependencies: COOKIE_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::EncodingRs,
        cargo_dependencies: ENCODING_RS_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Flate2,
        cargo_dependencies: FLATE2_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::H2,
        cargo_dependencies: H2_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Http,
        cargo_dependencies: HTTP_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::HttpBody,
        cargo_dependencies: HTTP_BODY_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::HttpBodyUtil,
        cargo_dependencies: HTTP_BODY_UTIL_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Hyper,
        cargo_dependencies: HYPER_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::HyperUtil,
        cargo_dependencies: HYPER_UTIL_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::IcuCollator,
        cargo_dependencies: ICU_COLLATOR_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::IcuDatetime,
        cargo_dependencies: ICU_DATETIME_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::IcuDecimal,
        cargo_dependencies: ICU_DECIMAL_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::IcuLocale,
        cargo_dependencies: ICU_LOCALE_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::IcuPlurals,
        cargo_dependencies: ICU_PLURALS_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Ipc,
        cargo_dependencies: IPC_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Md5,
        cargo_dependencies: MD5_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Metrics,
        cargo_dependencies: METRICS_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::NumBigint,
        cargo_dependencies: NUM_BIGINT_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::NumTraits,
        cargo_dependencies: NUM_TRAITS_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::PercentEncoding,
        cargo_dependencies: PERCENT_ENCODING_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::PythonRuntime,
        cargo_dependencies: PYTHON_RUNTIME_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Rand,
        cargo_dependencies: RAND_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::RandDistr,
        cargo_dependencies: RAND_DISTR_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Rayon,
        cargo_dependencies: RAYON_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Regex,
        cargo_dependencies: REGEX_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Rustls,
        cargo_dependencies: RUSTLS_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::RustlsPemfile,
        cargo_dependencies: RUSTLS_PEMFILE_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::RustlsPlatformVerifier,
        cargo_dependencies: RUSTLS_PLATFORM_VERIFIER_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::RustDecimal,
        cargo_dependencies: RUST_DECIMAL_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::SerdeJson,
        cargo_dependencies: SERDE_JSON_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Sha1,
        cargo_dependencies: SHA1_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Sha2,
        cargo_dependencies: SHA2_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::SifrRuntime,
        cargo_dependencies: SIFR_RUNTIME_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Tokio,
        cargo_dependencies: TOKIO_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::TokioRustls,
        cargo_dependencies: TOKIO_RUSTLS_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Toml,
        cargo_dependencies: TOML_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::TowerService,
        cargo_dependencies: TOWER_SERVICE_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Tracing,
        cargo_dependencies: TRACING_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::UnicodeNames,
        cargo_dependencies: UNICODE_NAMES_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::UnicodeNormalization,
        cargo_dependencies: UNICODE_NORMALIZATION_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::UnicodeSegmentation,
        cargo_dependencies: UNICODE_SEGMENTATION_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Url,
        cargo_dependencies: URL_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Uuid,
        cargo_dependencies: UUID_DEPS,
    },
    StdlibFeatureSpec {
        feature: StdlibFeature::Zip,
        cargo_dependencies: ZIP_DEPS,
    },
];

#[must_use]
pub fn feature_for_codegen_requirement(name: &str) -> Option<StdlibFeature> {
    match name {
        "base64" => Some(StdlibFeature::Base64),
        "bigdecimal" => Some(StdlibFeature::BigDecimal),
        "blake2" => Some(StdlibFeature::Blake2),
        "bytes" => Some(StdlibFeature::Bytes),
        "chrono" => Some(StdlibFeature::Chrono),
        "cookie" => Some(StdlibFeature::Cookie),
        "encoding_rs" | "encoding-rs" => Some(StdlibFeature::EncodingRs),
        "flate2" => Some(StdlibFeature::Flate2),
        "h2" => Some(StdlibFeature::H2),
        "http" => Some(StdlibFeature::Http),
        "http-body" | "http_body" => Some(StdlibFeature::HttpBody),
        "http-body-util" | "http_body_util" => Some(StdlibFeature::HttpBodyUtil),
        "hyper" => Some(StdlibFeature::Hyper),
        "hyper-util" | "hyper_util" => Some(StdlibFeature::HyperUtil),
        "icu_collator" | "icu-collator" => Some(StdlibFeature::IcuCollator),
        "icu_datetime" | "icu-datetime" => Some(StdlibFeature::IcuDatetime),
        "icu_decimal" | "icu-decimal" => Some(StdlibFeature::IcuDecimal),
        "icu_locale" | "icu-locale" => Some(StdlibFeature::IcuLocale),
        "icu_plurals" | "icu-plurals" => Some(StdlibFeature::IcuPlurals),
        "ipc" | "postcard" => Some(StdlibFeature::Ipc),
        "md5" => Some(StdlibFeature::Md5),
        "metrics" => Some(StdlibFeature::Metrics),
        "num-bigint" => Some(StdlibFeature::NumBigint),
        "num-traits" => Some(StdlibFeature::NumTraits),
        "percent-encoding" | "percent_encoding" => Some(StdlibFeature::PercentEncoding),
        "sifr_runtime/python" | "sifr-runtime/python" | "python-runtime" => {
            Some(StdlibFeature::PythonRuntime)
        }
        "rand" => Some(StdlibFeature::Rand),
        "rand_distr" => Some(StdlibFeature::RandDistr),
        "rayon" => Some(StdlibFeature::Rayon),
        "regex" => Some(StdlibFeature::Regex),
        "rustls" => Some(StdlibFeature::Rustls),
        "rustls-pemfile" | "rustls_pemfile" => Some(StdlibFeature::RustlsPemfile),
        "rustls-platform-verifier" | "rustls_platform_verifier" => {
            Some(StdlibFeature::RustlsPlatformVerifier)
        }
        "rust_decimal" => Some(StdlibFeature::RustDecimal),
        "serde_json" => Some(StdlibFeature::SerdeJson),
        "sha1" => Some(StdlibFeature::Sha1),
        "sha2" => Some(StdlibFeature::Sha2),
        "sifr_runtime" | "sifr-runtime" => Some(StdlibFeature::SifrRuntime),
        "tokio" => Some(StdlibFeature::Tokio),
        "tokio-rustls" | "tokio_rustls" => Some(StdlibFeature::TokioRustls),
        "toml" => Some(StdlibFeature::Toml),
        "tower-service" | "tower_service" => Some(StdlibFeature::TowerService),
        "tracing" => Some(StdlibFeature::Tracing),
        "unicode_names2" => Some(StdlibFeature::UnicodeNames),
        "unicode-normalization" | "unicode_normalization" => {
            Some(StdlibFeature::UnicodeNormalization)
        }
        "unicode-segmentation" | "unicode_segmentation" => Some(StdlibFeature::UnicodeSegmentation),
        "url" => Some(StdlibFeature::Url),
        "uuid" => Some(StdlibFeature::Uuid),
        "zip" => Some(StdlibFeature::Zip),
        _ => None,
    }
}

#[must_use]
pub fn features_for_stdlib_module(module_name: &str) -> &'static [StdlibFeature] {
    match module_name {
        "sifr.json" | "sifr.collections" | "_sifr.json" | "_sifr.collections" => {
            &[StdlibFeature::SerdeJson]
        }
        "sifr.time" | "_sifr.time" => &[StdlibFeature::Chrono],
        "sifr.random" | "_sifr.crypto" => &[StdlibFeature::Rand, StdlibFeature::RandDistr],
        "sifr.uuid" | "_sifr.uuid" => &[StdlibFeature::Rand, StdlibFeature::Uuid],
        "sifr.re" | "_sifr.regex" | "sifr.pathlib" => &[StdlibFeature::Regex],
        "sifr.hash" | "sifr.hashlib" => &[
            StdlibFeature::Sha2,
            StdlibFeature::Md5,
            StdlibFeature::Sha1,
            StdlibFeature::Blake2,
        ],
        "sifr.encoding" | "_sifr.encoding" => {
            &[StdlibFeature::EncodingRs, StdlibFeature::SifrRuntime]
        }
        "sifr.unicode" | "_sifr.unicode" => &[
            StdlibFeature::SifrRuntime,
            StdlibFeature::UnicodeNames,
            StdlibFeature::UnicodeNormalization,
            StdlibFeature::UnicodeSegmentation,
        ],
        "sifr.i18n" | "_sifr.i18n" => &[
            StdlibFeature::SifrRuntime,
            StdlibFeature::IcuCollator,
            StdlibFeature::IcuDatetime,
            StdlibFeature::IcuDecimal,
            StdlibFeature::IcuLocale,
            StdlibFeature::IcuPlurals,
        ],
        "sifr.base64" => &[StdlibFeature::Base64],
        "sifr.ipc" | "_sifr.ipc" => &[StdlibFeature::Ipc],
        "sifr.net" | "_sifr.net" => &[
            StdlibFeature::SifrRuntime,
            StdlibFeature::Tokio,
            StdlibFeature::Tracing,
        ],
        "sifr.tls" | "_sifr.tls" => &[
            StdlibFeature::SifrRuntime,
            StdlibFeature::Tokio,
            StdlibFeature::TokioRustls,
            StdlibFeature::Rustls,
            StdlibFeature::RustlsPemfile,
            StdlibFeature::RustlsPlatformVerifier,
            StdlibFeature::Tracing,
        ],
        "sifr.url" | "_sifr.url" => &[StdlibFeature::Url, StdlibFeature::PercentEncoding],
        "sifr.http" | "_sifr.http" => &[StdlibFeature::Http],
        "sifr.http_transport" => &[
            StdlibFeature::SifrRuntime,
            StdlibFeature::Tokio,
            StdlibFeature::TokioRustls,
            StdlibFeature::Rustls,
            StdlibFeature::RustlsPemfile,
            StdlibFeature::RustlsPlatformVerifier,
            StdlibFeature::Tracing,
            StdlibFeature::Bytes,
            StdlibFeature::Http,
            StdlibFeature::HttpBody,
            StdlibFeature::HttpBodyUtil,
            StdlibFeature::Hyper,
            StdlibFeature::HyperUtil,
            StdlibFeature::H2,
            StdlibFeature::TowerService,
        ],
        "sifr.parallel" => &[StdlibFeature::Rayon],
        "sifr.runtime" | "_sifr.runtime" => &[StdlibFeature::Metrics, StdlibFeature::Tracing],
        "sifr.tomllib" | "_sifr.toml" => &[StdlibFeature::Toml],
        "sifr.datetime" | "_sifr.datetime" => &[StdlibFeature::Chrono],
        "sifr.gzip" | "sifr.zipfile" | "_sifr.compress" => {
            &[StdlibFeature::Flate2, StdlibFeature::Zip]
        }
        "_bigint" => &[StdlibFeature::NumBigint, StdlibFeature::NumTraits],
        _ => &[],
    }
}

#[must_use]
pub fn generated_cargo_dependencies(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> Vec<String> {
    let mut deps = Vec::new();
    let mut packages = BTreeSet::new();
    let runtime_features = RuntimeFeatures::from_requirements(stdlib_modules, required_features);

    for module_name in stdlib_modules
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
    {
        for feature in features_for_stdlib_module(module_name) {
            push_feature_dependencies(&mut deps, &mut packages, *feature, runtime_features);
        }
    }

    for feature in required_features.iter().copied().collect::<BTreeSet<_>>() {
        push_feature_dependencies(&mut deps, &mut packages, feature, runtime_features);
    }

    deps
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RuntimeFeatures {
    http: bool,
    i18n: bool,
    net: bool,
    python: bool,
    tls: bool,
    unicode: bool,
}

impl RuntimeFeatures {
    fn from_requirements(
        stdlib_modules: &HashSet<String>,
        required_features: &HashSet<StdlibFeature>,
    ) -> Self {
        Self {
            http: needs_sifr_runtime_http(stdlib_modules, required_features),
            i18n: needs_sifr_runtime_i18n(stdlib_modules, required_features),
            net: needs_sifr_runtime_net(stdlib_modules),
            python: required_features.contains(&StdlibFeature::PythonRuntime),
            tls: needs_sifr_runtime_tls(stdlib_modules, required_features),
            unicode: needs_sifr_runtime_unicode(stdlib_modules, required_features),
        }
    }
}

fn needs_sifr_runtime_http(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> bool {
    stdlib_modules
        .iter()
        .any(|module| module.as_str() == "sifr.http_transport")
        || required_features.contains(&StdlibFeature::Hyper)
        || required_features.contains(&StdlibFeature::HyperUtil)
        || required_features.contains(&StdlibFeature::H2)
        || required_features.contains(&StdlibFeature::HttpBody)
        || required_features.contains(&StdlibFeature::HttpBodyUtil)
        || required_features.contains(&StdlibFeature::TowerService)
}

fn needs_sifr_runtime_net(stdlib_modules: &HashSet<String>) -> bool {
    stdlib_modules
        .iter()
        .any(|module| matches!(module.as_str(), "sifr.net" | "_sifr.net"))
}

fn needs_sifr_runtime_tls(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> bool {
    stdlib_modules
        .iter()
        .any(|module| matches!(module.as_str(), "sifr.tls" | "_sifr.tls"))
        || required_features.contains(&StdlibFeature::Rustls)
        || required_features.contains(&StdlibFeature::RustlsPemfile)
        || required_features.contains(&StdlibFeature::RustlsPlatformVerifier)
        || required_features.contains(&StdlibFeature::TokioRustls)
}

fn needs_sifr_runtime_i18n(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> bool {
    stdlib_modules
        .iter()
        .any(|module| matches!(module.as_str(), "sifr.i18n" | "_sifr.i18n"))
        || required_features.contains(&StdlibFeature::IcuCollator)
        || required_features.contains(&StdlibFeature::IcuDatetime)
        || required_features.contains(&StdlibFeature::IcuDecimal)
        || required_features.contains(&StdlibFeature::IcuLocale)
        || required_features.contains(&StdlibFeature::IcuPlurals)
}

fn needs_sifr_runtime_unicode(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> bool {
    stdlib_modules
        .iter()
        .any(|module| matches!(module.as_str(), "sifr.unicode" | "_sifr.unicode"))
        || required_features.contains(&StdlibFeature::UnicodeNames)
        || required_features.contains(&StdlibFeature::UnicodeNormalization)
        || required_features.contains(&StdlibFeature::UnicodeSegmentation)
}

fn push_feature_dependencies(
    deps: &mut Vec<String>,
    packages: &mut BTreeSet<&'static str>,
    feature: StdlibFeature,
    runtime_features: RuntimeFeatures,
) {
    if let Some(spec) = STDLIB_FEATURE_SPECS
        .iter()
        .find(|spec| spec.feature == feature)
    {
        for dependency in spec.cargo_dependencies {
            if packages.insert(dependency.package) {
                deps.push(render_dependency_spec(dependency, runtime_features));
            }
        }
    }
}

fn render_dependency_spec(
    dependency: &GeneratedCargoDependency,
    runtime_features: RuntimeFeatures,
) -> String {
    if dependency.package == "sifr_runtime" {
        return sifr_runtime_dependency_spec(runtime_features);
    }
    if dependency.package == "tokio" {
        return tokio_dependency_spec(runtime_features);
    }
    dependency.spec.to_string()
}

fn tokio_dependency_spec(runtime_features: RuntimeFeatures) -> String {
    let features = if runtime_features.net || runtime_features.tls || runtime_features.http {
        "\"io-util\", \"macros\", \"net\", \"process\", \"rt\", \"signal\", \"sync\", \"time\""
    } else {
        "\"io-util\", \"macros\", \"process\", \"rt\", \"signal\", \"sync\", \"time\""
    };
    format!("tokio = {{ version = \"1.52.3\", features = [{features}] }}")
}

fn sifr_runtime_dependency_spec(runtime_features: RuntimeFeatures) -> String {
    let runtime_path = discover_sifr_runtime_path().unwrap_or_else(compile_time_sifr_runtime_path);
    let escaped_path = runtime_path
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let mut features = Vec::new();
    if runtime_features.i18n {
        features.push("\"i18n\"");
    }
    if runtime_features.net || runtime_features.tls || runtime_features.http {
        features.push("\"net\"");
    }
    if runtime_features.python {
        features.push("\"python\"");
    }
    if runtime_features.tls || runtime_features.http {
        features.push("\"tls\"");
    }
    if runtime_features.http {
        features.push("\"http\"");
    }
    if runtime_features.unicode {
        features.push("\"unicode\"");
    }
    if features.is_empty() {
        return format!("sifr_runtime = {{ path = \"{escaped_path}\" }}");
    }
    format!(
        "sifr_runtime = {{ path = \"{escaped_path}\", features = [{}] }}",
        features.join(", ")
    )
}

fn discover_sifr_runtime_path() -> Option<PathBuf> {
    env::var_os("SIFR_RUNTIME_PATH")
        .map(PathBuf::from)
        .filter(|path| path.join("Cargo.toml").is_file())
        .or_else(discover_sifr_runtime_path_from_current_dir)
        .or_else(discover_sifr_runtime_path_from_current_exe)
}

fn discover_sifr_runtime_path_from_current_dir() -> Option<PathBuf> {
    env::current_dir()
        .ok()
        .and_then(|dir| find_sifr_runtime_ancestor(&dir))
}

fn discover_sifr_runtime_path_from_current_exe() -> Option<PathBuf> {
    env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .and_then(|dir| find_sifr_runtime_ancestor(&dir))
}

fn find_sifr_runtime_ancestor(start: &Path) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        let candidate = ancestor.join("crates").join("sifr_runtime");
        if candidate.join("Cargo.toml").is_file() {
            return Some(candidate);
        }
    }
    None
}

fn compile_time_sifr_runtime_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    match manifest_dir.parent() {
        Some(parent) => parent.join("sifr_runtime"),
        None => manifest_dir.join("../sifr_runtime"),
    }
}
