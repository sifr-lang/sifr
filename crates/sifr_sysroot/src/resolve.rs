use crate::error::{SysrootError, SysrootErrorKind};
use crate::layout::ResolvedSysroot;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub const SIFR_SYSROOT_ENV: &str = "SIFR_SYSROOT";

static PROCESS_SYSROOT_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct SysrootResolutionInput {
    pub explicit_sysroot: Option<PathBuf>,
    pub env_sysroot: Option<PathBuf>,
    pub current_exe: PathBuf,
    pub current_dir: PathBuf,
    pub allow_source_tree_development: bool,
}

pub fn resolve_sysroot(explicit_sysroot: Option<PathBuf>) -> Result<ResolvedSysroot, SysrootError> {
    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("<unknown>"));
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let input = SysrootResolutionInput {
        explicit_sysroot: explicit_sysroot.or_else(|| PROCESS_SYSROOT_OVERRIDE.get().cloned()),
        env_sysroot: std::env::var_os(SIFR_SYSROOT_ENV).map(PathBuf::from),
        current_exe,
        current_dir,
        allow_source_tree_development: is_source_tree_development_mode(),
    };
    resolve_sysroot_with(&input)
}

pub fn set_process_sysroot_override(path: PathBuf) -> Result<(), PathBuf> {
    PROCESS_SYSROOT_OVERRIDE.set(path)
}

pub fn resolve_sysroot_with(
    input: &SysrootResolutionInput,
) -> Result<ResolvedSysroot, SysrootError> {
    let mut last_error = None;
    for candidate in candidate_roots(input) {
        match ResolvedSysroot::from_root(candidate.clone(), &input.current_exe) {
            Ok(sysroot) => return Ok(sysroot),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| {
        SysrootError::new(
            SysrootErrorKind::NoCandidate,
            input.current_exe.clone(),
            PathBuf::new(),
            None,
            "no Sifr sysroot candidate was found",
        )
    }))
}

#[must_use]
pub fn is_source_tree_development_mode() -> bool {
    cfg!(debug_assertions)
}

#[must_use]
pub fn discover_source_tree_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|ancestor| {
            ancestor
                .join("crates")
                .join("sifr_runtime")
                .join("Cargo.toml")
                .is_file()
                && ancestor.join("lib").join("sifr").is_dir()
                && ancestor.join("Cargo.lock").is_file()
        })
        .map(Path::to_path_buf)
}

fn candidate_roots(input: &SysrootResolutionInput) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(root) = &input.explicit_sysroot {
        candidates.push(root.clone());
        return candidates;
    }
    if let Some(root) = &input.env_sysroot {
        candidates.push(root.clone());
        return candidates;
    }
    if let Some(root) = installed_sysroot_root(&input.current_exe) {
        candidates.push(root);
    }
    if input.allow_source_tree_development {
        candidates.extend(development_candidates(input));
    }
    dedupe_paths(candidates)
}

fn installed_sysroot_root(current_exe: &Path) -> Option<PathBuf> {
    let bin_dir = current_exe.parent()?;
    if bin_dir.file_name().is_some_and(|name| name == "bin") {
        return bin_dir.parent().map(Path::to_path_buf);
    }
    None
}

fn development_candidates(input: &SysrootResolutionInput) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(root) = discover_source_tree_root(&input.current_dir) {
        candidates.push(root);
    }
    if let Some(parent) = input.current_exe.parent() {
        if let Some(root) = discover_source_tree_root(parent) {
            candidates.push(root);
        }
    }
    candidates
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for path in paths {
        if !out.iter().any(|seen| seen == &path) {
            out.push(path);
        }
    }
    out
}
