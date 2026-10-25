//! Invocation-owned semantic service behind the explicit sql-compiler grant.
use sifr_process::{OwnedChild, SocketReadiness, socket_readiness};
use sifr_sql_tool::compiler_selection::SelectedCompiler;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub(super) struct LazyCompiler {
    workspace: PathBuf,
    admission: Result<crate::sql_compiler_admission::AdmissionFile, String>,
    compiler_executable: PathBuf,
    endpoint: Option<UnixStream>,
    worker: Option<(SelectedCompiler, OwnedChild)>,
}
impl LazyCompiler {
    pub(super) fn attach(
        command: &mut Command,
        workspace: &Path,
        compiler_executable: &Path,
    ) -> Result<Self, String> {
        let admission = crate::sql_compiler_admission::AdmissionFile::capture(workspace);
        let (tool, endpoint) = UnixStream::pair().map_err(|error| error.to_string())?;
        command
            .stdin(Stdio::from(OwnedFd::from(tool)))
            .env("SIFR_TOOL_SQL_COMPILER", "stdin-v1");
        Ok(Self {
            workspace: workspace.to_owned(),
            admission,
            compiler_executable: compiler_executable.to_owned(),
            endpoint: Some(endpoint),
            worker: None,
        })
    }
    pub(super) fn poll(&mut self) -> Result<(), String> {
        let Some(endpoint) = &self.endpoint else {
            return Ok(());
        };
        match socket_readiness(endpoint).map_err(|error| error.to_string())? {
            SocketReadiness::Pending => return Ok(()),
            SocketReadiness::Closed => {
                self.endpoint = None;
                return Ok(());
            }
            SocketReadiness::Data => {}
        }
        let admission = self.admission.as_ref().map_err(Clone::clone)?;
        admission.verify()?;
        let root = sifr_sysroot::resolve_sysroot(None).map_err(|error| error.boundary_message())?;
        let selected = match root.mode() {
            sifr_sysroot::SysrootMode::SourceTreeDevelopment => {
                SelectedCompiler::source(&self.compiler_executable, root)?
            }
            sifr_sysroot::SysrootMode::InstalledToolchain => SelectedCompiler::installed(root)?,
        };
        let mut command = selected.worker_command(&self.workspace)?;
        admission.configure(&mut command);
        let endpoint = self
            .endpoint
            .take()
            .ok_or("SQL compiler channel is missing")?;
        command
            .stdin(Stdio::from(OwnedFd::from(
                endpoint.try_clone().map_err(|error| error.to_string())?,
            )))
            .stdout(Stdio::from(OwnedFd::from(endpoint)))
            .stderr(Stdio::inherit());
        let worker = OwnedChild::spawn(&mut command).map_err(|error| error.to_string())?;
        selected.verify()?;
        self.worker = Some((selected, worker));
        Ok(())
    }
    pub(super) fn finish(mut self, tool_succeeded: bool) -> Result<(), String> {
        // A failed tool may not have sent shutdown; Drop terminates its worker.
        if !tool_succeeded {
            return Ok(());
        }
        let Some((selected, mut worker)) = self.worker.take() else {
            return Ok(());
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if sifr_process::process_signals::cancelled() != 0 {
                return Err("SQL compiler cleanup cancelled".into());
            }
            if let Some(status) = worker.try_wait().map_err(|error| error.to_string())? {
                if !status.success() {
                    return Err(format!("SQL compiler exited with {status}"));
                }
                return selected.verify();
            }
            if Instant::now() >= deadline {
                return Err("SQL compiler did not finish after the tool exited".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
