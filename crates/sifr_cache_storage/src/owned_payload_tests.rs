//! Opened-object authority and nonblocking rejection of non-file payloads.
use crate::read_owned_payload;
use std::{
    fs::{self, Permissions},
    io::Read,
    os::unix::fs::{MetadataExt, PermissionsExt, symlink},
};

fn private_root() -> tempfile::TempDir {
    tempfile::Builder::new()
        .permissions(Permissions::from_mode(0o700))
        .tempdir()
        .unwrap()
}

fn private_payload(path: &std::path::Path, bytes: &[u8]) {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
}

#[test]
fn payload_handle_binds_bytes_and_metadata_across_path_replacement() {
    let root = private_root();
    let root = root.path().canonicalize().unwrap();
    let path = root.join("payload");
    private_payload(&path, b"original");
    let (mut file, metadata) = read_owned_payload(&path).unwrap();
    fs::rename(&path, root.join("retained")).unwrap();
    private_payload(&path, b"replacement");
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"original");
    assert_eq!(metadata.len(), bytes.len() as u64);
    assert_eq!(metadata.ino(), file.metadata().unwrap().ino());
    assert_ne!(metadata.ino(), fs::metadata(&path).unwrap().ino());
    let (mut current, current_metadata) = read_owned_payload(&path).unwrap();
    bytes.clear();
    current.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"replacement");
    assert_eq!(current_metadata.len(), bytes.len() as u64);
}

#[test]
fn payload_rejects_unsafe_parent_file_symlinks_and_directories() {
    let root = private_root();
    let root = root.path().canonicalize().unwrap();
    let parent = root.join("private");
    crate::directory(&parent).unwrap();
    let path = parent.join("payload");
    private_payload(&path, b"private");
    read_owned_payload(&path).unwrap();
    fs::set_permissions(&path, Permissions::from_mode(0o666)).unwrap();
    assert!(read_owned_payload(&path).is_err());
    fs::set_permissions(&path, Permissions::from_mode(0o600)).unwrap();
    fs::set_permissions(&parent, Permissions::from_mode(0o777)).unwrap();
    assert!(read_owned_payload(&path).is_err());
    fs::set_permissions(&parent, Permissions::from_mode(0o700)).unwrap();
    symlink(&path, parent.join("alias")).unwrap();
    assert!(read_owned_payload(&parent.join("alias")).is_err());
    symlink(&parent, root.join("alias")).unwrap();
    assert!(read_owned_payload(&root.join("alias/payload")).is_err());
    assert!(read_owned_payload(&parent).is_err());
}

#[test]
#[allow(unsafe_code)]
fn fifo_payload_is_rejected_without_waiting_for_a_writer() {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    let root = private_root();
    let path = root.path().canonicalize().unwrap().join("fifo");
    let name = CString::new(path.as_os_str().as_bytes()).unwrap();
    // SAFETY: name is a live NUL-terminated path in this test's private directory.
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
    assert!(read_owned_payload(&path).is_err());
}
