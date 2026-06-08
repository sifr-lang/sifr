use std::collections::{BTreeSet, HashSet};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StdlibFeature {
    Base64,
    BigDecimal,
    Blake2,
    Chrono,
    EncodingRs,
    Flate2,
    IcuCollator,
    IcuDatetime,
    IcuDecimal,
    IcuLocale,
    IcuPlurals,
    Md5,
    NumBigint,
    NumTraits,
    Rand,
    RandDistr,
    Rayon,
    Regex,
    RustDecimal,
    SerdeJson,
    Sha1,
    Sha2,
    SifrRuntime,
    Tokio,
    Toml,
    UnicodeNames,
    UnicodeNormalization,
    UnicodeSegmentation,
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
            Self::Chrono => "chrono",
            Self::EncodingRs => "encoding_rs",
            Self::Flate2 => "flate2",
            Self::IcuCollator => "icu_collator",
            Self::IcuDatetime => "icu_datetime",
            Self::IcuDecimal => "icu_decimal",
            Self::IcuLocale => "icu_locale",
            Self::IcuPlurals => "icu_plurals",
            Self::Md5 => "md5",
            Self::NumBigint => "num-bigint",
            Self::NumTraits => "num-traits",
            Self::Rand => "rand",
            Self::RandDistr => "rand_distr",
            Self::Rayon => "rayon",
            Self::Regex => "regex",
            Self::RustDecimal => "rust_decimal",
            Self::SerdeJson => "serde_json",
            Self::Sha1 => "sha1",
            Self::Sha2 => "sha2",
            Self::SifrRuntime => "sifr_runtime",
            Self::Tokio => "tokio",
            Self::Toml => "toml",
            Self::UnicodeNames => "unicode_names2",
            Self::UnicodeNormalization => "unicode-normalization",
            Self::UnicodeSegmentation => "unicode-segmentation",
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
const CHRONO_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "chrono",
    spec: "chrono = \"0.4.44\"",
}];
const ENCODING_RS_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "encoding_rs",
    spec: "encoding_rs = \"0.8.35\"",
}];
const FLATE2_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "flate2",
    spec: "flate2 = \"1.1.9\"",
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
const MD5_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "md5",
    spec: "md5 = \"0.8.0\"",
}];
const NUM_BIGINT_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "num-bigint",
    spec: "num-bigint = \"0.4.6\"",
}];
const NUM_TRAITS_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "num-traits",
    spec: "num-traits = \"0.2.19\"",
}];
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
    spec: "tokio = { version = \"1.52.3\", features = [\"io-util\", \"macros\", \"process\", \"rt\", \"sync\", \"time\"] }",
}];
const TOML_DEPS: &[GeneratedCargoDependency] = &[GeneratedCargoDependency {
    package: "toml",
    spec: "toml = { version = \"1.1.2\", features = [\"preserve_order\"] }",
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
        feature: StdlibFeature::Chrono,
        cargo_dependencies: CHRONO_DEPS,
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
        feature: StdlibFeature::Md5,
        cargo_dependencies: MD5_DEPS,
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
        feature: StdlibFeature::Toml,
        cargo_dependencies: TOML_DEPS,
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
        "chrono" => Some(StdlibFeature::Chrono),
        "encoding_rs" | "encoding-rs" => Some(StdlibFeature::EncodingRs),
        "flate2" => Some(StdlibFeature::Flate2),
        "icu_collator" | "icu-collator" => Some(StdlibFeature::IcuCollator),
        "icu_datetime" | "icu-datetime" => Some(StdlibFeature::IcuDatetime),
        "icu_decimal" | "icu-decimal" => Some(StdlibFeature::IcuDecimal),
        "icu_locale" | "icu-locale" => Some(StdlibFeature::IcuLocale),
        "icu_plurals" | "icu-plurals" => Some(StdlibFeature::IcuPlurals),
        "md5" => Some(StdlibFeature::Md5),
        "num-bigint" => Some(StdlibFeature::NumBigint),
        "num-traits" => Some(StdlibFeature::NumTraits),
        "rand" => Some(StdlibFeature::Rand),
        "rand_distr" => Some(StdlibFeature::RandDistr),
        "rayon" => Some(StdlibFeature::Rayon),
        "regex" => Some(StdlibFeature::Regex),
        "rust_decimal" => Some(StdlibFeature::RustDecimal),
        "serde_json" => Some(StdlibFeature::SerdeJson),
        "sha1" => Some(StdlibFeature::Sha1),
        "sha2" => Some(StdlibFeature::Sha2),
        "sifr_runtime" | "sifr-runtime" => Some(StdlibFeature::SifrRuntime),
        "tokio" => Some(StdlibFeature::Tokio),
        "toml" => Some(StdlibFeature::Toml),
        "unicode_names2" => Some(StdlibFeature::UnicodeNames),
        "unicode-normalization" | "unicode_normalization" => {
            Some(StdlibFeature::UnicodeNormalization)
        }
        "unicode-segmentation" | "unicode_segmentation" => Some(StdlibFeature::UnicodeSegmentation),
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
        "sifr.parallel" => &[StdlibFeature::Rayon],
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
    i18n: bool,
    unicode: bool,
}

impl RuntimeFeatures {
    fn from_requirements(
        stdlib_modules: &HashSet<String>,
        required_features: &HashSet<StdlibFeature>,
    ) -> Self {
        Self {
            i18n: needs_sifr_runtime_i18n(stdlib_modules, required_features),
            unicode: needs_sifr_runtime_unicode(stdlib_modules, required_features),
        }
    }
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
    dependency.spec.to_string()
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

#[cfg(test)]
mod tests {
    use super::{generated_cargo_dependencies, StdlibFeature};
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
    fn unicode_module_emits_runtime_and_unicode_dependencies() {
        let deps = generated_cargo_dependencies(
            &HashSet::from(["sifr.unicode".to_string()]),
            &HashSet::new(),
        );

        assert!(deps
            .iter()
            .any(|dep| dep.starts_with("sifr_runtime = ")
                && dep.contains("features = [\"unicode\"]")));
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
            .any(|dep| dep.starts_with("sifr_runtime = ")
                && dep.contains("features = [\"unicode\"]")));
    }

    #[test]
    fn i18n_module_emits_runtime_and_icu_dependencies() {
        let deps = generated_cargo_dependencies(
            &HashSet::from(["sifr.i18n".to_string()]),
            &HashSet::new(),
        );

        assert!(
            deps.iter()
                .any(|dep| dep.starts_with("sifr_runtime = ")
                    && dep.contains("features = [\"i18n\"]"))
        );
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
    fn text_i18n_feature_dependency_snapshots_cover_phase_combinations() {
        let cases = [
            (
                "encoding",
                HashSet::from(["sifr.encoding".to_string()]),
                HashSet::new(),
                vec![
                    "encoding_rs = \"0.8.35\"",
                    "sifr_runtime = { path = \"<sifr_runtime_path>\" }",
                ],
            ),
            (
                "unicode",
                HashSet::from(["sifr.unicode".to_string()]),
                HashSet::new(),
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
                HashSet::new(),
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
                HashSet::new(),
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
                HashSet::new(),
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
                HashSet::new(),
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
                HashSet::new(),
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

        for (name, modules, required_features, expected) in cases {
            let deps = generated_cargo_dependencies(&modules, &required_features)
                .iter()
                .map(|dependency| normalize_runtime_dependency(dependency))
                .collect::<Vec<_>>();

            assert_eq!(deps, expected, "{name}");
        }
    }
}
