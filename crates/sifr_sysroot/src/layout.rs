use crate::error::{SysrootError, SysrootErrorKind};
use crate::manifest::{read_sysroot_manifest, SysrootManifest};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSysroot {
    pub root: PathBuf,
    pub manifest: SysrootManifest,
    pub paths: SysrootPaths,
}

impl ResolvedSysroot {
    pub(crate) fn from_root(root: PathBuf, binary_path: &Path) -> Result<Self, SysrootError> {
        let manifest = read_sysroot_manifest(&root, binary_path)?;
        let paths = SysrootPaths::from_root(&root);
        paths.validate(binary_path, &root)?;
        Ok(Self {
            root,
            manifest,
            paths,
        })
    }

    #[must_use]
    pub fn toolchain_id(&self) -> String {
        self.manifest.toolchain_id()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SysrootPaths {
    pub manifest: PathBuf,
    pub stdlib_root: PathBuf,
    pub runtime_crate: PathBuf,
    pub runtime_crate_manifest: PathBuf,
    pub cargo_manifest: PathBuf,
    pub cargo_lock: PathBuf,
    pub cargo_config: PathBuf,
    pub vendor: PathBuf,
}

impl SysrootPaths {
    #[must_use]
    pub fn from_root(root: &Path) -> Self {
        let runtime_crate = root.join("crates").join("sifr_runtime");
        Self {
            manifest: root.join("sysroot.toml"),
            stdlib_root: root.join("lib").join("sifr"),
            runtime_crate_manifest: runtime_crate.join("Cargo.toml"),
            runtime_crate,
            cargo_manifest: root.join("Cargo.toml"),
            cargo_lock: root.join("Cargo.lock"),
            cargo_config: root.join(".cargo").join("config.toml"),
            vendor: root.join("vendor"),
        }
    }

    fn validate(&self, binary_path: &Path, root: &Path) -> Result<(), SysrootError> {
        require_file(binary_path, root, &self.cargo_manifest, "Cargo.toml")?;
        require_file(binary_path, root, &self.cargo_lock, "Cargo.lock")?;
        require_file(binary_path, root, &self.cargo_config, ".cargo/config.toml")?;
        require_dir(binary_path, root, &self.stdlib_root, "lib/sifr")?;
        require_dir(binary_path, root, &self.vendor, "vendor")?;
        require_file(
            binary_path,
            root,
            &self.runtime_crate_manifest,
            "crates/sifr_runtime/Cargo.toml",
        )?;
        validate_workspace_manifest(binary_path, root, &self.cargo_manifest)
    }
}

fn require_file(
    binary_path: &Path,
    root: &Path,
    path: &Path,
    label: &str,
) -> Result<(), SysrootError> {
    if path.is_file() {
        Ok(())
    } else {
        Err(missing_asset(
            binary_path,
            root,
            path,
            format!("Sifr sysroot is missing {label}"),
        ))
    }
}

fn require_dir(
    binary_path: &Path,
    root: &Path,
    path: &Path,
    label: &str,
) -> Result<(), SysrootError> {
    if path.is_dir() {
        Ok(())
    } else {
        Err(missing_asset(
            binary_path,
            root,
            path,
            format!("Sifr sysroot is missing {label}"),
        ))
    }
}

fn validate_workspace_manifest(
    binary_path: &Path,
    root: &Path,
    manifest: &Path,
) -> Result<(), SysrootError> {
    let input = std::fs::read_to_string(manifest).map_err(|error| {
        SysrootError::new(
            SysrootErrorKind::InvalidWorkspace,
            binary_path.to_path_buf(),
            root.to_path_buf(),
            Some(manifest.to_path_buf()),
            format!("Sifr sysroot workspace manifest could not be read: {error}"),
        )
    })?;
    let value = toml::from_str::<toml::Value>(&input).map_err(|error| {
        SysrootError::new(
            SysrootErrorKind::InvalidWorkspace,
            binary_path.to_path_buf(),
            root.to_path_buf(),
            Some(manifest.to_path_buf()),
            format!("Sifr sysroot workspace manifest is not valid TOML: {error}"),
        )
    })?;
    let workspace = value.get("workspace").and_then(toml::Value::as_table);
    let Some(workspace) = workspace else {
        return Err(SysrootError::new(
            SysrootErrorKind::InvalidWorkspace,
            binary_path.to_path_buf(),
            root.to_path_buf(),
            Some(manifest.to_path_buf()),
            "Sifr sysroot Cargo.toml must define a [workspace] table",
        ));
    };
    if !workspace.contains_key("members") {
        return Err(SysrootError::new(
            SysrootErrorKind::InvalidWorkspace,
            binary_path.to_path_buf(),
            root.to_path_buf(),
            Some(manifest.to_path_buf()),
            "Sifr sysroot Cargo.toml must define workspace members",
        ));
    }
    if workspace.get("resolver").and_then(toml::Value::as_str) != Some("2") {
        return Err(SysrootError::new(
            SysrootErrorKind::InvalidWorkspace,
            binary_path.to_path_buf(),
            root.to_path_buf(),
            Some(manifest.to_path_buf()),
            "Sifr sysroot Cargo.toml must use workspace resolver \"2\"",
        ));
    }
    Ok(())
}

fn missing_asset(binary_path: &Path, root: &Path, path: &Path, message: String) -> SysrootError {
    SysrootError::new(
        SysrootErrorKind::MissingAsset,
        binary_path.to_path_buf(),
        root.to_path_buf(),
        Some(path.to_path_buf()),
        message,
    )
}
