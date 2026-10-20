use std::os::fd::AsRawFd;
use std::os::unix::fs::PermissionsExt;

#[test]
fn metadata_owner_is_not_inherited_by_unrelated_native_commands() {
    let root = tempfile::Builder::new()
        .permissions(std::fs::Permissions::from_mode(0o700))
        .tempdir()
        .unwrap();
    let context = crate::CompilerContext::for_test().with_cache_root(root.path().join("cache"));
    let provider = context.metadata_provider().unwrap();
    let prepared = &provider.metadata;
    let key = prepared.path.file_stem().unwrap();
    let lock = prepared.path.parent().unwrap().join(".locks").join(key);
    let descriptor = format!("/dev/fd/{}", prepared._lease.as_raw_fd());
    // Both supported Unix families expose /dev/fd. Compare inode identity so
    // incidental reuse of the numeric descriptor in the shell cannot fake a hit.
    assert!(std::path::Path::new("/dev/fd").is_dir());
    let result = std::process::Command::new("sh")
        .args(["-c", "test \"$1\" -ef \"$2\"", "metadata-child-lease"])
        .arg(descriptor)
        .arg(lock)
        .status()
        .unwrap();
    assert_eq!(
        result.code(),
        Some(1),
        "unrelated command inherited semantic metadata ownership"
    );
}
