use super::cargo_invocation_trace::record_cargo_invocation;
use super::rust_interop_digest::digest_file_checked;

use crate::diagnostics::{RenderedDiagnostic, diagnostic_with_code};
use sifr_diagnostics::DiagnosticCode;
use sifr_package::{CargoLockMode, cargo::lock_modes::cargo_lock_failure_reason};
use sifr_stdlib_manifest::CargoVendorMode;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

#[path = "cargo_resolution_identity.rs"]
mod identity;
use identity::{checked_authorities, prepared_lock_path};
static PREPARED_LOCK_NONCE: AtomicU64 = AtomicU64::new(0);
type RegistryEntry = (String, String, String, String);
type RegistryCompatibilityFamily = (String, String, String);

#[derive(Clone, Debug)]
pub(crate) struct CargoResolutionPolicy {
    pub(crate) application_profile: crate::ApplicationProfile,
    pub(crate) native_toolchain: Result<sifr_sysroot::NativeToolchain, String>,
    pub(crate) lock_mode: CargoLockMode,
    pub(crate) cargo_vendor_mode: CargoVendorMode,
    pub(crate) authoritative_locks: Vec<PathBuf>,
    pub(crate) trusted_vendor_dirs: Vec<PathBuf>,
}

impl CargoResolutionPolicy {
    pub(crate) fn resolve_native_toolchain() -> Result<sifr_sysroot::NativeToolchain, String> {
        let cwd =
            std::env::current_dir().map_err(|_| "cannot resolve native invocation directory")?;
        sifr_sysroot::NativeToolchain::resolve_at(&cwd)
    }
    pub(crate) fn cargo_command(&self) -> Result<Command, Vec<RenderedDiagnostic>> {
        self.native_toolchain
            .as_ref()
            .map_err(Clone::clone)
            .and_then(sifr_sysroot::NativeToolchain::cargo_command)
            .map_err(|error| vec![cargo_resolution_error(error.clone())])
    }

    pub(crate) fn normal() -> Self {
        Self {
            application_profile: crate::ApplicationProfile::Release,
            native_toolchain: Self::resolve_native_toolchain(),
            lock_mode: CargoLockMode::Normal,
            cargo_vendor_mode: CargoVendorMode::SysrootOnly,
            authoritative_locks: Vec::new(),
            trusted_vendor_dirs: Vec::new(),
        }
    }

    pub(super) const fn uses_sysroot_vendor(&self) -> bool {
        matches!(self.cargo_vendor_mode, CargoVendorMode::SysrootOnly)
    }
}

pub(crate) struct PreparedCargoResolution {
    lock_path: PathBuf,
    initial_digest: Option<String>,
    lock_mode: CargoLockMode,
    authority_check: Option<PreparedAuthorityCheck>,
    _cache_lease: Option<std::fs::File>,
}

struct PreparedAuthorityCheck {
    project_dir: PathBuf,
    policy: CargoResolutionPolicy,
    cargo_prefix_args: Vec<String>,
    prepared_lock: PathBuf,
}

pub(crate) fn prepare_cargo_resolution(
    project_dir: &Path,
    policy: &CargoResolutionPolicy,
    cargo_prefix_args: &[String],
) -> Result<PreparedCargoResolution, Vec<RenderedDiagnostic>> {
    let lock_path = project_dir.join("Cargo.lock");
    checked_authorities(&policy.authoritative_locks)?;
    if policy.lock_mode == CargoLockMode::Normal {
        // A generated workspace must start from the package's resolved pins,
        // just as Cargo does in the original workspace. Normal mode may still
        // update the generated lock; it must not discard it and resolve anew
        // merely because the compiler placed the probe in a temporary root.
        let seed = policy.normal_seed_cache_fragment();
        let seed_path = project_dir.join(".sifr-cargo-seed");
        let previous_seed = match std::fs::read_to_string(&seed_path) {
            Ok(previous) => Some(previous),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(vec![cargo_resolution_error(error.to_string())]),
        };
        let authority_changed = previous_seed
            .as_ref()
            .zip(seed.as_ref())
            .is_some_and(|(previous, current)| previous != current);
        // A stable generated root retains its normal Cargo updates while the
        // source authority is unchanged; a new authority must reseed its pins.
        if authority_changed || (!lock_path.is_file() && !policy.authoritative_locks.is_empty()) {
            seed_lockfile_for_resolution(
                &lock_path,
                &policy.authoritative_locks,
                cargo_prefix_args,
            )?;
        }
        if let Some(seed) = seed {
            super::native_storage::write_changed(&seed_path, seed.as_bytes())
                .map_err(|error| vec![cargo_resolution_error(error.to_string())])?;
        }
        return Ok(PreparedCargoResolution {
            initial_digest: digest_file_checked(&lock_path).map_err(|error| {
                vec![cargo_resolution_error(format!(
                    "unreadable generated Cargo lockfile: {error}"
                ))]
            })?,
            lock_path,
            lock_mode: policy.lock_mode,
            authority_check: None,
            _cache_lease: None,
        });
    }
    if policy.authoritative_locks.is_empty() {
        return Err(vec![cargo_resolution_error(
            "locked Rust interop Cargo resolution has no authoritative package or sysroot lockfile",
        )]);
    }

    let prepared_lock = prepared_lock_path(project_dir, policy, cargo_prefix_args)?;
    let prepared_root = prepared_lock
        .parent()
        .ok_or_else(|| vec![cargo_resolution_error("missing prepared lock parent")])?;
    let cache_root = prepared_root
        .parent()
        .ok_or_else(|| vec![cargo_resolution_error("missing prepared cache root")])?;
    let key = prepared_root
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| vec![cargo_resolution_error("invalid prepared cache key")])?;
    crate::cache_storage::directory(prepared_root)
        .map_err(|error| vec![cargo_resolution_error(error.to_string())])?;
    let cache_lease = crate::cache_storage::entry_lock(cache_root, key)
        .map_err(|error| vec![cargo_resolution_error(error.to_string())])?;
    #[cfg(unix)]
    crate::cache_storage::lock_bounded(
        &cache_lease,
        &cache_root.join(".locks").join(key),
        false,
        crate::cache_storage::LEASE_WAIT,
    )
    .map_err(|error| vec![cargo_resolution_error(error.to_string())])?;
    #[cfg(windows)]
    cache_lease
        .lock()
        .map_err(|error| vec![cargo_resolution_error(error.to_string())])?;
    let scope = crate::cache_storage::owner_scope()
        .map_err(|error| vec![cargo_resolution_error(error.to_string())])?;
    let owner_path = prepared_root.join("resolution_owner.json");
    let owner = serde_json::json!({"schema":1, "key":key, "owner_scope":scope});
    match std::fs::symlink_metadata(&owner_path) {
        Ok(_) => {
            crate::cache_storage::payload(prepared_root, Path::new("resolution_owner.json"))
                .map_err(|error| vec![cargo_resolution_error(error.to_string())])?;
            let existing = std::fs::read(&owner_path)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
            if existing.as_ref() != Some(&owner) {
                return Err(vec![cargo_resolution_error(
                    "prepared Cargo cache owner mismatch",
                )]);
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut marker = crate::cache_storage::new_private_file(&owner_path)
                .map_err(|error| vec![cargo_resolution_error(error.to_string())])?;
            let bytes = serde_json::to_vec(&owner)
                .map_err(|error| vec![cargo_resolution_error(error.to_string())])?;
            std::io::Write::write_all(&mut marker, &bytes)
                .and_then(|()| marker.sync_all())
                .map_err(|error| vec![cargo_resolution_error(error.to_string())])?;
        }
        Err(error) => return Err(vec![cargo_resolution_error(error.to_string())]),
    }
    let marker_path = project_dir.join(".sifr-cargo-resolution");
    let previous_marker = match std::fs::read_to_string(&marker_path) {
        Ok(marker) => Some(marker),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(vec![cargo_resolution_error(error.to_string())]),
    };
    let existing_digest = digest_file_checked(&lock_path).map_err(|error| {
        vec![cargo_resolution_error(format!(
            "unreadable generated Cargo lockfile: {error}"
        ))]
    })?;
    let expected_marker = existing_digest
        .as_ref()
        .map(|digest| format!("{}\n{digest}\n", prepared_lock.display()));
    // The editable root outlives one generated manifest. A lock prepared for
    // the previous manifest cannot be used by a later --locked Cargo build.
    // The prepared cache key includes the normalized manifest and authorities;
    // keep the target warm while reconciling only the generated lock.
    if !lock_path.is_file() || previous_marker.as_deref() != expected_marker.as_deref() {
        if prepared_lock.is_file() {
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
            if let Some(parent) = prepared_lock.parent() {
                crate::cache_storage::directory(parent).map_err(|error| {
                    vec![cargo_resolution_error(format!(
                        "failed to create prepared Cargo resolution cache '{}': {error}",
                        parent.display()
                    ))]
                })?;
            }
            cache_prepared_lock(&lock_path, &prepared_lock)?;
        }
    }
    validate_authoritative_registry_entries(
        &lock_path,
        &policy.authoritative_locks,
        &policy.trusted_vendor_dirs,
    )?;
    let initial_digest = digest_file_checked(&lock_path)
        .map_err(|error| {
            vec![cargo_resolution_error(format!(
                "unreadable prepared Cargo lockfile: {error}"
            ))]
        })?
        .ok_or_else(|| {
            vec![cargo_resolution_error(format!(
                "prepared Cargo lockfile '{}' is unreadable",
                lock_path.display()
            ))]
        })?;
    let marker = format!("{}\n{initial_digest}\n", prepared_lock.display());
    super::native_storage::write_changed(&marker_path, marker.as_bytes())
        .map_err(|error| vec![cargo_resolution_error(error.to_string())])?;
    // The prepared payload is immutable now. Keep it leased while Cargo uses it,
    // while allowing concurrent readers of the same prepared resolution.
    #[cfg(unix)]
    crate::cache_storage::lock_bounded(
        &cache_lease,
        &cache_root.join(".locks").join(key),
        true,
        crate::cache_storage::LEASE_WAIT,
    )
    .map_err(|error| vec![cargo_resolution_error(error.to_string())])?;
    Ok(PreparedCargoResolution {
        lock_path,
        initial_digest: Some(initial_digest),
        lock_mode: policy.lock_mode,
        authority_check: Some(PreparedAuthorityCheck {
            project_dir: project_dir.to_path_buf(),
            policy: policy.clone(),
            cargo_prefix_args: cargo_prefix_args.to_vec(),
            prepared_lock,
        }),
        _cache_lease: Some(cache_lease),
    })
}

#[cfg(test)]
#[path = "cargo_resolution_normal_tests.rs"]
mod normal_tests;

fn cache_prepared_lock(
    source_lock: &Path,
    prepared_lock: &Path,
) -> Result<(), Vec<RenderedDiagnostic>> {
    if prepared_lock.is_file() {
        return Ok(());
    }
    let nonce = PREPARED_LOCK_NONCE.fetch_add(1, Ordering::Relaxed);
    let temporary = prepared_lock.with_extension(format!("tmp-{}-{nonce}", std::process::id()));
    std::fs::copy(source_lock, &temporary).map_err(|error| {
        vec![cargo_resolution_error(format!(
            "failed to stage prepared Cargo lockfile '{}': {error}",
            temporary.display()
        ))]
    })?;
    if let Err(error) = std::fs::rename(&temporary, prepared_lock) {
        let _ = std::fs::remove_file(&temporary);
        if !prepared_lock.is_file() {
            return Err(vec![cargo_resolution_error(format!(
                "failed to publish prepared Cargo lockfile '{}': {error}",
                prepared_lock.display()
            ))]);
        }
    }
    Ok(())
}

impl PreparedCargoResolution {
    pub(crate) fn assert_unchanged(&self) -> Result<(), Vec<RenderedDiagnostic>> {
        if self.lock_mode == CargoLockMode::Normal {
            return Ok(());
        }
        if let Some(check) = &self.authority_check {
            let current_key =
                prepared_lock_path(&check.project_dir, &check.policy, &check.cargo_prefix_args)?;
            if current_key != check.prepared_lock {
                return Err(vec![cargo_resolution_error(
                    "Cargo resolution authority changed while building",
                )]);
            }
        }
        // Cargo's standalone `--offline` mode can update a source lockfile.
        // Generated interop projects are stricter: their prepared lock is a
        // validated cache artifact, so every constrained mode keeps it
        // byte-identical.
        let current = digest_file_checked(&self.lock_path).map_err(|error| {
            vec![cargo_resolution_error(format!(
                "unreadable prepared Cargo lockfile {}: {error}",
                self.lock_path.display()
            ))]
        })?;
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

pub(crate) fn cargo_lock_mode_diagnostic(
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
    seed_lockfile_for_resolution(
        &project_dir.join("Cargo.lock"),
        &policy.authoritative_locks,
        cargo_prefix_args,
    )?;
    let mut command = policy.cargo_command()?;
    command
        .args(cargo_prefix_args)
        .args(["metadata", "--format-version=1"])
        .arg("--manifest-path")
        .arg(project_dir.join("Cargo.toml"));
    if policy.lock_mode.is_network_disallowed() {
        command.arg("--offline");
    }
    record_cargo_invocation("resolution", policy.lock_mode, &command);
    let output = crate::process_execution::output(&mut command).map_err(|error| {
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

fn seed_lockfile_for_resolution(
    destination: &Path,
    authoritative_locks: &[PathBuf],
    cargo_prefix_args: &[String],
) -> Result<(), Vec<RenderedDiagnostic>> {
    if cargo_prefix_args.is_empty() {
        return seed_lockfile_from_authorities(destination, authoritative_locks);
    }

    // A release sysroot's vendored source is itself an immutable authority.
    // Start from an empty lock so Cargo selects only versions actually present
    // in that source; importing a newer workspace lock can otherwise pin a
    // package absent from the release vendor directory.
    std::fs::write(destination, "version = 4\n").map_err(|error| {
        vec![cargo_resolution_error(format!(
            "failed to initialize generated Cargo lockfile at '{}': {error}",
            destination.display()
        ))]
    })
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

fn cargo_resolution_error(message: impl Into<String>) -> RenderedDiagnostic {
    diagnostic_with_code(message.into(), DiagnosticCode::RUST_CARGO_METADATA)
}

#[cfg(test)]
mod tests {
    use super::identity::normalized_manifest_cache_input;
    use super::{
        CargoResolutionPolicy, CargoVendorMode, PREPARED_LOCK_NONCE, registry_entries,
        registry_version_compatibility_family, seed_lockfile_for_resolution,
        seed_lockfile_from_authorities, validate_authoritative_registry_entries,
    };
    use sifr_package::CargoLockMode;
    use std::collections::BTreeSet;
    use std::path::PathBuf;
    use std::sync::atomic::Ordering;

    #[test]
    fn probe_vendor_replacement_follows_resolution_ownership() {
        assert!(CargoResolutionPolicy::normal().uses_sysroot_vendor());
        let package_owned = CargoResolutionPolicy {
            application_profile: crate::ApplicationProfile::Release,
            native_toolchain: CargoResolutionPolicy::resolve_native_toolchain(),
            lock_mode: CargoLockMode::Frozen,
            cargo_vendor_mode: CargoVendorMode::PackageOwned,
            authoritative_locks: Vec::new(),
            trusted_vendor_dirs: Vec::new(),
        };
        assert!(!package_owned.uses_sysroot_vendor());
    }

    #[test]
    fn vendor_resolution_does_not_import_unavailable_workspace_pins() {
        let root = std::env::temp_dir().join(format!(
            "sifr_vendor_seed_{}_{}",
            std::process::id(),
            PREPARED_LOCK_NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).expect("test root should be created");
        let authority = root.join("authority.lock");
        let destination = root.join("Cargo.lock");
        std::fs::write(
            &authority,
            "version = 4\n\n[[package]]\nname = \"newer-than-vendor\"\nversion = \"9.9.9\"\n",
        )
        .expect("authority should be written");

        seed_lockfile_for_resolution(
            &destination,
            &[authority],
            &["--config".to_string(), "source replacement".to_string()],
        )
        .expect("vendor seed should initialize");

        assert_eq!(
            std::fs::read_to_string(&destination).expect("seed should be readable"),
            "version = 4\n"
        );
        let _ = std::fs::remove_dir_all(root);
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
