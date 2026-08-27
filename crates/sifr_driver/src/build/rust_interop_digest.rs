use sha2::{Digest as _, Sha256};
use std::fs;
use std::path::Path;

pub(super) fn digest_path(path: &Path) -> String {
    let mut entries = Vec::new();
    collect_digest_entries(path, path, &mut entries);
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    let mut digest_input = Vec::new();
    for (relative, file_bytes) in entries {
        push_cache_bytes(&mut digest_input, &relative);
        digest_input.extend_from_slice(&file_bytes);
        digest_input.push(0);
    }
    sha256_hex(&digest_input)
}

pub(super) fn digest_file(path: &Path) -> Option<String> {
    fs::read(path).ok().map(|bytes| sha256_hex(&bytes))
}

pub(super) fn relative_path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map_or_else(|_| normalized_path_string(path), normalized_path_string)
}

pub(super) fn normalized_path_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}

pub(super) fn cache_identity(domain: &str, input: Vec<u8>) -> CacheIdentity {
    let mut material = Vec::new();
    push_cache_bytes(&mut material, domain);
    material.extend(input);
    CacheIdentity {
        digest: sha256_hex(&material),
        material,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(super) struct CacheIdentity {
    pub(super) digest: String,
    pub(super) material: Vec<u8>,
}

impl CacheIdentity {
    pub(super) fn matches(&self, other: &Self) -> bool {
        self.digest == other.digest && self.material == other.material
    }
}

fn collect_digest_entries(root: &Path, path: &Path, entries: &mut Vec<(String, Vec<u8>)>) {
    if path.is_file() {
        if let Ok(bytes) = fs::read(path) {
            entries.push((relative_path_string(root, path), bytes));
        }
        return;
    }
    let Ok(read_dir) = fs::read_dir(path) else {
        return;
    };
    for entry in read_dir.flatten() {
        collect_digest_entries(root, &entry.path(), entries);
    }
}

pub(super) fn push_cache_bytes(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(value.as_bytes());
    out.push(0xff);
}
