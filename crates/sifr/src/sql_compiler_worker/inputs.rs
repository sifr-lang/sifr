//! Bind the worker to the canonical frozen graph and the inputs it may read.
use sifr_package::{PackageGraphSnapshot, SifrPackageId};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(PartialEq, Eq)]
pub(super) struct Inputs {
    pub snapshot: Option<(PackageGraphSnapshot, SifrPackageId)>,
    files: BTreeMap<PathBuf, (PathBuf, String)>,
    pub profile_names: BTreeSet<String>,
}

impl Inputs {
    pub fn capture(root: &Path) -> Result<Self, String> {
        let snapshot = sifr_semantic_services::sql_editor::load_sql_editor_snapshot(root, root)
            .map_err(super::messages)?;
        let mut paths = BTreeSet::new();
        let mut profile_names = BTreeSet::new();
        for name in ["Cargo.toml", "Cargo.lock", "sifr.toml"] {
            let path = root.join(name);
            if path.try_exists().map_err(|error| error.to_string())? {
                paths.insert(path);
            }
        }
        if let Some((snapshot, owner)) = &snapshot {
            paths.insert(snapshot.metadata.workspace_root.join("Cargo.toml"));
            paths.insert(snapshot.metadata.workspace_root.join("Cargo.lock"));
            for package in snapshot.metadata.packages.values() {
                paths.insert(package.manifest_path.clone());
            }
            for package in snapshot.graph.packages.values() {
                paths.insert(package.sifr_manifest.clone());
            }
            let profiles = sifr_package::resolve_sql_profiles(&snapshot.graph, owner)
                .map_err(package_messages)?;
            let owner_root = &snapshot
                .graph
                .packages
                .get(owner)
                .ok_or("SQL compiler profile owner disappeared")?
                .package_root;
            for (name, profile) in profiles {
                profile_names.insert(name);
                for path in profile.schema_sources {
                    admit(&path, owner_root)?;
                    paths.insert(path);
                }
            }
            let requirements = sifr_package::resolve_sql_requirements(&snapshot.graph, owner)
                .map_err(package_messages)?;
            for requirement in requirements.values() {
                let package_root = &snapshot
                    .graph
                    .packages
                    .get(&requirement.owner_package_id)
                    .ok_or("SQL compiler requirement owner disappeared")?
                    .package_root;
                for provider in requirement.providers.values() {
                    admit(&provider.source_path, package_root)?;
                    paths.insert(provider.source_path.clone());
                }
            }
            for component in sifr_package::compiler_component_registrations(&snapshot.graph)
                .map_err(|error| error.to_string())?
                .values()
            {
                admit(&component.artifact_path, &component.package_root)?;
                paths.insert(component.artifact_path.clone());
            }
        }
        let files = paths
            .into_iter()
            .map(|path| pin(&path).map(|value| (path, value)))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            snapshot,
            files,
            profile_names,
        })
    }

    pub fn verify(&self, root: &Path) -> Result<(), String> {
        if Self::capture(root)? != *self {
            return Err(
                "SQL compiler project, lock or schema inputs changed during the session".into(),
            );
        }
        Ok(())
    }
}

fn admit(path: &Path, root: &Path) -> Result<(), String> {
    let path = path.canonicalize().map_err(|error| error.to_string())?;
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    if !path.starts_with(root) {
        return Err("SQL compiler input escapes its admitted package".into());
    }
    Ok(())
}

fn pin(path: &Path) -> Result<(PathBuf, String), String> {
    let canonical = path.canonicalize().map_err(|error| error.to_string())?;
    if !canonical
        .metadata()
        .map_err(|error| error.to_string())?
        .is_file()
    {
        return Err("SQL compiler input must be a regular file".into());
    }
    let hash = sifr_sysroot::sha256_file(&canonical).map_err(|error| error.to_string())?;
    Ok((canonical, hash))
}

fn package_messages(failures: Vec<sifr_package::PackageDiagnostic>) -> String {
    failures
        .into_iter()
        .map(|failure| {
            sifr_semantic_services::package_diagnostics::render_package_diagnostic(failure).message
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Sources are admitted by bytes from configured migration directories. Requests
/// cannot supply a filename or another root. Native tools still own baseline
/// loading and filename/declaration consistency checks.
pub(super) fn migration_sources(
    root: &Path,
    profiles: &BTreeSet<String>,
) -> Result<BTreeMap<PathBuf, String>, String> {
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    let mut files = BTreeMap::new();
    for profile in profiles {
        // The graph validator owns profile names; keep the filesystem boundary
        // explicit even when this helper is called independently.
        let mut components = Path::new(profile).components();
        if !matches!(components.next(), Some(std::path::Component::Normal(_)))
            || components.next().is_some()
        {
            return Err("SQL compiler migration profile must be one path segment".into());
        }
        let directory = root.join("migrations").join(profile);
        if !directory.try_exists().map_err(|error| error.to_string())? {
            continue;
        }
        let canonical = directory
            .canonicalize()
            .map_err(|error| error.to_string())?;
        if !canonical.starts_with(&root)
            || !directory
                .symlink_metadata()
                .map_err(|error| error.to_string())?
                .is_dir()
        {
            return Err("SQL compiler migration directory escapes its admitted project".into());
        }
        for entry in std::fs::read_dir(&directory).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("sifr") {
                continue;
            }
            if !entry
                .file_type()
                .map_err(|error| error.to_string())?
                .is_file()
            {
                return Err("SQL compiler migration input must be a regular file".into());
            }
            let canonical = path.canonicalize().map_err(|error| error.to_string())?;
            if !canonical.starts_with(&root) {
                return Err("SQL compiler migration input escapes its admitted project".into());
            }
            files.insert(
                path,
                std::fs::read_to_string(canonical).map_err(|error| error.to_string())?,
            );
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn input_snapshot_rejects_added_or_changed_lock_but_accepts_identical_bytes() {
        let root = tempfile::tempdir().unwrap();
        let absent = Inputs::capture(root.path()).unwrap();
        let lock = root.path().join("Cargo.lock");
        std::fs::write(&lock, "original").unwrap();
        assert!(absent.verify(root.path()).is_err());
        let before = Inputs::capture(root.path()).unwrap();
        std::fs::write(&lock, "changed").unwrap();
        assert!(before.verify(root.path()).is_err());
        std::fs::write(&lock, "original").unwrap();
        before.verify(root.path()).unwrap();
    }

    #[test]
    fn package_admission_rejects_outside_paths_before_reading_content() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        assert!(admit(outside.path(), root.path()).is_err());
    }

    #[test]
    fn migration_admission_is_limited_to_configured_files_and_detects_edits() {
        let root = tempfile::tempdir().unwrap();
        let admitted = root.path().join("migrations/app");
        let other = root.path().join("migrations/other");
        std::fs::create_dir_all(&admitted).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        std::fs::write(admitted.join("a.sifr"), "first").unwrap();
        std::fs::write(admitted.join("ignored.txt"), "not source").unwrap();
        std::fs::write(other.join("b.sifr"), "not admitted").unwrap();
        let profiles = BTreeSet::from(["app".into()]);
        let before = migration_sources(root.path(), &profiles).unwrap();
        assert_eq!(before.len(), 1);
        assert!(before.values().any(|value| value == "first"));
        std::fs::write(admitted.join("a.sifr"), "changed").unwrap();
        assert_ne!(before, migration_sources(root.path(), &profiles).unwrap());
        assert!(migration_sources(root.path(), &BTreeSet::from(["../outside".into()])).is_err());
    }

    #[test]
    #[cfg(unix)]
    fn migration_admission_rejects_file_and_parent_symlink_escapes() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::create_dir(outside.path().join("app")).unwrap();
        std::fs::write(outside.path().join("app/a.sifr"), "outside").unwrap();
        symlink(outside.path(), root.path().join("migrations")).unwrap();
        let profiles = BTreeSet::from(["app".into()]);
        assert!(migration_sources(root.path(), &profiles).is_err());
        std::fs::remove_file(root.path().join("migrations")).unwrap();
        std::fs::create_dir_all(root.path().join("migrations/app")).unwrap();
        symlink(
            outside.path().join("app/a.sifr"),
            root.path().join("migrations/app/a.sifr"),
        )
        .unwrap();
        assert!(migration_sources(root.path(), &profiles).is_err());
    }
}
