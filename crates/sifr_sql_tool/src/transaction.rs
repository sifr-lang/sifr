use crate::lifecycle::error;
use crate::{SchemaBuildArtifacts, SchemaLifecycleError, SchemaLifecycleErrorKind};
use std::fs;
use std::path::{Component, Path, PathBuf};
use tempfile::Builder;

pub fn write_artifacts_atomically(
    output_directory: &Path,
    artifacts: &SchemaBuildArtifacts,
) -> Result<(), SchemaLifecycleError> {
    validate_output_directory(output_directory)?;
    let parent = output_directory.parent().ok_or_else(|| {
        error(
            SchemaLifecycleErrorKind::FileSystem,
            "schema artifact directory needs a parent directory",
        )
    })?;
    reject_project_symlinks(output_directory)?;
    fs::create_dir_all(parent).map_err(file_error("create schema artifact parent"))?;
    reject_project_symlinks(output_directory)?;

    let staging = Builder::new()
        .prefix(".sifr-schema-stage-")
        .tempdir_in(parent)
        .map_err(file_error("create schema artifact staging directory"))?;
    for (relative, bytes) in artifacts.files() {
        validate_relative_path(relative)?;
        let destination = staging.path().join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(file_error("create staged artifact directory"))?;
        }
        fs::write(&destination, bytes).map_err(file_error("write staged schema artifact"))?;
    }

    let backup = backup_path(output_directory);
    if backup.exists() {
        return Err(error(
            SchemaLifecycleErrorKind::FileSystem,
            format!(
                "stale schema artifact backup exists at '{}'",
                backup.display()
            ),
        ));
    }
    let had_existing = output_directory.exists();
    if had_existing {
        if output_directory
            .symlink_metadata()
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return Err(error(
                SchemaLifecycleErrorKind::FileSystem,
                "schema artifact directory cannot be a symbolic link",
            ));
        }
        fs::rename(output_directory, &backup)
            .map_err(file_error("move previous schema artifacts to backup"))?;
    }
    let staged_path = staging.keep();
    if let Err(failure) = fs::rename(&staged_path, output_directory) {
        let rollback = if had_existing {
            fs::rename(&backup, output_directory).err()
        } else {
            None
        };
        let _ = fs::remove_dir_all(&staged_path);
        let suffix = rollback.map_or_else(String::new, |rollback| {
            format!("; rollback also failed: {rollback}")
        });
        return Err(error(
            SchemaLifecycleErrorKind::FileSystem,
            format!("replace schema artifacts: {failure}{suffix}"),
        ));
    }
    if had_existing {
        fs::remove_dir_all(&backup)
            .map_err(file_error("remove previous schema artifact backup"))?;
    }
    Ok(())
}

fn validate_output_directory(path: &Path) -> Result<(), SchemaLifecycleError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(error(
            SchemaLifecycleErrorKind::FileSystem,
            "schema artifact directory must be an absolute normalized path",
        ));
    }
    Ok(())
}

fn validate_relative_path(path: &str) -> Result<(), SchemaLifecycleError> {
    let path = Path::new(path);
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(error(
            SchemaLifecycleErrorKind::FileSystem,
            "schema artifact names must be normalized relative paths",
        ));
    }
    Ok(())
}

fn reject_project_symlinks(path: &Path) -> Result<(), SchemaLifecycleError> {
    for current in path.ancestors().take(4) {
        if current
            .symlink_metadata()
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return Err(error(
                SchemaLifecycleErrorKind::FileSystem,
                format!(
                    "schema artifact path crosses symbolic link '{}'",
                    current.display()
                ),
            ));
        }
    }
    Ok(())
}

fn backup_path(output_directory: &Path) -> PathBuf {
    let name = output_directory
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("schema");
    output_directory.with_file_name(format!(".{name}.sifr-backup"))
}

fn file_error(operation: &'static str) -> impl FnOnce(std::io::Error) -> SchemaLifecycleError {
    move |failure| {
        error(
            SchemaLifecycleErrorKind::FileSystem,
            format!("{operation}: {failure}"),
        )
    }
}
