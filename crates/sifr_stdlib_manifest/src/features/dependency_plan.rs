use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use sifr_sysroot::{resolve_sysroot, ResolvedSysroot, SysrootError};

use super::generated_stdlib_features::planned_sifr_stdlib_features;
use super::runtime_features::RuntimeFeatures;
use super::{
    features_for_stdlib_module, GeneratedCargoDependency, StdlibFeature, STDLIB_FEATURE_SPECS,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SysrootCrate {
    SifrRuntime,
    SifrStdlib,
}

impl SysrootCrate {
    #[must_use]
    pub const fn package_name(self) -> &'static str {
        match self {
            Self::SifrRuntime => "sifr_runtime",
            Self::SifrStdlib => "sifr_stdlib",
        }
    }

    #[must_use]
    pub const fn fingerprint_key(self) -> &'static str {
        match self {
            Self::SifrRuntime => "sifr_runtime",
            Self::SifrStdlib => "sifr_stdlib",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CargoVendorMode {
    SysrootOnly,
    PackageOwned,
}

impl CargoVendorMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SysrootOnly => "sysroot-only",
            Self::PackageOwned => "package-owned",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SysrootCrateDependency {
    pub krate: SysrootCrate,
    pub path: PathBuf,
    pub features: BTreeSet<String>,
}

impl SysrootCrateDependency {
    #[must_use]
    pub fn cargo_line(&self) -> String {
        let package = self.krate.package_name();
        let path = toml_quote_path(&self.path);
        if self.features.is_empty() {
            return format!("{package} = {{ path = {path}, default-features = false }}");
        }
        let features = self
            .features
            .iter()
            .map(|feature| toml_quote_string(feature))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{package} = {{ path = {path}, default-features = false, features = [{features}] }}"
        )
    }

    fn fingerprint_lines(&self) -> String {
        format!(
            "{}\npath={}\nfeatures={}\n",
            self.krate.fingerprint_key(),
            self.path.display(),
            self.features.iter().cloned().collect::<Vec<_>>().join(",")
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SysrootDependencyPlan {
    pub sysroot_root: PathBuf,
    pub toolchain_id: String,
    pub sysroot_content_sha256: String,
    pub cargo_config: PathBuf,
    pub vendor_dir: PathBuf,
    pub crates: Vec<SysrootCrateDependency>,
    pub retained_direct_dependencies: Vec<String>,
    pub cargo_vendor_mode: CargoVendorMode,
    pub cache_fingerprint: String,
}

impl SysrootDependencyPlan {
    #[must_use]
    pub fn cargo_dependency_lines(&self) -> Vec<String> {
        self.crates
            .iter()
            .map(SysrootCrateDependency::cargo_line)
            .chain(self.retained_direct_dependencies.iter().cloned())
            .collect()
    }
}

pub fn try_sysroot_dependency_plan(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
    cargo_vendor_mode: CargoVendorMode,
) -> Result<SysrootDependencyPlan, SysrootError> {
    let sysroot = resolve_sysroot(None)?;
    Ok(sysroot_dependency_plan_with_sysroot(
        stdlib_modules,
        required_features,
        &sysroot,
        cargo_vendor_mode,
    ))
}

#[must_use]
pub fn sysroot_dependency_plan_with_sysroot(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
    sysroot: &ResolvedSysroot,
    cargo_vendor_mode: CargoVendorMode,
) -> SysrootDependencyPlan {
    let runtime_features = RuntimeFeatures::from_requirements(stdlib_modules, required_features);
    let stdlib_features = planned_sifr_stdlib_features(stdlib_modules, required_features)
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let direct_dependencies = retained_direct_dependencies(stdlib_modules, required_features);
    let runtime_required = runtime_features.requires_runtime_crate()
        || required_features.contains(&StdlibFeature::SifrRuntime);
    let crates = sysroot_crate_dependencies(
        sysroot,
        runtime_required,
        runtime_features,
        &stdlib_features,
    );
    let cache_fingerprint =
        cache_fingerprint(sysroot, &crates, &direct_dependencies, cargo_vendor_mode);

    SysrootDependencyPlan {
        sysroot_root: sysroot.root.clone(),
        toolchain_id: sysroot.toolchain_id(),
        sysroot_content_sha256: sysroot.manifest.sysroot_content_sha256.clone(),
        cargo_config: sysroot.paths.cargo_config.clone(),
        vendor_dir: sysroot.paths.vendor.clone(),
        crates,
        retained_direct_dependencies: direct_dependencies,
        cargo_vendor_mode,
        cache_fingerprint,
    }
}

pub fn try_generated_cargo_dependencies(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> Result<Vec<String>, SysrootError> {
    try_sysroot_dependency_plan(
        stdlib_modules,
        required_features,
        CargoVendorMode::SysrootOnly,
    )
    .map(|plan| plan.cargo_dependency_lines())
}

fn sysroot_crate_dependencies(
    sysroot: &ResolvedSysroot,
    runtime_required: bool,
    runtime_features: RuntimeFeatures,
    stdlib_features: &BTreeSet<String>,
) -> Vec<SysrootCrateDependency> {
    let mut crates = Vec::new();
    if runtime_required {
        crates.push(SysrootCrateDependency {
            krate: SysrootCrate::SifrRuntime,
            path: sysroot.paths.runtime_crate.clone(),
            features: runtime_features.feature_names(),
        });
    }
    if !stdlib_features.is_empty() {
        crates.push(SysrootCrateDependency {
            krate: SysrootCrate::SifrStdlib,
            path: sysroot.paths.stdlib_crate.clone(),
            features: stdlib_features.clone(),
        });
    }
    crates
}

fn retained_direct_dependencies(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> Vec<String> {
    let runtime_features = RuntimeFeatures::from_requirements(stdlib_modules, required_features);
    let mut deps = Vec::new();
    let mut packages = BTreeSet::new();

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
            if dependency.package != "sifr_runtime" && packages.insert(dependency.package) {
                deps.push(render_retained_dependency_spec(
                    dependency,
                    runtime_features,
                ));
            }
        }
    }
}

fn render_retained_dependency_spec(
    dependency: &GeneratedCargoDependency,
    runtime_features: RuntimeFeatures,
) -> String {
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

fn cache_fingerprint(
    sysroot: &ResolvedSysroot,
    crates: &[SysrootCrateDependency],
    direct_dependencies: &[String],
    cargo_vendor_mode: CargoVendorMode,
) -> String {
    let mut fingerprint = String::new();
    fingerprint.push_str("[sysroot]\n");
    fingerprint.push_str("toolchain_id=");
    fingerprint.push_str(&sysroot.toolchain_id());
    fingerprint.push('\n');
    fingerprint.push_str("content_sha256=");
    fingerprint.push_str(&sysroot.manifest.sysroot_content_sha256);
    fingerprint.push('\n');
    fingerprint.push_str("cargo_lock_sha256=");
    fingerprint.push_str(&sysroot.manifest.cargo_lock_sha256);
    fingerprint.push('\n');
    fingerprint.push_str("vendor_mode=");
    fingerprint.push_str(cargo_vendor_mode.as_str());
    fingerprint.push('\n');
    fingerprint.push_str("[crates]\n");
    for krate in crates {
        fingerprint.push_str(&krate.fingerprint_lines());
    }
    fingerprint.push_str("[retained-direct-dependencies]\n");
    for dependency in direct_dependencies {
        fingerprint.push_str(dependency);
        fingerprint.push('\n');
    }
    fingerprint
}

fn toml_quote_path(path: &Path) -> String {
    toml_quote_string(&path.display().to_string())
}

fn toml_quote_string(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            '\u{08}' => quoted.push_str("\\b"),
            '\u{0C}' => quoted.push_str("\\f"),
            ch if ch.is_control() => {
                push_unicode_escape(&mut quoted, u32::from(ch));
            }
            ch => quoted.push(ch),
        }
    }
    quoted.push('"');
    quoted
}

fn push_unicode_escape(output: &mut String, value: u32) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    output.push_str("\\u");
    for shift in [12, 8, 4, 0] {
        let index = ((value >> shift) & 0xF) as usize;
        output.push(char::from(HEX[index]));
    }
}

#[cfg(test)]
mod tests {
    use super::{SysrootCrate, SysrootCrateDependency};
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    #[test]
    fn sysroot_crate_dependency_line_quotes_toml_strings() {
        let dependency = SysrootCrateDependency {
            krate: SysrootCrate::SifrStdlib,
            path: PathBuf::from("/opt/sifr\nsysroot/crates/sifr_stdlib"),
            features: BTreeSet::from(["json".to_string()]),
        };

        assert_eq!(
            dependency.cargo_line(),
            "sifr_stdlib = { path = \"/opt/sifr\\nsysroot/crates/sifr_stdlib\", default-features = false, features = [\"json\"] }"
        );
    }
}
