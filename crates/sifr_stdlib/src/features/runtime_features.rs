use super::StdlibFeature;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RuntimeFeatures {
    pub(super) http: bool,
    pub(super) i18n: bool,
    pub(super) net: bool,
    pub(super) python: bool,
    pub(super) tls: bool,
    pub(super) unicode: bool,
}

impl RuntimeFeatures {
    pub(super) fn from_requirements(
        stdlib_modules: &HashSet<String>,
        required_features: &HashSet<StdlibFeature>,
    ) -> Self {
        Self {
            http: needs_sifr_runtime_http(stdlib_modules, required_features),
            i18n: needs_sifr_runtime_i18n(stdlib_modules, required_features),
            net: needs_sifr_runtime_net(stdlib_modules),
            python: needs_sifr_runtime_python(stdlib_modules, required_features),
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

fn needs_sifr_runtime_python(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> bool {
    stdlib_modules
        .iter()
        .any(|module| matches!(module.as_str(), "sifr.python" | "_sifr.python"))
        || required_features.contains(&StdlibFeature::PythonRuntime)
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
