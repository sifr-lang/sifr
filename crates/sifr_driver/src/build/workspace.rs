use crate::diagnostics::{CompileError, CompilePhase};
use std::path::PathBuf;

pub(crate) fn create_invocation_workspace(prefix: &str) -> Result<PathBuf, Vec<CompileError>> {
    let base_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let root = std::env::temp_dir();
    for attempt in 0..8u8 {
        let unique = if attempt == 0 {
            format!("sifr_{}_{}_{}", prefix, std::process::id(), base_nanos)
        } else {
            format!(
                "sifr_{}_{}_{}_{}",
                prefix,
                std::process::id(),
                base_nanos,
                attempt
            )
        };
        let workspace = root.join(unique);
        match std::fs::create_dir(&workspace) {
            Ok(()) => return Ok(workspace),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                let workspace_display = workspace.display();
                return Err(vec![CompileError {
                    message: format!(
                        "failed to create invocation workspace '{workspace_display}': {error}"
                    ),
                    phase: CompilePhase::Build,
                }]);
            }
        }
    }
    Err(vec![CompileError {
        message: format!("failed to allocate unique invocation workspace for prefix '{prefix}'"),
        phase: CompilePhase::Build,
    }])
}

pub(crate) struct InvocationWorkspaceGuard {
    workspace: PathBuf,
}

impl InvocationWorkspaceGuard {
    pub(crate) fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

impl Drop for InvocationWorkspaceGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.workspace);
    }
}
