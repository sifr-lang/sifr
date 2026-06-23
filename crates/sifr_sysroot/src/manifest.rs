use crate::error::{SysrootError, SysrootErrorKind};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const SUPPORTED_SYSROOT_SCHEMA_VERSION: u64 = 1;
pub const COMPILER_SIFR_VERSION: &str = env!("SIFR_SYSROOT_COMPILER_VERSION");

pub const SYSROOT_MANIFEST_FIELDS: &[&str] = &[
    "schema-version",
    "sifr-version",
    "target-triple",
    "built-by-compiler-commit",
    "sysroot-content-sha256",
    "cargo-lock-sha256",
];

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct SysrootManifest {
    pub schema_version: u64,
    pub sifr_version: String,
    pub target_triple: String,
    pub built_by_compiler_commit: String,
    pub sysroot_content_sha256: String,
    pub cargo_lock_sha256: String,
}

impl SysrootManifest {
    #[must_use]
    pub fn toolchain_id(&self) -> String {
        format!("{}-{}", self.sifr_version, self.target_triple)
    }
}

pub(crate) fn read_sysroot_manifest(
    root: &Path,
    binary_path: &Path,
) -> Result<SysrootManifest, SysrootError> {
    let path = root.join("sysroot.toml");
    let input = std::fs::read_to_string(&path).map_err(|error| {
        SysrootError::new(
            SysrootErrorKind::MissingManifest,
            binary_path.to_path_buf(),
            root.to_path_buf(),
            Some(path.clone()),
            format!("Sifr sysroot manifest could not be read: {error}"),
        )
    })?;
    parse_sysroot_manifest(&input).map_err(|mut error| {
        error.binary_path = binary_path.to_path_buf();
        error.attempted_sysroot = root.to_path_buf();
        error.asset_path = Some(path);
        error
    })
}

pub fn parse_sysroot_manifest(input: &str) -> Result<SysrootManifest, SysrootError> {
    let value = toml::from_str::<toml::Value>(input).map_err(|error| {
        schema_error(
            SysrootErrorKind::MalformedManifest,
            format!("Sifr sysroot manifest is not valid TOML: {error}"),
        )
    })?;
    let table = value.as_table().ok_or_else(|| {
        schema_error(
            SysrootErrorKind::MalformedManifest,
            "Sifr sysroot manifest must be a TOML table",
        )
    })?;
    reject_unknown_fields(table.keys().map(String::as_str))?;
    let manifest: SysrootManifest = value.try_into().map_err(|error| {
        schema_error(
            SysrootErrorKind::MalformedManifest,
            format!("Sifr sysroot manifest fields are malformed: {error}"),
        )
    })?;
    if manifest.schema_version != SUPPORTED_SYSROOT_SCHEMA_VERSION {
        return Err(schema_error(
            SysrootErrorKind::UnsupportedSchemaVersion,
            format!(
                "Sifr sysroot schema-version {} is unsupported; supported schema-version is {}",
                manifest.schema_version, SUPPORTED_SYSROOT_SCHEMA_VERSION
            ),
        ));
    }
    validate_sifr_version(&manifest.sifr_version)?;
    validate_sha256("sysroot-content-sha256", &manifest.sysroot_content_sha256)?;
    validate_sha256("cargo-lock-sha256", &manifest.cargo_lock_sha256)?;
    Ok(manifest)
}

fn validate_sifr_version(value: &str) -> Result<(), SysrootError> {
    if value == COMPILER_SIFR_VERSION || value.strip_suffix("-dev") == Some(COMPILER_SIFR_VERSION) {
        return Ok(());
    }
    Err(schema_error(
        SysrootErrorKind::VersionMismatch,
        format!(
            "Sifr sysroot version {value} does not match compiler version {COMPILER_SIFR_VERSION}"
        ),
    ))
}

fn reject_unknown_fields<'a>(fields: impl Iterator<Item = &'a str>) -> Result<(), SysrootError> {
    let allowed = SYSROOT_MANIFEST_FIELDS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    for field in fields {
        if allowed.contains(field) || optional_field_allowed(field) {
            continue;
        }
        return Err(schema_error(
            SysrootErrorKind::UnknownManifestField,
            format!("Sifr sysroot manifest field `{field}` is not part of schema-version 1"),
        ));
    }
    Ok(())
}

fn optional_field_allowed(field: &str) -> bool {
    field
        .strip_prefix("optional-")
        .is_some_and(|tail| !tail.is_empty())
}

fn validate_sha256(field: &str, value: &str) -> Result<(), SysrootError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }
    Err(schema_error(
        SysrootErrorKind::MalformedManifest,
        format!("Sifr sysroot manifest field `{field}` must be a lowercase SHA-256 hex digest"),
    ))
}

fn schema_error(kind: SysrootErrorKind, message: impl Into<String>) -> SysrootError {
    SysrootError::new(kind, PathBuf::new(), PathBuf::new(), None, message)
}
