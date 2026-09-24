use super::digest_file_checked;
use super::{
    CargoResolutionPolicy, PREPARED_LOCK_NONCE, PreparedCargoResolution, prepare_cargo_resolution,
};
use sifr_package::CargoLockMode;
use sifr_stdlib_manifest::CargoVendorMode;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "sifr_normal_lock_seed_{}_{}",
            std::process::id(),
            PREPARED_LOCK_NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).expect("fixture root");
        Self(root)
    }

    fn policy(&self, source: &str) -> CargoResolutionPolicy {
        let authority = self.0.join("package.lock");
        std::fs::write(&authority, source).expect("package lock");
        CargoResolutionPolicy {
            application_profile: crate::ApplicationProfile::Release,
            native_toolchain: CargoResolutionPolicy::resolve_native_toolchain(),
            lock_mode: CargoLockMode::Normal,
            cargo_vendor_mode: CargoVendorMode::PackageOwned,
            authoritative_locks: vec![authority],
            trusted_vendor_dirs: Vec::new(),
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const PACKAGE_LOCK: &str = r#"version = 4

[[package]]
name = "locked-library"
version = "0.10.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "accepted-package-checksum"
"#;

#[test]
fn normal_generated_resolution_seeds_existing_package_selection_without_source_mutation() {
    let fixture = Fixture::new();
    let policy = fixture.policy(PACKAGE_LOCK);
    let prepared = prepare_cargo_resolution(&fixture.0, &policy, &[]).expect("seed package pins");
    let generated = std::fs::read_to_string(fixture.0.join("Cargo.lock")).expect("generated lock");
    assert_eq!(
        generated.parse::<toml::Table>().expect("generated TOML"),
        PACKAGE_LOCK.parse::<toml::Table>().expect("original TOML")
    );
    assert_eq!(
        std::fs::read_to_string(&policy.authoritative_locks[0]).expect("original lock"),
        PACKAGE_LOCK
    );
    // Preserve normal Cargo semantics, not an implicit switch to --locked.
    std::fs::write(fixture.0.join("Cargo.lock"), "version = 4\n").expect("normal update");
    prepared
        .assert_unchanged()
        .expect("normal mode permits updates");
}

#[test]
fn normal_generated_resolution_preserves_an_existing_generated_lock() {
    let fixture = Fixture::new();
    let policy = fixture.policy(PACKAGE_LOCK);
    let existing = "# generated selection\nversion = 4\n";
    std::fs::write(fixture.0.join("Cargo.lock"), existing).expect("existing generated lock");
    prepare_cargo_resolution(&fixture.0, &policy, &[]).expect("normal existing resolution");
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("Cargo.lock")).expect("generated lock"),
        existing
    );
}

#[test]
fn normal_generated_resolution_without_authority_does_not_invent_a_lock() {
    let fixture = Fixture::new();
    prepare_cargo_resolution(&fixture.0, &CargoResolutionPolicy::normal(), &[])
        .expect("normal unconstrained resolution");
    assert!(!fixture.0.join("Cargo.lock").exists());
}

#[test]
fn normal_generated_resolution_rejects_invalid_authority_without_fresh_resolution() {
    let fixture = Fixture::new();
    let policy = fixture.policy("invalid TOML [");
    let errors = match prepare_cargo_resolution(&fixture.0, &policy, &[]) {
        Err(errors) => errors,
        Ok(_) => panic!("invalid authority must not be discarded"),
    };
    assert!(
        errors[0]
            .message
            .contains("failed to parse Cargo lock authority")
    );
    assert!(!fixture.0.join("Cargo.lock").exists());
}

#[test]
fn normal_generated_vendor_resolution_keeps_the_vendor_inventory_authoritative() {
    let fixture = Fixture::new();
    let policy = fixture.policy(PACKAGE_LOCK);
    prepare_cargo_resolution(
        &fixture.0,
        &policy,
        &["--config".to_string(), "source replacement".to_string()],
    )
    .expect("existing vendor source selection policy");
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("Cargo.lock")).expect("vendor seed"),
        "version = 4\n"
    );
}

#[test]
fn normal_seed_cache_identity_tracks_ordered_authorities_without_resetting_other_modes() {
    let fixture = Fixture::new();
    let mut policy = fixture.policy(PACKAGE_LOCK);
    let first = policy.normal_seed_cache_fragment().expect("seed identity");
    assert_eq!(
        policy.normal_seed_cache_fragment().as_deref(),
        Some(first.as_str())
    );
    let second_lock = fixture.0.join("sysroot.lock");
    std::fs::write(&second_lock, "version = 4\n").expect("second authority");
    policy.authoritative_locks.push(second_lock.clone());
    let two = policy
        .normal_seed_cache_fragment()
        .expect("two authorities");
    assert_ne!(two, first);
    policy.authoritative_locks.reverse();
    assert_ne!(
        policy.normal_seed_cache_fragment().as_deref(),
        Some(two.as_str())
    );
    policy.authoritative_locks.reverse();
    std::fs::write(&second_lock, "version = 3\n").expect("changed selection");
    assert_ne!(
        policy.normal_seed_cache_fragment().as_deref(),
        Some(two.as_str())
    );
    let package_owned = policy.normal_seed_cache_fragment();
    policy.cargo_vendor_mode = CargoVendorMode::SysrootOnly;
    assert_ne!(policy.normal_seed_cache_fragment(), package_owned);
    policy.lock_mode = CargoLockMode::Locked;
    assert_eq!(policy.normal_seed_cache_fragment(), None);
    assert_eq!(
        CargoResolutionPolicy::normal().normal_seed_cache_fragment(),
        None
    );
}

#[test]
fn normal_seed_identity_distinguishes_absent_empty_and_unreadable_authority() {
    let fixture = Fixture::new();
    let policy = fixture.policy(PACKAGE_LOCK);
    let original = policy.normal_seed_cache_fragment().unwrap();
    let authority = &policy.authoritative_locks[0];
    std::fs::remove_file(authority).unwrap();
    let absent = policy.normal_seed_cache_fragment().unwrap();
    assert_ne!(original, absent);
    assert!(prepare_cargo_resolution(&fixture.0, &policy, &[]).is_err());
    std::fs::write(authority, "").unwrap();
    let empty = policy.normal_seed_cache_fragment().unwrap();
    assert_ne!(absent, empty);
    #[cfg(unix)]
    {
        std::fs::remove_file(authority).unwrap();
        std::os::unix::fs::symlink(fixture.0.join("missing"), authority).unwrap();
        assert_ne!(empty, policy.normal_seed_cache_fragment().unwrap());
        assert!(prepare_cargo_resolution(&fixture.0, &policy, &[]).is_err());
    }
}

#[test]
fn dx9_changed_authority_reseeds_a_reused_generated_root() {
    let fixture = Fixture::new();
    let policy = fixture.policy(PACKAGE_LOCK);
    prepare_cargo_resolution(&fixture.0, &policy, &[]).expect("first seed");
    let lock = fixture.0.join("Cargo.lock");
    let generated = std::fs::read_to_string(&lock).expect("seed");
    let retained = format!("# Cargo updated this generated root\n{generated}");
    std::fs::write(&lock, &retained).expect("normal Cargo update");
    prepare_cargo_resolution(&fixture.0, &policy, &[]).expect("same authority");
    assert_eq!(
        std::fs::read_to_string(&lock).expect("retained pins"),
        retained
    );
    let changed = PACKAGE_LOCK.replace("0.10.0", "0.11.0");
    std::fs::write(&policy.authoritative_locks[0], &changed).expect("new source selection");
    prepare_cargo_resolution(&fixture.0, &policy, &[]).expect("reseed changed authority");
    let actual = std::fs::read_to_string(&lock).expect("new generated selection");
    assert!(actual.contains("0.11.0"));
    assert!(!actual.contains("0.10.0"));
    assert_eq!(
        std::fs::read_to_string(&policy.authoritative_locks[0]).expect("source authority"),
        changed
    );
}

#[test]
fn constrained_resolution_rejects_changed_missing_and_unreadable_lock_payload() {
    let fixture = Fixture::new();
    let lock_path = fixture.0.join("Cargo.lock");
    std::fs::write(&lock_path, "version = 4\n").unwrap();
    let prepared = PreparedCargoResolution {
        initial_digest: digest_file_checked(&lock_path).unwrap(),
        lock_path: lock_path.clone(),
        lock_mode: CargoLockMode::Locked,
        authority_check: None,
        _cache_lease: None,
    };
    prepared.assert_unchanged().unwrap();
    std::fs::write(&lock_path, "").unwrap();
    assert!(prepared.assert_unchanged().is_err());
    std::fs::remove_file(&lock_path).unwrap();
    assert!(prepared.assert_unchanged().is_err());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(fixture.0.join("missing.lock"), &lock_path).unwrap();
        assert!(prepared.assert_unchanged().is_err());
    }
}
