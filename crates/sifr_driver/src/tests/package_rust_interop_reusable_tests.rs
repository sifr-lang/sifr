use super::*;
use std::time::{Duration, SystemTime};

fn acquire(base: &Path, owner: &Path, inputs: Inputs) -> io::Result<ReusableScenario> {
    ReusableScenario::acquire(base, owner, FIXTURE, SCENARIO, TEST, inputs)
}

pub(super) fn private_root() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    }
    root
}

fn inputs() -> Inputs {
    BTreeMap::from([
        ("Cargo.toml".into(), b"final manifest".to_vec()),
        ("src/main.sifr".into(), b"final evidence".to_vec()),
        ("replace-file".into(), b"old file".to_vec()),
        ("replace-dir/nested".into(), b"old nested".to_vec()),
    ])
}

pub(super) fn assert_reconciliation() {
    let temp = private_root();
    let base = temp.path().join("fixtures");
    let original = inputs();
    let first = acquire(&base, temp.path(), original.clone()).unwrap();
    let root = first.root.clone();
    let key = root.file_name().unwrap().to_str().unwrap();
    let lock = crate::build::native_child_leases::open(&base, key).unwrap();
    assert!(matches!(lock.try_lock(), Err(fs::TryLockError::WouldBlock)));
    let stamp = SystemTime::UNIX_EPOCH + Duration::from_secs(1234);
    for path in original.keys() {
        File::options()
            .write(true)
            .open(root.join(path))
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(stamp))
            .unwrap();
    }
    drop(first);
    assert!(
        root.is_dir(),
        "input lifetime is independent of assertion completion"
    );
    let second = acquire(&base, temp.path(), original.clone()).unwrap();
    assert_eq!(second.root, root);
    for (path, bytes) in &original {
        assert_eq!(fs::read(root.join(path)).unwrap(), *bytes);
        assert_eq!(
            fs::metadata(root.join(path)).unwrap().modified().unwrap(),
            stamp
        );
    }
    fs::write(root.join("Cargo.toml"), "altered authority").unwrap();
    fs::remove_file(root.join("src/main.sifr")).unwrap();
    crate::cache_storage::directory(&root.join("stale/nested")).unwrap();
    crate::build::native_storage::write_changed(&root.join("stale/nested/file"), b"stale").unwrap();
    drop(second);
    let mut current = original;
    current.remove(Path::new("replace-file"));
    current.remove(Path::new("replace-dir/nested"));
    current.insert("replace-file/nested".into(), b"new nested".to_vec());
    current.insert("replace-dir".into(), b"new file".to_vec());
    let third = acquire(&base, temp.path(), current.clone()).unwrap();
    assert_eq!(read_inputs(&third.root).unwrap(), current);
    assert!(!root.join("stale").exists());
    assert!(matches!(lock.try_lock(), Err(fs::TryLockError::WouldBlock)));
    drop(third);
    lock.try_lock().unwrap();
}

#[test]
fn final_overlay_is_reconciled_once_and_canonical_source_stays_unchanged() {
    let temp = private_root();
    let canonical = temp.path().join("canonical");
    crate::cache_storage::directory(&canonical).unwrap();
    crate::build::native_storage::write_changed(
        &canonical.join("Cargo.toml"),
        b"[dependencies]\nsifr_runtime={path='../../../../../../../crates/sifr_runtime'}\n",
    )
    .unwrap();
    let before = read_inputs(&canonical).unwrap();
    let mut final_inputs = before.clone();
    let rebased =
        rebased_runtime_manifest(std::str::from_utf8(&before[Path::new("Cargo.toml")]).unwrap());
    final_inputs.insert("Cargo.toml".into(), rebased.into_bytes());
    final_inputs.insert("src/main.sifr".into(), b"final assertion source".to_vec());
    let first = acquire(
        &temp.path().join("cache"),
        temp.path(),
        final_inputs.clone(),
    )
    .unwrap();
    let manifest = first.root.join("Cargo.toml");
    let stamp = SystemTime::UNIX_EPOCH + Duration::from_secs(5678);
    File::options()
        .write(true)
        .open(&manifest)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(stamp))
        .unwrap();
    drop(first);
    let second = acquire(
        &temp.path().join("cache"),
        temp.path(),
        final_inputs.clone(),
    )
    .unwrap();
    assert_eq!(fs::metadata(&manifest).unwrap().modified().unwrap(), stamp);
    assert_eq!(read_inputs(&second.root).unwrap(), final_inputs);
    assert_eq!(read_inputs(&canonical).unwrap(), before);
}

#[test]
fn input_limits_and_invalid_layout_fail_before_retained_input_mutation() {
    let temp = private_root();
    let base = temp.path().join("cache");
    let initial = inputs();
    let first = acquire(&base, temp.path(), initial.clone()).unwrap();
    let root = first.root.clone();
    drop(first);
    for invalid_inputs in [
        BTreeMap::from([("huge".into(), vec![0; MAX_BYTES + 1])]),
        (0..=MAX_FILES)
            .map(|i| (PathBuf::from(format!("file-{i}")), vec![]))
            .collect(),
        BTreeMap::from([("a".into(), vec![]), ("a/b".into(), vec![])]),
        BTreeMap::from([("../escape".into(), vec![])]),
    ] {
        assert!(acquire(&base, temp.path(), invalid_inputs).is_err());
        assert_eq!(read_inputs(&root).unwrap(), initial);
    }
    let sparse = temp.path().join("huge-source");
    fs::create_dir(&sparse).unwrap();
    File::create(sparse.join("huge"))
        .unwrap()
        .set_len((MAX_BYTES + 1) as u64)
        .unwrap();
    assert!(read_inputs(&sparse).is_err());
    assert!(!temp.path().join("escape").exists());
}

#[cfg(unix)]
#[test]
fn source_and_retained_aliases_are_rejected_without_touching_their_targets() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let temp = private_root();
    let outside = temp.path().join("outside");
    fs::write(&outside, "untouched").unwrap();
    fs::set_permissions(&outside, fs::Permissions::from_mode(0o600)).unwrap();
    let source = temp.path().join("source");
    fs::create_dir(&source).unwrap();
    symlink(&outside, source.join("alias")).unwrap();
    assert!(read_inputs(&source).is_err());
    let fixture = acquire(&temp.path().join("cache"), temp.path(), inputs()).unwrap();
    let root = fixture.root.clone();
    drop(fixture);
    let manifest = root.join("Cargo.toml");
    fs::remove_file(&manifest).unwrap();
    symlink(&outside, &manifest).unwrap();
    assert!(reconcile(&root, &inputs()).is_err());
    assert_eq!(fs::read_to_string(&outside).unwrap(), "untouched");
    fs::remove_file(&manifest).unwrap();
    fs::hard_link(&outside, &manifest).unwrap();
    assert!(reconcile(&root, &inputs()).is_err());
    assert_eq!(fs::read_to_string(&outside).unwrap(), "untouched");
    fs::remove_file(&manifest).unwrap();
    symlink(&source, root.join("stale-alias")).unwrap();
    assert!(reconcile(&root, &inputs()).is_err());
    assert!(source.join("alias").is_symlink());
}

#[cfg(unix)]
#[path = "package_rust_interop_reusable_child_tests.rs"]
mod child;

#[test]
fn variants_are_isolated_and_current_inputs_replace_each_authority_kind() {
    let temp = private_root();
    let base = temp.path().join("cache");
    let original: Inputs = [
        "src/main.sifr",
        "Cargo.toml",
        "Cargo.lock",
        "src/bridge.rs",
        "sifr.toml",
    ]
    .into_iter()
    .map(|name| (name.into(), b"original".to_vec()))
    .collect();
    let first = acquire(&base, temp.path(), original.clone()).unwrap();
    let first_root = first.root.clone();
    // Every ownership coordinate is significant; contents are deliberately identical.
    for (fixture, scenario, test, owner) in [
        ("other-fixture", SCENARIO, TEST, temp.path().to_owned()),
        (FIXTURE, "other-scenario", TEST, temp.path().to_owned()),
        (FIXTURE, SCENARIO, "other-variant", temp.path().to_owned()),
        (FIXTURE, SCENARIO, TEST, temp.path().join("other-checkout")),
    ] {
        let other =
            ReusableScenario::acquire(&base, &owner, fixture, scenario, test, original.clone())
                .unwrap();
        assert_ne!(first_root, other.root);
        fs::write(other.root.join("src/main.sifr"), "other variant mutation").unwrap();
        assert_eq!(read_inputs(&first_root).unwrap(), original);
    }
    drop(first);
    let mut current = original;
    for name in [
        "src/main.sifr",
        "Cargo.toml",
        "Cargo.lock",
        "src/bridge.rs",
        "sifr.toml",
    ] {
        current.insert(name.into(), format!("current {name}").into_bytes());
        let updated = acquire(&base, temp.path(), current.clone()).unwrap();
        assert_eq!(
            updated.root, first_root,
            "input changes keep the same owned directory"
        );
        assert_eq!(
            read_inputs(&updated.root).unwrap(),
            current,
            "current bytes, never a cached pass"
        );
    }
}

#[cfg(unix)]
#[path = "package_rust_interop_reusable_permissions_tests.rs"]
mod permissions;
