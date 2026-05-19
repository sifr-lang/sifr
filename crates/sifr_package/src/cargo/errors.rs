#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CargoAction {
    Metadata,
    Fetch,
    Build,
    Package,
    Publish,
    Vendor,
    Add,
    Remove,
    Update,
}

impl CargoAction {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Metadata => "metadata",
            Self::Fetch => "fetch",
            Self::Build => "build",
            Self::Package => "package",
            Self::Publish => "publish",
            Self::Vendor => "vendor",
            Self::Add => "add",
            Self::Remove => "remove",
            Self::Update => "update",
        }
    }
}

#[must_use]
pub fn looks_like_credentials_error(stderr: &str) -> bool {
    let lower = stderr.to_ascii_lowercase();
    [
        "authentication",
        "credential",
        "credentials",
        "login",
        "unauthorized",
        "forbidden",
        "401",
        "403",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

#[must_use]
pub fn redact_cargo_stderr(stderr: &str) -> String {
    stderr
        .split_whitespace()
        .map(redact_word)
        .collect::<Vec<_>>()
        .join(" ")
}

#[must_use]
pub fn map_cargo_failure(action: CargoAction, stderr: &str) -> PackageDiagnostic {
    let redacted = redact_cargo_stderr(stderr);
    if looks_like_credentials_error(stderr) {
        PackageDiagnostic::credentials_unavailable(action, redacted)
    } else {
        PackageDiagnostic::cargo_command_failed(action, redacted)
    }
}

fn redact_word(word: &str) -> &str {
    if word.starts_with("token=")
        || word.starts_with("Bearer")
        || word.starts_with("gho_")
        || word.starts_with("cargo:token")
        || word.contains("://")
    {
        "[redacted]"
    } else {
        word
    }
}
use crate::diag::PackageDiagnostic;
