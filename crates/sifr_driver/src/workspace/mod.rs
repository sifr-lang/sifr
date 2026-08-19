use crate::diagnostics::RenderedDiagnostic;
use sifr_diagnostics::DiagnosticCode;
use sifr_frontend::SourceProvider;
use std::path::{Component, Path, PathBuf};

const MANIFEST_FILE: &str = "sifr.toml";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceRoot {
    pub dir: PathBuf,
    pub config: SifrWorkspaceConfig,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SifrWorkspaceConfig {
    pub source_root: PathBuf,
    pub package_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SifrManifest {
    package_name: Option<String>,
    source_root: String,
}

pub fn find_workspace_root(
    entry: &Path,
    provider: &mut dyn SourceProvider,
) -> Result<Option<WorkspaceRoot>, Vec<RenderedDiagnostic>> {
    let Some(mut current) = entry.parent().map(Path::to_path_buf) else {
        return Ok(None);
    };

    loop {
        let manifest_path = current.join(MANIFEST_FILE);
        if provider.is_file(&manifest_path) {
            let config = parse_workspace_config(&current, &manifest_path, provider)?;
            let dir = if current.as_os_str().is_empty() {
                PathBuf::from(".")
            } else {
                current
            };
            return Ok(Some(WorkspaceRoot { dir, config }));
        }
        if !current.pop() {
            return Ok(None);
        }
    }
}

fn parse_workspace_config(
    workspace_root: &Path,
    manifest_path: &Path,
    provider: &mut dyn SourceProvider,
) -> Result<SifrWorkspaceConfig, Vec<RenderedDiagnostic>> {
    let source = provider
        .read_file(manifest_path)
        .map_err(|error| vec![parse_manifest_error(manifest_path, error)])?;
    let manifest = parse_manifest(manifest_path, source.as_str())?;
    let source_root = validate_source_root(workspace_root, &manifest.source_root, provider)?;

    Ok(SifrWorkspaceConfig {
        source_root,
        package_name: manifest.package_name,
    })
}

fn parse_manifest(
    manifest_path: &Path,
    source: &str,
) -> Result<SifrManifest, Vec<RenderedDiagnostic>> {
    let value = source
        .parse::<toml::Table>()
        .map(toml::Value::Table)
        .map_err(|error| vec![parse_manifest_error(manifest_path, error)])?;

    let package_name = value
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|package| package.get("name"))
        .map(|name| {
            name.as_str().map(str::to_string).ok_or_else(|| {
                parse_manifest_schema_error(manifest_path, "package.name must be a string")
            })
        })
        .transpose()?;

    let source = value.get("source").and_then(toml::Value::as_table);
    if source.is_some_and(|source| source.keys().any(|field| field != "root")) {
        return Err(parse_manifest_schema_error(
            manifest_path,
            "source contains an unsupported field",
        ));
    }
    let source_root = source
        .and_then(|source| source.get("root"))
        .map(|root| parse_source_root(manifest_path, root))
        .transpose()?
        .unwrap_or_else(|| "src".to_string());

    Ok(SifrManifest {
        package_name,
        source_root,
    })
}

fn parse_source_root(
    manifest_path: &Path,
    root: &toml::Value,
) -> Result<String, Vec<RenderedDiagnostic>> {
    let Some(root) = root.as_str() else {
        return Err(parse_manifest_schema_error(
            manifest_path,
            "source.root must be a string",
        ));
    };
    Ok(root.to_string())
}

fn validate_source_root(
    workspace_root: &Path,
    source_root: &str,
    provider: &mut dyn SourceProvider,
) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    let raw = Path::new(source_root);
    if source_root.is_empty() || raw.is_absolute() {
        return Err(vec![source_root_error(
            source_root,
            SourceRootErrorKind::Invalid,
        )]);
    }

    let mut normalized = PathBuf::new();
    for component in raw.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                return Err(vec![source_root_error(
                    source_root,
                    SourceRootErrorKind::Escapes,
                )]);
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(vec![source_root_error(
                    source_root,
                    SourceRootErrorKind::Invalid,
                )]);
            }
        }
    }

    let relative = if normalized.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        normalized
    };
    let absolute = workspace_root.join(&relative);
    if !provider.is_dir(&absolute) {
        return Err(vec![source_root_error(
            source_root,
            SourceRootErrorKind::NotDirectory,
        )]);
    }
    Ok(relative)
}

fn parse_manifest_error(path: &Path, reason: impl std::fmt::Display) -> RenderedDiagnostic {
    crate::diagnostics::diagnostic_with_code(
        format!(
            "could not parse sifr.toml at '{}': {reason}",
            path.display()
        ),
        DiagnosticCode::WORKSPACE_MALFORMED_MANIFEST,
    )
}

fn parse_manifest_schema_error(path: &Path, reason: &'static str) -> Vec<RenderedDiagnostic> {
    vec![parse_manifest_error(path, reason)]
}

#[derive(Clone, Copy)]
enum SourceRootErrorKind {
    Escapes,
    Invalid,
    NotDirectory,
}

impl SourceRootErrorKind {
    fn code(self) -> DiagnosticCode {
        match self {
            Self::Escapes => DiagnosticCode::WORKSPACE_SOURCE_ROOT_ESCAPES,
            Self::Invalid => DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
            Self::NotDirectory => DiagnosticCode::WORKSPACE_SOURCE_ROOT_NOT_DIRECTORY,
        }
    }

    fn reason(self) -> &'static str {
        match self {
            Self::Escapes => "escapes the workspace root via '..'",
            Self::Invalid => "must be a relative non-empty path under the workspace root",
            Self::NotDirectory => "is not a directory under the workspace root",
        }
    }
}

fn source_root_error(source_root: &str, kind: SourceRootErrorKind) -> RenderedDiagnostic {
    crate::diagnostics::diagnostic_with_code(
        format!("[source].root '{source_root}' {}", kind.reason()),
        kind.code(),
    )
}

#[cfg(test)]
mod tests;
