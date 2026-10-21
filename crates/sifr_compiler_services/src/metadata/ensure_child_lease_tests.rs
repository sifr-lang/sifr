use std::os::unix::fs::MetadataExt;
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
    // The shared reader owns its lease privately. Observe the real open lock
    // descriptors through the OS instead of exposing that implementation field.
    let lock_identity = std::fs::metadata(&lock).unwrap();
    let descriptors = std::fs::read_dir("/dev/fd")
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>()
        .into_iter()
        .filter(|path| {
            std::fs::metadata(path).is_ok_and(|metadata| {
                metadata.dev() == lock_identity.dev() && metadata.ino() == lock_identity.ino()
            })
        })
        .collect::<Vec<_>>();
    assert!(
        !descriptors.is_empty(),
        "prepared metadata did not retain its lease"
    );
    for descriptor in descriptors {
        let result = std::process::Command::new("sh")
            .args(["-c", "test \"$1\" -ef \"$2\"", "metadata-child-lease"])
            .arg(descriptor)
            .arg(&lock)
            .status()
            .unwrap();
        assert_eq!(
            result.code(),
            Some(1),
            "unrelated command inherited semantic metadata ownership"
        );
    }
}
