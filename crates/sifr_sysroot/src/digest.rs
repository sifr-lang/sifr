use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalDigestPolicy {
    pub include_extensions: Vec<String>,
    pub follow_symlinks: bool,
    pub normalize_line_endings: bool,
    pub include_executable_bit: bool,
}

impl Default for CanonicalDigestPolicy {
    fn default() -> Self {
        Self {
            include_extensions: vec![
                "toml".to_string(),
                "lock".to_string(),
                "rs".to_string(),
                "sifr".to_string(),
            ],
            follow_symlinks: false,
            normalize_line_endings: true,
            include_executable_bit: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalTreeDigestEntry {
    pub relative_path: String,
    pub executable: bool,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalTreeDigest {
    pub algorithm: &'static str,
    pub hex: String,
    pub entries: Vec<CanonicalTreeDigestEntry>,
}

pub fn canonical_sysroot_tree_digest(
    root: &Path,
    policy: &CanonicalDigestPolicy,
) -> std::io::Result<CanonicalTreeDigest> {
    let mut entries = Vec::new();
    collect_entries(root, root, policy, &mut entries)?;
    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    let mut hasher = Sha256::new();
    for entry in &entries {
        hasher.update(entry.relative_path.as_bytes());
        hasher.update([0]);
        if policy.include_executable_bit {
            hasher.update(if entry.executable { [1] } else { [0] });
        }
        hasher.update([0]);
        hasher.update(&entry.bytes);
        hasher.update([0]);
    }
    Ok(CanonicalTreeDigest {
        algorithm: "sha256",
        hex: format!("{:x}", hasher.finalize()),
        entries,
    })
}

fn collect_entries(
    root: &Path,
    path: &Path,
    policy: &CanonicalDigestPolicy,
    entries: &mut Vec<CanonicalTreeDigestEntry>,
) -> std::io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() {
        let mut children = fs::read_dir(path)?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        children.sort();
        for child in children {
            collect_entries(root, &child, policy, entries)?;
        }
        return Ok(());
    }
    if metadata.file_type().is_symlink() && !policy.follow_symlinks {
        return Ok(());
    }
    if !metadata.is_file() || !included(path, policy) {
        return Ok(());
    }
    let relative_path = normalized_relative_path(root, path)?;
    let mut bytes = fs::read(path)?;
    if policy.normalize_line_endings {
        bytes = normalize_lf(&bytes);
    }
    entries.push(CanonicalTreeDigestEntry {
        relative_path,
        executable: executable_bit(&metadata),
        bytes,
    });
    Ok(())
}

fn included(path: &Path, policy: &CanonicalDigestPolicy) -> bool {
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return false;
    };
    policy
        .include_extensions
        .iter()
        .any(|allowed| allowed == extension)
}

fn normalized_relative_path(root: &Path, path: &Path) -> std::io::Result<String> {
    let relative = path.strip_prefix(root).map_err(|error| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, error.to_string())
    })?;
    Ok(relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

fn normalize_lf(bytes: &[u8]) -> Vec<u8> {
    let text = String::from_utf8_lossy(bytes);
    text.replace("\r\n", "\n").replace('\r', "\n").into_bytes()
}

#[cfg(unix)]
fn executable_bit(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn executable_bit(_metadata: &fs::Metadata) -> bool {
    false
}
