use sha2::{Digest as _, Sha256};
use sifr_analysis::SourceProvider;
use std::path::Path;
use std::time::UNIX_EPOCH;

pub(super) fn package_input_fingerprint(root: &Path, provider: &mut impl SourceProvider) -> String {
    let mut digest = Sha256::new();
    field(&mut digest, "schema", b"lsp-python-input-v2");
    let mut paths = vec![
        root.join("Cargo.toml"),
        root.join("Cargo.lock"),
        root.join("sifr.toml"),
        root.join(sifr_package::PYTHON_BINDINGS_FILE),
        root.join(sifr_package::PYTHON_CERTIFICATIONS_FILE),
    ];
    let interpreter = python_environment_selection(root, provider).map(|selection| {
        paths.extend(selection.pyproject);
        paths.extend(selection.lock);
        selection.interpreter
    });
    for path in paths {
        path_field(&mut digest, "path", &path);
        match std::fs::read(&path) {
            Ok(bytes) => field(&mut digest, "contents", &bytes),
            Err(error) => field(
                &mut digest,
                "read-error",
                format!("{:?}", error.kind()).as_bytes(),
            ),
        }
    }
    hash_python_bridge_inputs(root, &mut digest);
    hash_runnable_app_entries(root, &mut digest, provider);
    if let Some(interpreter) = interpreter {
        path_field(&mut digest, "interpreter", &interpreter);
        match std::fs::metadata(&interpreter) {
            Ok(metadata) => {
                field(
                    &mut digest,
                    "interpreter-len",
                    &metadata.len().to_le_bytes(),
                );
                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .map(|duration| duration.as_nanos())
                    .unwrap_or_default();
                field(&mut digest, "interpreter-modified", &modified.to_le_bytes());
            }
            Err(error) => field(
                &mut digest,
                "metadata-error",
                format!("{:?}", error.kind()).as_bytes(),
            ),
        }
    }
    lower_hex(&digest.finalize())
}

fn hash_runnable_app_entries(root: &Path, digest: &mut Sha256, provider: &mut impl SourceProvider) {
    let result = sifr_package::PackageSession::discover(
        sifr_package::PackageSessionOptions {
            current_dir: root.to_path_buf(),
            lock_mode: sifr_package::CargoLockMode::Frozen,
        },
        provider,
    )
    .and_then(|session| session.runnable_app_paths());
    match result {
        Ok(mut paths) => {
            paths.sort();
            for path in paths {
                path_field(digest, "runnable-app", &path);
            }
        }
        Err(error) => field(
            digest,
            "runnable-app-error",
            format!("{error:?}").as_bytes(),
        ),
    }
}

fn hash_python_bridge_inputs(root: &Path, digest: &mut Sha256) {
    let bridge_root = root.join(sifr_package::PYTHON_BRIDGE_ROOT);
    path_field(digest, "bridge-root", &bridge_root);
    let mut pending = vec![bridge_root];
    while let Some(directory) = pending.pop() {
        let entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                field(
                    digest,
                    "bridge-read-error",
                    format!("{:?}", error.kind()).as_bytes(),
                );
                continue;
            }
        };
        let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
        entries.sort_by_key(std::fs::DirEntry::path);
        for entry in entries {
            let path = entry.path();
            path_field(digest, "bridge-path", &path);
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => pending.push(path),
                Ok(kind) if kind.is_file() => match std::fs::read(&path) {
                    Ok(bytes) => field(digest, "bridge-contents", &bytes),
                    Err(error) => field(
                        digest,
                        "bridge-file-error",
                        format!("{:?}", error.kind()).as_bytes(),
                    ),
                },
                Ok(kind) => field(digest, "bridge-symlink", &[u8::from(kind.is_symlink())]),
                Err(error) => field(
                    digest,
                    "bridge-type-error",
                    format!("{:?}", error.kind()).as_bytes(),
                ),
            }
        }
    }
}

fn python_environment_selection(
    root: &Path,
    provider: &mut impl SourceProvider,
) -> Option<sifr_package::PythonEnvironmentSelection> {
    let session = sifr_package::PackageSession::discover(
        sifr_package::PackageSessionOptions {
            current_dir: root.to_path_buf(),
            lock_mode: sifr_package::CargoLockMode::Frozen,
        },
        provider,
    )
    .ok()?;
    let manifest = session.manifest?;
    sifr_package::select_root_python_environment(root, &manifest.python)
}

fn path_field(digest: &mut Sha256, name: &str, path: &Path) {
    field(digest, name, path.to_string_lossy().as_bytes());
}

fn field(digest: &mut Sha256, name: &str, value: &[u8]) {
    digest.update(name.len().to_le_bytes());
    digest.update(name.as_bytes());
    digest.update(value.len().to_le_bytes());
    digest.update(value);
}

fn lower_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}
