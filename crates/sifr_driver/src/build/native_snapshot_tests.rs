use super::NativeSnapshot;
use std::fs;
use std::path::{Path, PathBuf};

fn fixture() -> (tempfile::TempDir, PathBuf, Vec<PathBuf>) {
    let root = tempfile::tempdir().expect("owned bundle fixture");
    let executable = root.path().join("program");
    fs::write(&executable, b"executable").unwrap();
    fs::write(executable.with_extension("pdb"), b"symbols").unwrap();
    let dsym = root.path().join("program.dSYM");
    fs::create_dir(&dsym).unwrap();
    fs::write(dsym.join("b"), b"second debug").unwrap();
    fs::write(dsym.join("a"), b"first debug").unwrap();
    fs::create_dir(root.path().join("one")).unwrap();
    fs::create_dir(root.path().join("two")).unwrap();
    let libraries = vec![
        root.path().join("one/libfirst.so"),
        root.path().join("two/libsecond.so"),
    ];
    fs::write(&libraries[0], b"first library").unwrap();
    fs::write(&libraries[1], b"second library").unwrap();
    (root, executable, libraries)
}

#[test]
fn complete_inventory_preserves_legacy_identity_and_order() {
    let (_root, executable, libraries) = fixture();
    let destination = Path::new("bundle/program");
    let snapshot = NativeSnapshot::inspect(&executable, destination, &libraries).unwrap();
    assert_eq!(
        snapshot.required(),
        vec![
            destination,
            Path::new("bundle/program.pdb"),
            Path::new("bundle/program.dSYM/a"),
            Path::new("bundle/program.dSYM/b"),
            Path::new("bundle/libfirst.so"),
            Path::new("bundle/libsecond.so"),
        ]
    );
    // Independently encoded legacy domain/path/bytes records, not a digest of
    // the new implementation's emitted inventory. Native path bytes are Unix-specific.
    #[cfg(unix)]
    assert_eq!(
        snapshot.identity,
        "0dca2e2b5f4291315d35e644117abcce7ba70824aaef789a78b86825008df7d9"
    );
    let other =
        NativeSnapshot::inspect(&executable, Path::new("elsewhere/program"), &libraries).unwrap();
    assert_ne!(snapshot.identity, other.identity);
    let reversed = NativeSnapshot::inspect(
        &executable,
        destination,
        &[libraries[1].clone(), libraries[0].clone()],
    )
    .unwrap();
    assert_ne!(snapshot.identity, reversed.identity);
}

#[test]
fn fresh_identity_observes_content_changes_with_unchanged_size_and_mtime() {
    let (_root, executable, libraries) = fixture();
    let destination = Path::new("program");
    let before = NativeSnapshot::inspect(&executable, destination, &libraries).unwrap();
    let metadata = fs::metadata(&executable).unwrap();
    fs::write(&executable, b"alteration").unwrap();
    fs::File::options()
        .write(true)
        .open(&executable)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(metadata.modified().unwrap()))
        .unwrap();
    assert_eq!(fs::metadata(&executable).unwrap().len(), metadata.len());
    assert_eq!(
        fs::metadata(&executable).unwrap().modified().unwrap(),
        metadata.modified().unwrap()
    );
    let after = NativeSnapshot::inspect(&executable, destination, &libraries).unwrap();
    assert_ne!(before.identity, after.identity);
}

#[test]
fn runtime_basename_deduplicates_only_equal_payloads() {
    let (root, executable, mut libraries) = fixture();
    let destination = Path::new("program");
    let initial = NativeSnapshot::inspect(&executable, destination, &libraries).unwrap();
    let duplicate = root.path().join("two/libfirst.so");
    fs::write(&duplicate, b"first library").unwrap();
    libraries.push(duplicate.clone());
    let equal = NativeSnapshot::inspect(&executable, destination, &libraries).unwrap();
    assert_eq!(initial.identity, equal.identity);
    assert_eq!(initial.required(), equal.required());
    fs::write(&duplicate, b"conflicting library").unwrap();
    assert_eq!(
        NativeSnapshot::inspect(&executable, destination, &libraries)
            .err()
            .unwrap()
            .to_string(),
        "conflicting native runtime library basenames"
    );
    let collision = root.path().join("two/program");
    fs::write(&collision, b"not the executable").unwrap();
    assert!(NativeSnapshot::inspect(&executable, destination, &[collision]).is_err());
}

#[test]
fn captured_and_current_content_are_freshly_verified() {
    let (root, executable, libraries) = fixture();
    let destination = Path::new("bundle/program");
    let snapshot = NativeSnapshot::inspect(&executable, destination, &libraries).unwrap();
    let captured = root.path().join("captured");
    snapshot.capture(&captured).unwrap();
    snapshot.verify(&captured).unwrap();
    assert_eq!(
        fs::read(captured.join(destination)).unwrap(),
        fs::read(&executable).unwrap()
    );
    fs::write(captured.join(destination), b"alteration").unwrap();
    assert!(snapshot.verify(&captured).is_err());
    snapshot.capture(&captured).unwrap();
    snapshot.verify(&captured).unwrap();
    fs::write(&executable, b"alteration").unwrap();
    assert!(snapshot.verify(&captured).is_err());
    fs::write(&executable, b"executable").unwrap();
    snapshot.verify(&captured).unwrap();
    fs::remove_file(captured.join("bundle/libfirst.so")).unwrap();
    assert!(snapshot.verify(&captured).is_err());
}

#[test]
fn missing_required_sources_are_rejected() {
    let (_root, executable, libraries) = fixture();
    let destination = Path::new("program");
    fs::remove_file(&libraries[0]).unwrap();
    assert!(NativeSnapshot::inspect(&executable, destination, &libraries).is_err());
    fs::remove_file(&executable).unwrap();
    assert!(NativeSnapshot::inspect(&executable, destination, &[]).is_err());
}

#[cfg(unix)]
#[test]
fn sidecar_aliases_and_special_files_are_rejected() {
    let (root, executable, libraries) = fixture();
    let destination = Path::new("program");
    let pdb = executable.with_extension("pdb");
    fs::remove_file(&pdb).unwrap();
    std::os::unix::fs::symlink(&libraries[0], &pdb).unwrap();
    assert!(NativeSnapshot::inspect(&executable, destination, &libraries).is_err());
    assert_eq!(fs::read(&libraries[0]).unwrap(), b"first library");
    fs::remove_file(&pdb).unwrap();
    let socket = std::os::unix::net::UnixListener::bind(&pdb).unwrap();
    assert!(NativeSnapshot::inspect(&executable, destination, &libraries).is_err());
    drop(socket);
    fs::remove_file(&pdb).unwrap();
    std::os::unix::fs::symlink(&libraries[0], root.path().join("program.dSYM/link")).unwrap();
    assert!(NativeSnapshot::inspect(&executable, destination, &libraries).is_err());
}

#[test]
fn capture_preserves_equal_files_and_repairs_same_metadata_corruption() {
    let (root, executable, libraries) = fixture();
    let destination = Path::new("bundle/program");
    let snapshot = NativeSnapshot::inspect(&executable, destination, &libraries).unwrap();
    let captured = root.path().join("captured");
    snapshot.capture(&captured).unwrap();
    let path = captured.join(destination);
    let before = fs::metadata(&path).unwrap();
    snapshot.capture(&captured).unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().modified().unwrap(),
        before.modified().unwrap()
    );
    fs::write(&path, b"alteration").unwrap();
    fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(before.modified().unwrap()))
        .unwrap();
    assert_eq!(fs::metadata(&path).unwrap().len(), before.len());
    assert_eq!(
        fs::metadata(&path).unwrap().modified().unwrap(),
        before.modified().unwrap()
    );
    assert!(snapshot.verify(&captured).is_err());
    snapshot.capture(&captured).unwrap();
    snapshot.verify(&captured).unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"executable");
    fs::remove_file(&executable).unwrap();
    assert!(snapshot.capture(&captured).is_err());
}
