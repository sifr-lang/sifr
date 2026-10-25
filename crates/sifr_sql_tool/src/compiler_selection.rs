//! Explicit compiler executable selection. A digest detects replacement; caller
//! selection and installed-generation integrity supply the execution authority.
use crate::compiler_session::CompilerSession;
use sifr_sysroot::{ResolvedSysroot, SysrootMode};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

pub struct SelectedCompiler {
    executable: PathBuf,
    executable_sha256: String,
    sysroot: ResolvedSysroot,
}

impl SelectedCompiler {
    /// Select exactly the developer-supplied executable. Never search PATH.
    pub fn source(executable: &Path, sysroot: ResolvedSysroot) -> Result<Self, String> {
        if sysroot.mode() != SysrootMode::SourceTreeDevelopment {
            return Err("source compiler selection requires a source sysroot".into());
        }
        if !executable.is_absolute() {
            return Err("source compiler selection must be an absolute executable path".into());
        }
        Self::pin(executable, sysroot)
    }

    pub fn installed(sysroot: ResolvedSysroot) -> Result<Self, String> {
        if sysroot.mode() != SysrootMode::InstalledToolchain {
            return Err("installed compiler selection requires an installed generation".into());
        }
        sysroot.verify_integrity()?;
        let executable = sysroot.root.join("bin/sifr");
        if executable
            .canonicalize()
            .map_err(|error| error.to_string())?
            != executable
            || !executable
                .symlink_metadata()
                .map_err(|error| error.to_string())?
                .is_file()
        {
            return Err("installed compiler must be a regular generation-owned executable".into());
        }
        Self::pin(&executable, sysroot)
    }

    fn pin(executable: &Path, sysroot: ResolvedSysroot) -> Result<Self, String> {
        let executable = executable
            .canonicalize()
            .map_err(|error| error.to_string())?;
        if !executable
            .metadata()
            .map_err(|error| error.to_string())?
            .is_file()
        {
            return Err("selected SQL compiler must be a regular executable file".into());
        }
        let executable_sha256 =
            sifr_sysroot::sha256_file(&executable).map_err(|error| error.to_string())?;
        Ok(Self {
            executable,
            executable_sha256,
            sysroot,
        })
    }

    pub fn verify(&self) -> Result<(), String> {
        if sifr_sysroot::sha256_file(&self.executable).map_err(|error| error.to_string())?
            != self.executable_sha256
        {
            return Err("selected SQL compiler executable changed".into());
        }
        if self.sysroot.mode() == SysrootMode::InstalledToolchain {
            if self
                .executable
                .canonicalize()
                .map_err(|error| error.to_string())?
                != self.executable
            {
                return Err("installed SQL compiler path changed outside its generation".into());
            }
            self.sysroot.verify_integrity()?;
        }
        Ok(())
    }

    /// The caller may only configure transport on this admitted worker command.
    /// It owns the resulting child and must retain this selection until cleanup.
    pub fn worker_command(&self, workspace: &Path) -> Result<Command, String> {
        self.verify()?;
        let workspace = workspace
            .canonicalize()
            .map_err(|error| error.to_string())?;
        let mut command = Command::new(&self.executable);
        command
            .arg("--sysroot")
            .arg(&self.sysroot.root)
            .arg("__sql-compiler-worker")
            .arg("--workspace")
            .arg(&workspace)
            .current_dir(workspace);
        Ok(command)
    }

    pub fn start(
        self,
        workspace: &Path,
        source_identity: &str,
        deadline: Duration,
    ) -> Result<SelectedSession, String> {
        let mut command = self.worker_command(workspace)?;
        let session = CompilerSession::spawn_admitted(&mut command, source_identity, deadline)?;
        self.verify()?;
        Ok(SelectedSession {
            selection: self,
            session,
        })
    }
}

pub struct SelectedSession {
    selection: SelectedCompiler,
    session: CompilerSession,
}
impl SelectedSession {
    pub fn profile(&mut self, name: &str) -> Result<sifr_sql_contract::ProfileAuthority, String> {
        self.session.profile(name)
    }
    pub fn migration(
        &mut self,
        source: &str,
    ) -> Result<Vec<sifr_sql_contract::MigrationSourceDeclaration>, String> {
        self.session.migration(source)
    }
    pub fn finish(self) -> Result<(), String> {
        self.session.finish()?;
        self.selection.verify()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source_root() -> ResolvedSysroot {
        sifr_sysroot::resolve_sysroot(Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")))
            .unwrap()
    }

    #[test]
    fn explicit_source_selection_rejects_relative_directory_and_replaced_executables() {
        let root = tempfile::tempdir().unwrap();
        let executable = root.path().join("compiler");
        std::fs::write(&executable, b"selected executable bytes").unwrap();
        assert!(SelectedCompiler::source(Path::new("compiler"), source_root()).is_err());
        assert!(SelectedCompiler::source(root.path(), source_root()).is_err());
        let selected = SelectedCompiler::source(&executable, source_root()).unwrap();
        selected.verify().unwrap();
        std::fs::write(&executable, b"replacement bytes").unwrap();
        assert!(selected.verify().unwrap_err().contains("changed"));
        assert!(selected.worker_command(root.path()).is_err());
    }

    #[test]
    fn installed_selection_cannot_accept_a_source_tree() {
        assert!(SelectedCompiler::installed(source_root()).is_err());
    }
}
