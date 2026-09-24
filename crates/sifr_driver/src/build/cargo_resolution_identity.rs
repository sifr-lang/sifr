use super::super::workspace::artifact_cache_root;
use super::digest_file_checked;
use super::{CargoResolutionPolicy, cargo_resolution_error};
use crate::diagnostics::RenderedDiagnostic;
use sifr_identity::IdentityEncoder;
use sifr_package::CargoLockMode;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const PREPARED_RESOLUTION_DIR: &str = "cargo_resolution";

fn lock_state(identity: &mut IdentityEncoder, path: &Path) -> io::Result<bool> {
    identity.field("lock-path", path.as_os_str().as_encoded_bytes());
    let digest = digest_file_checked(path)?;
    identity.field("lock-present", &[u8::from(digest.is_some())]);
    if let Some(digest) = digest {
        identity.field("lock-digest", digest.as_bytes());
        Ok(true)
    } else {
        Ok(false)
    }
}

pub(super) fn checked_authorities(paths: &[PathBuf]) -> Result<(), Vec<RenderedDiagnostic>> {
    for path in paths {
        match digest_file_checked(path) {
            Ok(Some(_)) => {}
            Ok(None) => {
                return Err(vec![cargo_resolution_error(format!(
                    "authoritative Cargo lockfile '{}' is missing",
                    path.display()
                ))]);
            }
            Err(error) => {
                return Err(vec![cargo_resolution_error(format!(
                    "authoritative Cargo lockfile '{}' is unreadable: {error}",
                    path.display()
                ))]);
            }
        }
    }
    Ok(())
}

impl CargoResolutionPolicy {
    pub(crate) fn normal_seed_cache_fragment(&self) -> Option<String> {
        if self.lock_mode != CargoLockMode::Normal || self.authoritative_locks.is_empty() {
            return None;
        }
        let mut identity = IdentityEncoder::new("normal-authority-seed-v3");
        identity.field(
            "toolchain",
            self.native_toolchain
                .as_ref()
                .map_or("<unavailable>", |tools| tools.identity())
                .as_bytes(),
        );
        identity.field("lock-mode", self.lock_mode.as_str().as_bytes());
        identity.field(
            "vendor-mode",
            format!("{:?}", self.cargo_vendor_mode).as_bytes(),
        );
        identity.field(
            "authority-count",
            &(self.authoritative_locks.len() as u64).to_be_bytes(),
        );
        // Order is significant: earlier authorities override later ones.
        for lock in &self.authoritative_locks {
            if let Err(error) = lock_state(&mut identity, lock) {
                identity.field("lock-unreadable", error.kind().to_string().as_bytes());
            }
        }
        Some(identity.finish())
    }
}

pub(super) fn prepared_lock_path(
    project_dir: &Path,
    policy: &CargoResolutionPolicy,
    cargo_prefix_args: &[String],
) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    let mut identity = IdentityEncoder::new("prepared-cargo-resolution-v10");
    let scope = crate::cache_storage::owner_scope().map_err(|error| {
        vec![cargo_resolution_error(format!(
            "unavailable prepared Cargo owner scope: {error}"
        ))]
    })?;
    identity.field("owner-scope", scope.as_os_str().as_encoded_bytes());
    let tools = policy
        .native_toolchain
        .as_ref()
        .map_err(|error| vec![cargo_resolution_error(error.clone())])?;
    identity.field("toolchain", tools.identity().as_bytes());
    identity.field(
        "manifest",
        normalized_manifest_cache_input(project_dir)?.as_bytes(),
    );
    identity.field("lock-mode", policy.lock_mode.as_str().as_bytes());
    identity.field(
        "vendor-mode",
        format!("{:?}", policy.cargo_vendor_mode).as_bytes(),
    );
    identity.field(
        "prefix-count",
        &(cargo_prefix_args.len() as u64).to_be_bytes(),
    );
    for argument in cargo_prefix_args {
        identity.field("prefix-arg", argument.as_bytes());
    }
    identity.field(
        "authority-count",
        &(policy.authoritative_locks.len() as u64).to_be_bytes(),
    );
    for lock in &policy.authoritative_locks {
        if !lock_state(&mut identity, lock).map_err(|error| {
            vec![cargo_resolution_error(format!(
                "unreadable authoritative Cargo lockfile '{}': {error}",
                lock.display()
            ))]
        })? {
            return Err(vec![cargo_resolution_error(format!(
                "authoritative Cargo lockfile '{}' is missing",
                lock.display()
            ))]);
        }
    }
    identity.field(
        "trusted-vendor-count",
        &(policy.trusted_vendor_dirs.len() as u64).to_be_bytes(),
    );
    for vendor_dir in &policy.trusted_vendor_dirs {
        identity.field(
            "trusted-vendor-path",
            vendor_dir.as_os_str().as_encoded_bytes(),
        );
        let digest = trusted_vendor_identity(vendor_dir).map_err(|error| {
            vec![cargo_resolution_error(format!(
                "unreadable trusted Cargo vendor directory '{}': {error}",
                vendor_dir.display()
            ))]
        })?;
        identity.field("trusted-vendor-inventory", digest.as_bytes());
    }
    Ok(artifact_cache_root()
        .join(PREPARED_RESOLUTION_DIR)
        .join(identity.finish())
        .join("Cargo.lock"))
}

// Registry authority uses these two files per candidate crate. Cargo itself
// validates source payload during the build before finalized output is reused.
fn trusted_vendor_identity(root: &Path) -> io::Result<String> {
    let mut entries = fs::read_dir(root)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<_>>>()?;
    entries.sort();
    let mut identity = IdentityEncoder::new("trusted-vendor-registry-v1");
    identity.field("entry-count", &(entries.len() as u64).to_be_bytes());
    for entry in entries {
        let name = entry.file_name().unwrap_or_default();
        identity.field("entry", name.as_encoded_bytes());
        let is_dir = entry.is_dir();
        identity.field("directory", &[u8::from(is_dir)]);
        if is_dir {
            for file in ["Cargo.toml", ".cargo-checksum.json"] {
                identity.field("file", file.as_bytes());
                let digest = digest_file_checked(&entry.join(file))?;
                identity.field("present", &[u8::from(digest.is_some())]);
                if let Some(digest) = digest {
                    identity.field("digest", digest.as_bytes());
                }
            }
        }
    }
    Ok(identity.finish())
}

pub(super) fn normalized_manifest_cache_input(
    project_dir: &Path,
) -> Result<String, Vec<RenderedDiagnostic>> {
    let manifest_path = project_dir.join("Cargo.toml");
    let source = fs::read_to_string(&manifest_path).map_err(|error| {
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
    normalize_path_dependency_identities(&mut manifest, project_dir)?;
    toml::to_string(&manifest).map_err(|error| {
        vec![cargo_resolution_error(format!(
            "failed to normalize generated Cargo manifest: {error}"
        ))]
    })
}

fn normalize_path_dependency_identities(
    value: &mut toml::Value,
    project_dir: &Path,
) -> Result<(), Vec<RenderedDiagnostic>> {
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
                        // Manifest `path` also names bin/lib source files. Only
                        // directories can be path dependencies.
                        match fs::metadata(&dependency_root) {
                            Ok(metadata) if metadata.is_file() => continue,
                            Ok(_) => {}
                            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                            Err(error) => {
                                return Err(vec![cargo_resolution_error(format!(
                                    "unreadable manifest path '{}': {error}",
                                    dependency_root.display()
                                ))]);
                            }
                        }
                        let dependency_manifest = dependency_root.join("Cargo.toml");
                        let digest =
                            digest_file_checked(&dependency_manifest).map_err(|error| {
                                vec![cargo_resolution_error(format!(
                                    "unreadable path dependency manifest '{}': {error}",
                                    dependency_manifest.display()
                                ))]
                            })?;
                        if let Some(digest) = digest {
                            *nested = toml::Value::String(format!(
                                "sifr-path-dependency-manifest:{digest}"
                            ));
                            continue;
                        }
                    }
                }
                normalize_path_dependency_identities(nested, project_dir)?;
            }
        }
        toml::Value::Array(values) => {
            for nested in values {
                normalize_path_dependency_identities(nested, project_dir)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::CargoResolutionPolicy;
    use super::{prepared_lock_path, trusted_vendor_identity};
    use sifr_package::CargoLockMode;
    use sifr_stdlib_manifest::CargoVendorMode;
    use std::fs;

    #[test]
    fn source_path_fields_are_not_dependency_manifests() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("src")).unwrap();
        fs::write(root.path().join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(root.path().join("Cargo.toml"),
            "[package]\nname = \"generated\"\nversion = \"0.1.0\"\n[[bin]]\nname = \"generated\"\npath = \"src/main.rs\"\n").unwrap();
        assert!(super::normalized_manifest_cache_input(root.path()).is_ok());
    }

    #[test]
    fn prepared_key_binds_resolution_policy_and_authorities() {
        let root = tempfile::tempdir().unwrap();
        fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname = \"generated\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let first = root.path().join("first.lock");
        let second = root.path().join("second.lock");
        fs::write(&first, "version = 4\n").unwrap();
        fs::write(&second, "version = 4\n").unwrap();
        let mut policy = CargoResolutionPolicy::normal();
        policy.lock_mode = CargoLockMode::Locked;
        policy.authoritative_locks = vec![first.clone(), second.clone()];
        let key =
            |policy: &CargoResolutionPolicy| prepared_lock_path(root.path(), policy, &[]).unwrap();
        let original = key(&policy);
        assert_eq!(original, key(&policy));
        policy.lock_mode = CargoLockMode::Frozen;
        assert_ne!(original, key(&policy));
        policy.lock_mode = CargoLockMode::Offline;
        assert_ne!(original, key(&policy));
        policy.lock_mode = CargoLockMode::Locked;
        policy.cargo_vendor_mode = CargoVendorMode::PackageOwned;
        assert_ne!(original, key(&policy));
        policy.cargo_vendor_mode = CargoVendorMode::SysrootOnly;
        policy.authoritative_locks.reverse();
        assert_ne!(original, key(&policy));
        policy.authoritative_locks.reverse();
        fs::write(&second, "version = 3\n").unwrap();
        assert_ne!(original, key(&policy));
        fs::write(&second, "version = 4\n").unwrap();
        assert_ne!(
            original,
            prepared_lock_path(root.path(), &policy, &["--offline".into()]).unwrap()
        );
        fs::remove_file(&first).unwrap();
        assert!(prepared_lock_path(root.path(), &policy, &[]).is_err());
        fs::write(&first, "").unwrap();
        assert_ne!(original, key(&policy));
        #[cfg(unix)]
        {
            fs::remove_file(&first).unwrap();
            std::os::unix::fs::symlink(root.path().join("missing"), &first).unwrap();
            assert!(prepared_lock_path(root.path(), &policy, &[]).is_err());
        }
    }

    #[test]
    fn unchanged_check_revalidates_authority_and_trusted_vendor_payload() {
        use super::super::{PreparedAuthorityCheck, PreparedCargoResolution};
        use super::digest_file_checked;
        let root = tempfile::tempdir().unwrap();
        fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname = \"generated\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let generated = root.path().join("Cargo.lock");
        let authority = root.path().join("authority.lock");
        fs::write(&generated, "version = 4\n").unwrap();
        fs::write(&authority, "version = 4\n").unwrap();
        let vendor = root.path().join("vendor");
        fs::create_dir(&vendor).unwrap();
        let mut policy = CargoResolutionPolicy::normal();
        policy.lock_mode = CargoLockMode::Locked;
        policy.authoritative_locks = vec![authority.clone()];
        policy.trusted_vendor_dirs = vec![vendor.clone()];
        let prepared_lock = prepared_lock_path(root.path(), &policy, &[]).unwrap();
        let prepared = PreparedCargoResolution {
            initial_digest: digest_file_checked(&generated).unwrap(),
            lock_path: generated,
            lock_mode: CargoLockMode::Locked,
            authority_check: Some(PreparedAuthorityCheck {
                project_dir: root.path().to_path_buf(),
                policy,
                cargo_prefix_args: vec![],
                prepared_lock,
            }),
            _cache_lease: None,
        };
        prepared.assert_unchanged().unwrap();
        fs::write(&authority, "version = 3\n").unwrap();
        assert!(prepared.assert_unchanged().is_err());
        fs::write(&authority, "version = 4\n").unwrap();
        prepared.assert_unchanged().unwrap();
        fs::create_dir(vendor.join("new-crate")).unwrap();
        assert!(prepared.assert_unchanged().is_err());
    }

    #[test]
    fn trusted_vendor_inventory_binds_payload_and_unreadable_states() {
        let root = tempfile::tempdir().unwrap();
        let vendor = root.path().join("vendor");
        assert!(trusted_vendor_identity(&vendor).is_err());
        fs::create_dir(&vendor).unwrap();
        let empty = trusted_vendor_identity(&vendor).unwrap();
        let crate_dir = vendor.join("crate-1.0.0");
        fs::create_dir(&crate_dir).unwrap();
        assert_ne!(empty, trusted_vendor_identity(&vendor).unwrap());
        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"crate\"\n",
        )
        .unwrap();
        let manifest = trusted_vendor_identity(&vendor).unwrap();
        assert_ne!(empty, manifest);
        fs::write(
            crate_dir.join(".cargo-checksum.json"),
            r#"{"package":"one"}"#,
        )
        .unwrap();
        let checksum = trusted_vendor_identity(&vendor).unwrap();
        assert_ne!(manifest, checksum);
        fs::write(
            crate_dir.join(".cargo-checksum.json"),
            r#"{"package":"two"}"#,
        )
        .unwrap();
        assert_ne!(checksum, trusted_vendor_identity(&vendor).unwrap());
        #[cfg(unix)]
        {
            fs::remove_file(crate_dir.join("Cargo.toml")).unwrap();
            std::os::unix::fs::symlink(root.path().join("missing"), crate_dir.join("Cargo.toml"))
                .unwrap();
            assert!(trusted_vendor_identity(&vendor).is_err());
        }
    }
}
