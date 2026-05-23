use crate::diag::PackageDiagnostic;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CargoAction {
    Metadata,
    Fetch,
    Build,
    Check,
    Run,
    Test,
    Tree,
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
            Self::Check => "check",
            Self::Run => "run",
            Self::Test => "test",
            Self::Tree => "tree",
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
    PackageDiagnostic::cargo_command_failed(action, redacted)
}

fn redact_word(word: &str) -> String {
    let lower = word.to_ascii_lowercase();
    if contains_secret_marker(&lower) {
        return "[redacted]".to_string();
    }
    if let Some(redacted) = redact_url_userinfo(word) {
        redacted
    } else {
        word.to_string()
    }
}

fn contains_secret_marker(lower: &str) -> bool {
    [
        "token=",
        "bearer",
        "gh_",
        "gho_",
        "ghp_",
        "ghs_",
        "ghr_",
        "ghu_",
        "cargo:token",
        "secret=",
        "password=",
        "api_key=",
        "x-token:",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn redact_url_userinfo(word: &str) -> Option<String> {
    let scheme_end = word.find("://")?;
    let authority_start = scheme_end + 3;
    let rest = &word[authority_start..];
    let slash = rest.find('/').unwrap_or(rest.len());
    let authority = &rest[..slash];
    if !authority.contains('@') {
        return Some(word.to_string());
    }
    if authority.split('@').next().is_none_or(str::is_empty) {
        return Some(word.to_string());
    }
    let path = &rest[slash..];
    Some(format!("{}://[redacted host]{path}", &word[..scheme_end]))
}
