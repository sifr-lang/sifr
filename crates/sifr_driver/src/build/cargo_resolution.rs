use super::cargo_invocation_trace::record_cargo_invocation;
use super::cargo_resolution_cache::{cache_prepared_lock, prepared_lock_is_valid};
use super::rust_interop_digest::{CacheIdentity, cache_identity, digest_file, push_cache_bytes};
use super::workspace::artifact_cache_root;
use crate::diagnostics::{RenderedDiagnostic, diagnostic_with_code};
use sifr_diagnostics::DiagnosticCode;
use sifr_package::{CargoLockMode, cargo::lock_modes::cargo_lock_failure_reason};
use sifr_stdlib_manifest::CargoVendorMode;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::AtomicU64;

const PREPARED_RESOLUTION_DIR: &str = "cargo_resolution";
pub(super) static PREPARED_LOCK_NONCE: AtomicU64 = AtomicU64::new(0);
type RegistryEntry = (String, String, String, String);
type RegistryCompatibilityFamily = (String, String, String);

#[derive(Clone, Debug)]
pub(crate) struct CargoResolutionPolicy {
    pub(super) lock_mode: CargoLockMode,
    pub(super) cargo_vendor_mode: CargoVendorMode,
    pub(super) authoritative_locks: Vec<PathBuf>,
    pub(super) trusted_vendor_dirs: Vec<PathBuf>,
}

impl CargoResolutionPolicy {
    pub(super) fn normal() -> Self {
        Self {
            lock_mode: CargoLockMode::Normal,
            cargo_vendor_mode: CargoVendorMode::SysrootOnly,
            authoritative_locks: Vec::new(),
            trusted_vendor_dirs: Vec::new(),
        }
    }

    pub(super) const fn uses_sysroot_vendor(&self) -> bool {
        matches!(self.cargo_vendor_mode, CargoVendorMode::SysrootOnly)
    }

    pub(crate) fn for_test_scope(
        test_scope: &Path,
        lock_mode: CargoLockMode,
        dependency_plan: &sifr_stdlib_manifest::SysrootDependencyPlan,
    ) -> Self {
        let mut authoritative_locks = Vec::new();
        if lock_mode != CargoLockMode::Normal {
            if let Some(lock) = nearest_ancestor_file(test_scope, "Cargo.lock") {
                authoritative_locks.push(lock);
            }
            let sysroot_lock = dependency_plan.sysroot_root.join("Cargo.lock");
            if !authoritative_locks.contains(&sysroot_lock) {
                authoritative_locks.push(sysroot_lock);
            }
        }
        Self {
            lock_mode,
            cargo_vendor_mode: dependency_plan.cargo_vendor_mode,
            authoritative_locks,
            trusted_vendor_dirs: vec![dependency_plan.vendor_dir.clone()],
        }
    }
}

fn nearest_ancestor_file(start: &Path, file_name: &str) -> Option<PathBuf> {
    start
        .ancestors()
        .map(|ancestor| ancestor.join(file_name))
        .find(|candidate| candidate.is_file())
}

pub(super) struct PreparedCargoResolution {
    lock_path: PathBuf,
    initial_digest: Option<String>,
    lock_mode: CargoLockMode,
}

pub(super) fn prepare_cargo_resolution(
    project_dir: &Path,
    policy: &CargoResolutionPolicy,
    cargo_prefix_args: &[String],
) -> Result<PreparedCargoResolution, Vec<RenderedDiagnostic>> {
    let lock_path = project_dir.join("Cargo.lock");
    if policy.lock_mode == CargoLockMode::Normal {
        return Ok(PreparedCargoResolution {
            initial_digest: digest_file(&lock_path),
            lock_path,
            lock_mode: policy.lock_mode,
        });
    }
    validate_constrained_resolution_policy(policy)?;
    prepare_constrained_cargo_resolution(project_dir, policy, cargo_prefix_args, lock_path)
}

fn prepare_constrained_cargo_resolution(
    project_dir: &Path,
    policy: &CargoResolutionPolicy,
    cargo_prefix_args: &[String],
    lock_path: PathBuf,
) -> Result<PreparedCargoResolution, Vec<RenderedDiagnostic>> {
    debug_assert_ne!(policy.lock_mode, CargoLockMode::Normal);
    debug_assert!(!policy.authoritative_locks.is_empty());

    if !lock_path.is_file() {
        let (prepared_lock, identity) = prepared_lock_path(project_dir, policy, cargo_prefix_args)?;
        if prepared_lock_is_valid(&prepared_lock, &identity)? {
            std::fs::copy(&prepared_lock, &lock_path).map_err(|error| {
                vec![cargo_resolution_error(format!(
                    "failed to restore prepared Cargo lockfile '{}': {error}",
                    prepared_lock.display()
                ))]
            })?;
        } else {
            prepare_lockfile_from_authority(project_dir, policy, cargo_prefix_args)?;
            validate_authoritative_registry_entries(
                &lock_path,
                &policy.authoritative_locks,
                &policy.trusted_vendor_dirs,
            )?;
            if let Some(cache_root) = prepared_lock.parent().and_then(Path::parent) {
                std::fs::create_dir_all(cache_root).map_err(|error| {
                    vec![cargo_resolution_error(format!(
                        "failed to create prepared Cargo resolution cache '{}': {error}",
                        cache_root.display()
                    ))]
                })?;
            }
            cache_prepared_lock(&lock_path, &prepared_lock, &identity)?;
        }
    }
    validate_authoritative_registry_entries(
        &lock_path,
        &policy.authoritative_locks,
        &policy.trusted_vendor_dirs,
    )?;
    let initial_digest = digest_file(&lock_path).ok_or_else(|| {
        vec![cargo_resolution_error(format!(
            "prepared Cargo lockfile '{}' is unreadable",
            lock_path.display()
        ))]
    })?;
    Ok(PreparedCargoResolution {
        lock_path,
        initial_digest: Some(initial_digest),
        lock_mode: policy.lock_mode,
    })
}

pub(crate) fn cargo_resolution_cache_key_fragment(
    policy: &CargoResolutionPolicy,
) -> Result<String, Vec<RenderedDiagnostic>> {
    let mut fragment = format!(
        "lock_mode={}\nvendor_mode={}\n",
        policy.lock_mode.as_str(),
        policy.cargo_vendor_mode.as_str()
    );
    if policy.lock_mode != CargoLockMode::Normal {
        validate_constrained_resolution_policy(policy)?;
        for (index, authoritative) in policy.authoritative_locks.iter().enumerate() {
            let digest = authoritative_lock_digest(authoritative)?;
            let identity = normalized_policy_path(authoritative);
            writeln!(
                fragment,
                "authority[{index}]={identity}\ndigest[{index}]={digest}"
            )
            .map_err(|error| {
                vec![cargo_resolution_error(format!(
                    "failed to format Cargo resolution cache identity: {error}"
                ))]
            })?;
        }
    }
    for (index, vendor_dir) in policy.trusted_vendor_dirs.iter().enumerate() {
        writeln!(
            fragment,
            "trusted_vendor[{index}]={}",
            normalized_policy_path(vendor_dir)
        )
        .map_err(|error| {
            vec![cargo_resolution_error(format!(
                "failed to format Cargo resolution cache identity: {error}"
            ))]
        })?;
    }
    Ok(fragment)
}

fn validate_constrained_resolution_policy(
    policy: &CargoResolutionPolicy,
) -> Result<(), Vec<RenderedDiagnostic>> {
    if policy.authoritative_locks.is_empty() {
        return Err(vec![cargo_resolution_error(
            "locked Rust interop Cargo resolution has no authoritative package or sysroot lockfile",
        )]);
    }
    for authoritative in &policy.authoritative_locks {
        authoritative_lock_digest(authoritative)?;
    }
    Ok(())
}

fn authoritative_lock_digest(path: &Path) -> Result<String, Vec<RenderedDiagnostic>> {
    if !path.is_file() {
        return Err(vec![cargo_resolution_error(format!(
            "authoritative Cargo lockfile '{}' is missing",
            path.display()
        ))]);
    }
    digest_file(path).ok_or_else(|| {
        vec![cargo_resolution_error(format!(
            "authoritative Cargo lockfile '{}' is unreadable",
            path.display()
        ))]
    })
}

pub(super) fn normalized_policy_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

impl PreparedCargoResolution {
    pub(super) fn assert_unchanged(&self) -> Result<(), Vec<RenderedDiagnostic>> {
        if self.lock_mode == CargoLockMode::Normal {
            return Ok(());
        }
        // Cargo's standalone `--offline` mode can update a source lockfile.
        // Generated interop projects are stricter: their prepared lock is a
        // validated cache artifact, so every constrained mode keeps it
        // byte-identical.
        let current = digest_file(&self.lock_path);
        if current == self.initial_digest {
            return Ok(());
        }
        Err(vec![cargo_resolution_error(format!(
            "Cargo {} mode changed prepared lockfile '{}'",
            self.lock_mode.as_str(),
            self.lock_path.display()
        ))])
    }
}

pub(super) fn cargo_lock_mode_diagnostic(
    context: &str,
    stderr: &str,
) -> Option<RenderedDiagnostic> {
    cargo_lock_failure_reason(stderr).map(|reason| {
        cargo_resolution_error(format!(
            "{context} violated locked Cargo resolution ({reason}): {}",
            bounded_excerpt(stderr)
        ))
    })
}

fn prepare_lockfile_from_authority(
    project_dir: &Path,
    policy: &CargoResolutionPolicy,
    cargo_prefix_args: &[String],
) -> Result<(), Vec<RenderedDiagnostic>> {
    seed_lockfile_from_authorities(&project_dir.join("Cargo.lock"), &policy.authoritative_locks)?;
    let mut command = Command::new("cargo");
    command
        .args(cargo_prefix_args)
        .args(["metadata", "--format-version=1"])
        .current_dir(project_dir);
    if policy.lock_mode.is_network_disallowed() {
        command.arg("--offline");
    }
    record_cargo_invocation("resolution", policy.lock_mode, &command);
    let output = command.output().map_err(|error| {
        vec![cargo_resolution_error(format!(
            "failed to prepare generated Cargo resolution: {error}"
        ))]
    })?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(vec![cargo_resolution_error(format!(
        "failed to prepare Cargo lockfile in {} mode: {}",
        policy.lock_mode.as_str(),
        bounded_excerpt(&stderr)
    ))])
}

fn seed_lockfile_from_authorities(
    destination: &Path,
    authoritative_locks: &[PathBuf],
) -> Result<(), Vec<RenderedDiagnostic>> {
    let Some(lowest_priority) = authoritative_locks.last() else {
        return Err(vec![cargo_resolution_error(
            "Cargo resolution preparation has no authoritative lockfile seed",
        )]);
    };
    let mut merged = read_lock_table(lowest_priority)?;
    for higher_priority in authoritative_locks[..authoritative_locks.len() - 1]
        .iter()
        .rev()
    {
        overlay_registry_packages(&mut merged, &read_lock_table(higher_priority)?);
    }
    let rendered = toml::to_string(&merged).map_err(|error| {
        vec![cargo_resolution_error(format!(
            "failed to serialize merged Cargo authority lock: {error}"
        ))]
    })?;
    std::fs::write(destination, rendered).map_err(|error| {
        vec![cargo_resolution_error(format!(
            "failed to seed generated Cargo resolution at '{}': {error}",
            destination.display()
        ))]
    })
}

fn read_lock_table(path: &Path) -> Result<toml::Table, Vec<RenderedDiagnostic>> {
    std::fs::read_to_string(path)
        .map_err(|error| {
            vec![cargo_resolution_error(format!(
                "failed to read Cargo lock authority '{}': {error}",
                path.display()
            ))]
        })?
        .parse::<toml::Table>()
        .map_err(|error| {
            vec![cargo_resolution_error(format!(
                "failed to parse Cargo lock authority '{}': {error}",
                path.display()
            ))]
        })
}

fn overlay_registry_packages(base: &mut toml::Table, overlay: &toml::Table) {
    let overlay_registry = overlay
        .get("package")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter(|package| registry_package_compatibility_family(package).is_some())
        .cloned()
        .collect::<Vec<_>>();
    let overlay_families = overlay_registry
        .iter()
        .filter_map(registry_package_compatibility_family)
        .collect::<BTreeSet<_>>();
    let Some(base_packages) = base
        .entry("package")
        .or_insert_with(|| toml::Value::Array(Vec::new()))
        .as_array_mut()
    else {
        return;
    };
    base_packages.retain(|package| {
        registry_package_compatibility_family(package)
            .is_none_or(|family| !overlay_families.contains(&family))
    });
    base_packages.extend(overlay_registry);
}

fn registry_package_compatibility_family(
    package: &toml::Value,
) -> Option<RegistryCompatibilityFamily> {
    let package = package.as_table()?;
    let source = package.get("source")?.as_str()?;
    if !source.starts_with("registry+") {
        return None;
    }
    let version = package.get("version")?.as_str()?;
    Some((
        package.get("name")?.as_str()?.to_string(),
        source.to_string(),
        registry_version_compatibility_family(version),
    ))
}

fn registry_version_compatibility_family(version: &str) -> String {
    if version.contains('-') {
        return format!("exact:{version}");
    }
    let core = version.split('+').next().unwrap_or(version);
    let components = core
        .split('.')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>();
    let Ok(components) = components else {
        return format!("exact:{version}");
    };
    let [major, minor, patch] = components.as_slice() else {
        return format!("exact:{version}");
    };
    if *major != 0 {
        format!("major:{major}")
    } else if *minor != 0 {
        format!("minor:{major}.{minor}")
    } else {
        format!("patch:{major}.{minor}.{patch}")
    }
}

fn prepared_lock_path(
    project_dir: &Path,
    policy: &CargoResolutionPolicy,
    cargo_prefix_args: &[String],
) -> Result<(PathBuf, CacheIdentity), Vec<RenderedDiagnostic>> {
    let mut input = Vec::new();
    push_cache_bytes(&mut input, "sifr-cargo-resolution-v7");
    push_cache_bytes(&mut input, policy.lock_mode.as_str());
    push_cache_bytes(&mut input, policy.cargo_vendor_mode.as_str());
    push_cache_bytes(&mut input, &normalized_manifest_cache_input(project_dir)?);
    for argument in cargo_prefix_args {
        push_cache_bytes(&mut input, argument);
    }
    for lock in &policy.authoritative_locks {
        push_cache_bytes(&mut input, &authoritative_lock_digest(lock)?);
    }
    for vendor_dir in &policy.trusted_vendor_dirs {
        push_cache_bytes(&mut input, &normalized_policy_path(vendor_dir));
    }
    let identity = cache_identity("cargo-resolution", input);
    let path = artifact_cache_root()
        .join(PREPARED_RESOLUTION_DIR)
        .join(&identity.digest)
        .join("Cargo.lock");
    Ok((path, identity))
}

fn normalized_manifest_cache_input(project_dir: &Path) -> Result<String, Vec<RenderedDiagnostic>> {
    let manifest_path = project_dir.join("Cargo.toml");
    let source = std::fs::read_to_string(&manifest_path).map_err(|error| {
        vec![cargo_resolution_error(format!(
            "failed to read generated Cargo manifest: {error}"
        ))]
    })?;
    let mut manifest = source
        .parse::<toml::Table>()
        .map(toml::Value::Table)
        .map_err(|error| {
            vec![cargo_resolution_error(format!(
                "failed to parse generated Cargo manifest: {error}"
            ))]
        })?;
    normalize_path_dependency_identities(&mut manifest, project_dir);
    toml::to_string(&manifest).map_err(|error| {
        vec![cargo_resolution_error(format!(
            "failed to normalize generated Cargo manifest: {error}"
        ))]
    })
}

fn normalize_path_dependency_identities(value: &mut toml::Value, project_dir: &Path) {
    match value {
        toml::Value::Table(table) => {
            for (key, nested) in table {
                if key == "path" {
                    if let Some(path) = nested.as_str() {
                        let dependency_root = if Path::new(path).is_absolute() {
                            PathBuf::from(path)
                        } else {
                            project_dir.join(path)
                        };
                        let dependency_manifest = dependency_root.join("Cargo.toml");
                        if let Some(digest) = digest_file(&dependency_manifest) {
                            *nested = toml::Value::String(format!(
                                "sifr-path-dependency-manifest:{digest}"
                            ));
                            continue;
                        }
                    }
                }
                normalize_path_dependency_identities(nested, project_dir);
            }
        }
        toml::Value::Array(values) => {
            for nested in values {
                normalize_path_dependency_identities(nested, project_dir);
            }
        }
        _ => {}
    }
}

fn validate_authoritative_registry_entries(
    prepared_lock: &Path,
    authoritative_locks: &[PathBuf],
    trusted_vendor_dirs: &[PathBuf],
) -> Result<(), Vec<RenderedDiagnostic>> {
    let prepared = registry_entries(prepared_lock)?;
    let mut authoritative = BTreeSet::new();
    for lock in authoritative_locks {
        authoritative.extend(registry_entries(lock)?);
    }
    let unknown = prepared.iter().find(|entry| {
        !authoritative.contains(*entry)
            && !trusted_vendor_dirs
                .iter()
                .any(|vendor_dir| vendor_contains(vendor_dir, entry))
    });
    let Some((name, version, source, checksum)) = unknown else {
        return Ok(());
    };
    Err(vec![cargo_resolution_error(format!(
        "prepared Cargo resolution contains non-authoritative registry package \
         `{name} {version}` from `{source}` with checksum `{checksum}`"
    ))])
}

fn vendor_contains(vendor_dir: &Path, entry: &RegistryEntry) -> bool {
    let (name, version, source, checksum) = entry;
    if !source.starts_with("registry+") {
        return false;
    }
    [
        vendor_dir.join(format!("{name}-{version}")),
        vendor_dir.join(name),
    ]
    .into_iter()
    .any(|crate_dir| {
        let manifest = std::fs::read_to_string(crate_dir.join("Cargo.toml"))
            .ok()
            .and_then(|source| source.parse::<toml::Table>().ok());
        let identity_matches = manifest.as_ref().is_some_and(|manifest| {
            manifest
                .get("package")
                .and_then(toml::Value::as_table)
                .is_some_and(|package| {
                    package.get("name").and_then(toml::Value::as_str) == Some(name)
                        && package.get("version").and_then(toml::Value::as_str) == Some(version)
                })
        });
        identity_matches
            && std::fs::read_to_string(crate_dir.join(".cargo-checksum.json"))
                .ok()
                .and_then(|contents| serde_json::from_str::<serde_json::Value>(&contents).ok())
                .and_then(|value| {
                    value
                        .get("package")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string)
                })
                .is_some_and(|package_checksum| package_checksum == *checksum)
    })
}

fn registry_entries(lock_path: &Path) -> Result<BTreeSet<RegistryEntry>, Vec<RenderedDiagnostic>> {
    let source = std::fs::read_to_string(lock_path).map_err(|error| {
        vec![cargo_resolution_error(format!(
            "failed to read Cargo lockfile '{}': {error}",
            lock_path.display()
        ))]
    })?;
    let value = source.parse::<toml::Table>().map_err(|error| {
        vec![cargo_resolution_error(format!(
            "failed to parse Cargo lockfile '{}': {error}",
            lock_path.display()
        ))]
    })?;
    let Some(packages) = value.get("package").and_then(toml::Value::as_array) else {
        return Ok(BTreeSet::new());
    };
    Ok(packages
        .iter()
        .filter_map(|package| {
            let package = package.as_table()?;
            let source = package.get("source")?.as_str()?;
            Some((
                package.get("name")?.as_str()?.to_string(),
                package.get("version")?.as_str()?.to_string(),
                source.to_string(),
                package
                    .get("checksum")
                    .and_then(toml::Value::as_str)
                    .unwrap_or("<missing>")
                    .to_string(),
            ))
        })
        .collect())
}

fn bounded_excerpt(stderr: &str) -> String {
    stderr
        .split_whitespace()
        .take(80)
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn cargo_resolution_error(message: impl Into<String>) -> RenderedDiagnostic {
    diagnostic_with_code(message.into(), DiagnosticCode::RUST_CARGO_METADATA)
}

#[cfg(test)]
mod tests {
    use super::{
        CargoResolutionPolicy, CargoVendorMode, PREPARED_LOCK_NONCE,
        cargo_resolution_cache_key_fragment, normalized_manifest_cache_input, registry_entries,
        registry_version_compatibility_family, seed_lockfile_from_authorities,
        validate_authoritative_registry_entries,
    };
    use crate::build::cargo_resolution_cache::{cache_prepared_lock, prepared_lock_is_valid};
    use crate::build::rust_interop_digest::cache_identity;
    use sifr_package::CargoLockMode;
    use std::collections::BTreeSet;
    use std::path::PathBuf;
    use std::sync::atomic::Ordering;

    #[test]
    fn probe_vendor_replacement_follows_resolution_ownership() {
        assert!(CargoResolutionPolicy::normal().uses_sysroot_vendor());
        let package_owned = CargoResolutionPolicy {
            lock_mode: CargoLockMode::Frozen,
            cargo_vendor_mode: CargoVendorMode::PackageOwned,
            authoritative_locks: Vec::new(),
            trusted_vendor_dirs: Vec::new(),
        };
        assert!(!package_owned.uses_sysroot_vendor());
    }

    #[test]
    fn constrained_cache_identity_rejects_missing_authority_before_cache_lookup() {
        let project = test_project("missing_authority_identity");
        let policy = CargoResolutionPolicy {
            lock_mode: CargoLockMode::Frozen,
            cargo_vendor_mode: CargoVendorMode::SysrootOnly,
            authoritative_locks: vec![project.0.join("missing.lock")],
            trusted_vendor_dirs: Vec::new(),
        };

        let diagnostics = cargo_resolution_cache_key_fragment(&policy)
            .expect_err("missing lock authority must reject a cache identity");

        assert!(diagnostics[0].message.contains("is missing"));
    }

    #[test]
    fn cache_identity_covers_lock_and_vendor_policy() {
        let project = test_project("cache_policy_identity");
        let authority = project.0.join("authority.lock");
        write_registry_lock(&authority, "1.2.3", "first-checksum");
        let base = CargoResolutionPolicy {
            lock_mode: CargoLockMode::Locked,
            cargo_vendor_mode: CargoVendorMode::SysrootOnly,
            authoritative_locks: vec![authority.clone()],
            trusted_vendor_dirs: vec![project.0.join("vendor-a")],
        };
        let first = cargo_resolution_cache_key_fragment(&base)
            .expect("complete constrained policy should have an identity");

        let mut changed_mode = base.clone();
        changed_mode.lock_mode = CargoLockMode::Frozen;
        assert_ne!(
            first,
            cargo_resolution_cache_key_fragment(&changed_mode)
                .expect("changed lock mode should have an identity")
        );

        let mut changed_vendor_mode = base.clone();
        changed_vendor_mode.cargo_vendor_mode = CargoVendorMode::PackageOwned;
        assert_ne!(
            first,
            cargo_resolution_cache_key_fragment(&changed_vendor_mode)
                .expect("changed vendor mode should have an identity")
        );

        let mut changed_vendor_root = base.clone();
        changed_vendor_root.trusted_vendor_dirs = vec![project.0.join("vendor-b")];
        assert_ne!(
            first,
            cargo_resolution_cache_key_fragment(&changed_vendor_root)
                .expect("changed vendor root should have an identity")
        );

        write_registry_lock(&authority, "1.2.3", "second-checksum");
        assert_ne!(
            first,
            cargo_resolution_cache_key_fragment(&base)
                .expect("changed authority content should have an identity")
        );
    }

    #[test]
    fn prepared_resolution_cache_identity_ignores_ephemeral_path_roots() {
        let first = test_project("first");
        let second = test_project("second");

        assert_eq!(
            normalized_manifest_cache_input(&first.0)
                .expect("first generated manifest should normalize"),
            normalized_manifest_cache_input(&second.0)
                .expect("second generated manifest should normalize")
        );
    }

    #[test]
    fn prepared_lock_hit_verifies_complete_cache_identity() {
        let project = test_project("prepared_full_identity");
        let source_lock = project.0.join("source.lock");
        let prepared_lock = project.0.join("cache").join("digest").join("Cargo.lock");
        std::fs::create_dir_all(project.0.join("cache")).expect("cache root should be created");
        write_registry_lock(&source_lock, "1.2.3", "checksum");
        let expected = cache_identity("cargo-resolution", b"expected".to_vec());
        let different = cache_identity("cargo-resolution", b"different".to_vec());

        cache_prepared_lock(&source_lock, &prepared_lock, &expected)
            .expect("prepared lock should be cached");

        assert!(
            prepared_lock_is_valid(&prepared_lock, &expected)
                .expect("matching identity should be valid")
        );
        assert!(prepared_lock_is_valid(&prepared_lock, &different).is_err());
    }

    #[test]
    fn registry_compatibility_families_match_cargo_semver_boundaries() {
        assert_eq!(
            registry_version_compatibility_family("1.2.3"),
            registry_version_compatibility_family("1.9.0")
        );
        assert_eq!(
            registry_version_compatibility_family("0.4.2"),
            registry_version_compatibility_family("0.4.8")
        );
        assert_ne!(
            registry_version_compatibility_family("0.3.4"),
            registry_version_compatibility_family("0.4.2")
        );
        assert_ne!(
            registry_version_compatibility_family("0.0.1"),
            registry_version_compatibility_family("0.0.2")
        );
        assert_ne!(
            registry_version_compatibility_family("1.0.0-alpha.1"),
            registry_version_compatibility_family("1.0.0-alpha.2")
        );
    }

    #[test]
    fn distinct_registry_versions_from_each_authority_are_seeded() {
        let project = test_project("authority_priority");
        let package_lock = project.0.join("package.lock");
        let sysroot_lock = project.0.join("sysroot.lock");
        let destination = project.0.join("Cargo.lock");
        write_registry_lock(&package_lock, "0.3.4", "package-checksum");
        write_registry_lock(&sysroot_lock, "0.4.2", "sysroot-checksum");

        seed_lockfile_from_authorities(&destination, &[package_lock.clone(), sysroot_lock.clone()])
            .expect("authority locks should merge");

        let entries = registry_entries(&destination).expect("merged lock should parse");
        assert!(entries.contains(&(
            "shared-package".to_string(),
            "0.3.4".to_string(),
            "registry+https://github.com/rust-lang/crates.io-index".to_string(),
            "package-checksum".to_string(),
        )));
        assert!(entries.contains(&(
            "shared-package".to_string(),
            "0.4.2".to_string(),
            "registry+https://github.com/rust-lang/crates.io-index".to_string(),
            "sysroot-checksum".to_string(),
        )));
    }

    #[test]
    fn higher_priority_lock_replaces_a_compatible_registry_version() {
        let project = test_project("exact_authority_priority");
        let package_lock = project.0.join("package.lock");
        let sysroot_lock = project.0.join("sysroot.lock");
        let destination = project.0.join("Cargo.lock");
        write_registry_lock(&package_lock, "0.4.8", "package-checksum");
        write_registry_lock(&sysroot_lock, "0.4.6", "sysroot-checksum");

        seed_lockfile_from_authorities(&destination, &[package_lock, sysroot_lock])
            .expect("authority locks should merge");

        let entries = registry_entries(&destination).expect("merged lock should parse");
        assert_eq!(
            entries,
            BTreeSet::from([(
                "shared-package".to_string(),
                "0.4.8".to_string(),
                "registry+https://github.com/rust-lang/crates.io-index".to_string(),
                "package-checksum".to_string(),
            )])
        );
    }

    #[test]
    fn exact_registry_entry_from_each_authority_is_accepted() {
        let project = test_project("authority_union");
        let package_lock = project.0.join("package.lock");
        let sysroot_lock = project.0.join("sysroot.lock");
        let prepared_lock = project.0.join("prepared.lock");
        write_registry_lock(&package_lock, "1.0.0", "package-checksum");
        write_registry_lock(&sysroot_lock, "2.0.0", "sysroot-checksum");
        write_registry_lock(&prepared_lock, "2.0.0", "sysroot-checksum");

        validate_authoritative_registry_entries(&prepared_lock, &[package_lock, sysroot_lock], &[])
            .expect("the exact sysroot entry must remain authoritative");
    }

    #[test]
    fn registry_entry_missing_from_every_authority_is_rejected() {
        let project = test_project("unknown_authority");
        let package_lock = project.0.join("package.lock");
        let sysroot_lock = project.0.join("sysroot.lock");
        let prepared_lock = project.0.join("prepared.lock");
        write_registry_lock(&package_lock, "1.0.0", "package-checksum");
        write_registry_lock(&sysroot_lock, "2.0.0", "sysroot-checksum");
        write_registry_lock(&prepared_lock, "3.0.0", "unknown-checksum");

        let error = validate_authoritative_registry_entries(
            &prepared_lock,
            &[package_lock, sysroot_lock],
            &[],
        )
        .expect_err("an unknown exact registry entry must fail closed");
        assert!(error[0].message.contains("shared-package 3.0.0"));
    }

    fn test_project(label: &str) -> TestProject {
        let nonce = PREPARED_LOCK_NONCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "sifr_cargo_resolution_identity_{}_{}_{}",
            std::process::id(),
            nonce,
            label
        ));
        let dependency = root.join("dependency");
        std::fs::create_dir_all(&dependency).expect("path dependency should be created");
        std::fs::write(
            dependency.join("Cargo.toml"),
            "[package]\nname = \"same-dependency\"\nversion = \"0.1.0\"\n",
        )
        .expect("path dependency manifest should be written");
        std::fs::write(
            root.join("Cargo.toml"),
            format!(
                "[package]\nname = \"generated\"\nversion = \"0.1.0\"\n\
                 [dependencies]\nsame-dependency = {{ path = {dependency:?} }}\n"
            ),
        )
        .expect("generated manifest should be written");
        TestProject(root)
    }

    fn write_registry_lock(path: &std::path::Path, version: &str, checksum: &str) {
        std::fs::write(
            path,
            format!(
                "version = 4\n\n[[package]]\nname = \"shared-package\"\n\
                 version = \"{version}\"\n\
                 source = \"registry+https://github.com/rust-lang/crates.io-index\"\n\
                 checksum = \"{checksum}\"\n"
            ),
        )
        .expect("authority lock should be written");
    }

    struct TestProject(PathBuf);

    impl Drop for TestProject {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}
