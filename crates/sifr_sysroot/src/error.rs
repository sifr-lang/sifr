use std::fmt::Write as _;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SysrootError {
    pub kind: SysrootErrorKind,
    pub binary_path: PathBuf,
    pub attempted_sysroot: PathBuf,
    pub asset_path: Option<PathBuf>,
    pub message: String,
}

impl SysrootError {
    pub(crate) fn new(
        kind: SysrootErrorKind,
        binary_path: PathBuf,
        attempted_sysroot: PathBuf,
        asset_path: Option<PathBuf>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            binary_path,
            attempted_sysroot,
            asset_path,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn boundary_message(&self) -> String {
        let mut message = format!(
            "{}; binary path: {}; attempted sysroot: {}",
            self.message,
            self.binary_path.display(),
            self.attempted_sysroot.display()
        );
        if let Some(asset_path) = &self.asset_path {
            let _ = write!(
                message,
                "; missing or invalid asset: {}",
                asset_path.display()
            );
        }
        message
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SysrootErrorKind {
    MissingManifest,
    MalformedManifest,
    UnsupportedSchemaVersion,
    VersionMismatch,
    UnknownManifestField,
    MissingAsset,
    InvalidWorkspace,
    NoCandidate,
}
