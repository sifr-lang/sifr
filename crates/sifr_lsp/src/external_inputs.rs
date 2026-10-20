mod interpreter;
use interpreter::InterpreterSnapshot;

use sifr_analysis::{DiskSourceProvider, SourceProvider};
use sifr_compiler_services::python::python_environment_selection;
use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash as _, Hasher as _};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RootSnapshot {
    fingerprint: u64,
    stable: bool,
    generation: u64,
}

#[derive(Default)]
pub(crate) struct ExternalInputSnapshots {
    roots: BTreeMap<PathBuf, RootSnapshot>,
    revision: u64,
    interpreters: BTreeMap<PathBuf, InterpreterSnapshot>,
}

impl ExternalInputSnapshots {
    pub(crate) fn observe(&mut self, root: &Path) -> (u64, bool) {
        let interpreter = self.interpreters.entry(root.to_path_buf()).or_default();
        let (fingerprint, stable) =
            package_input_snapshot(root, &mut DiskSourceProvider::new(), interpreter);
        match self.roots.get_mut(root) {
            Some(snapshot) if stable && snapshot.stable && snapshot.fingerprint == fingerprint => {
                (snapshot.generation, false)
            }
            Some(snapshot) => {
                self.revision = self.revision.saturating_add(1);
                snapshot.fingerprint = fingerprint;
                snapshot.stable = stable;
                snapshot.generation = snapshot.generation.saturating_add(1);
                (snapshot.generation, true)
            }
            None => {
                self.revision = self.revision.saturating_add(1);
                self.roots.insert(
                    root.to_path_buf(),
                    RootSnapshot {
                        fingerprint,
                        stable,
                        generation: 1,
                    },
                );
                (1, false)
            }
        }
    }

    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn contains_root(&self, root: &Path) -> bool {
        self.roots.contains_key(root)
    }

    pub(crate) fn retire_root(&mut self, root: &Path) {
        self.interpreters.remove(root);
        if self.roots.remove(root).is_some() {
            self.revision = self.revision.saturating_add(1);
        }
    }

    pub(crate) fn owning_root(&self, path: &Path) -> Option<&Path> {
        self.roots
            .keys()
            .filter(|root| path.starts_with(root))
            .max_by_key(|root| root.components().count())
            .map(PathBuf::as_path)
    }

    #[cfg(test)]
    pub(crate) fn generation(&self, root: &Path) -> Option<u64> {
        self.roots.get(root).map(|snapshot| snapshot.generation)
    }

    pub(crate) fn state(&self, root: &Path) -> Option<(u64, u64, bool)> {
        self.roots
            .get(root)
            .map(|snapshot| (snapshot.generation, snapshot.fingerprint, snapshot.stable))
    }
}

#[cfg(test)]
pub(crate) fn package_input_fingerprint(root: &Path, provider: &mut impl SourceProvider) -> u64 {
    package_input_snapshot(root, provider, &mut InterpreterSnapshot::default()).0
}

fn package_input_snapshot(
    root: &Path,
    provider: &mut impl SourceProvider,
    interpreter_snapshot: &mut InterpreterSnapshot,
) -> (u64, bool) {
    let mut hasher = DefaultHasher::new();
    let mut stable = true;
    let mut paths = vec![
        root.join("Cargo.toml"),
        root.join("Cargo.lock"),
        root.join("sifr.toml"),
        root.join(sifr_package::PYTHON_BINDINGS_FILE),
        root.join(sifr_package::PYTHON_CERTIFICATIONS_FILE),
    ];
    for ancestor in root.ancestors() {
        for name in [
            "Cargo.toml",
            "Cargo.lock",
            ".cargo/config.toml",
            ".cargo/config",
            "rust-toolchain.toml",
            "rust-toolchain",
            "pyproject.toml",
            "uv.lock",
        ] {
            paths.push(ancestor.join(name));
        }
    }
    for relative in sifr_package::required_python_binding_archive_entries(root) {
        paths.push(root.join(relative));
    }
    for relative in sifr_package::required_python_certification_archive_entries(root) {
        paths.push(root.join(relative));
    }
    let interpreter = python_environment_selection(root, provider).map(|selection| {
        paths.extend(selection.pyproject);
        paths.extend(selection.lock);
        paths.push(selection.venv_root.join("pyvenv.cfg"));
        selection.interpreter
    });
    paths.sort();
    paths.dedup();
    for path in paths {
        hash_path(&path, &mut hasher, &mut stable, true);
    }
    hash_python_bridge_inputs(root, &mut hasher, &mut stable);
    hash_runnable_app_entries(root, &mut hasher, provider);
    if let Some(interpreter) = interpreter {
        interpreter_snapshot.hash(&interpreter, &mut hasher, &mut stable);
    }
    (hasher.finish(), stable)
}

fn hash_path(path: &Path, hasher: &mut DefaultHasher, stable: &mut bool, expected_file: bool) {
    path.hash(hasher);
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            metadata.file_type().is_file().hash(hasher);
            metadata.file_type().is_dir().hash(hasher);
            metadata.file_type().is_symlink().hash(hasher);
            if metadata.file_type().is_symlink() {
                let target = std::fs::read_link(path).map_err(|error| error.kind());
                if target.is_err() {
                    *stable = false;
                }
                target.hash(hasher);
            }
            if metadata.is_dir() {
                if expected_file {
                    *stable = false;
                }
                return;
            }
            match std::fs::read(path) {
                Ok(bytes) => bytes.hash(hasher),
                Err(error) => {
                    if error.kind() != std::io::ErrorKind::NotFound {
                        *stable = false;
                    }
                    error.kind().hash(hasher);
                }
            }
        }
        Err(error) => {
            if error.kind() != std::io::ErrorKind::NotFound {
                *stable = false;
            }
            error.kind().hash(hasher);
        }
    }
}

fn hash_runnable_app_entries(
    root: &Path,
    hasher: &mut DefaultHasher,
    provider: &mut impl SourceProvider,
) {
    "runnable-app-entries".hash(hasher);
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
            paths.hash(hasher);
        }
        Err(error) => format!("{error:?}").hash(hasher),
    }
}

fn hash_python_bridge_inputs(root: &Path, hasher: &mut DefaultHasher, stable: &mut bool) {
    let bridge_root = root.join(sifr_package::PYTHON_BRIDGE_ROOT);
    bridge_root.hash(hasher);
    let mut pending = vec![bridge_root];
    while let Some(directory) = pending.pop() {
        let entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                if error.kind() != std::io::ErrorKind::NotFound {
                    *stable = false;
                }
                error.kind().hash(hasher);
                continue;
            }
        };
        let mut entries = entries.collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.as_ref().map(std::fs::DirEntry::path).ok());
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    *stable = false;
                    error.kind().hash(hasher);
                    continue;
                }
            };
            let path = entry.path();
            hash_path(&path, hasher, stable, false);
            if let Ok(kind) = entry.file_type() {
                if kind.is_dir() {
                    pending.push(path);
                }
            }
        }
    }
}
