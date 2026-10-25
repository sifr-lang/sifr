//! Lazy semantic access for a native SQL invocation. Database work remains local.
use crate::compiler_selection::{SelectedCompiler, SelectedSession};
use sifr_process::process_signals;
use sifr_sql_contract::{MigrationSourceDeclaration, ProfileAuthority};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[cfg(unix)]
#[path = "native_compiler/socket.rs"]
mod socket;

const OPERATION_DEADLINE: Duration = Duration::from_mins(40);
enum Session {
    Direct(SelectedSession),
    #[cfg(unix)]
    Inherited(socket::SocketSession),
}
#[derive(Default)]
struct State {
    session: Option<Session>,
    failed: bool,
}

/// The invocation owns cancellation even while its compiler is idle during a
/// database await. Constructing this controller does not launch a worker.
pub struct NativeCompiler {
    root: PathBuf,
    source_identity: &'static str,
    state: RefCell<State>,
    _signals: process_signals::Guard,
}
impl NativeCompiler {
    pub fn new(root: &Path, source_identity: &'static str) -> Result<Self, String> {
        Ok(Self {
            root: root.canonicalize().map_err(|error| error.to_string())?,
            source_identity,
            state: RefCell::new(State::default()),
            _signals: process_signals::acquire().map_err(|error| error.to_string())?,
        })
    }

    fn connect(&self) -> Result<Session, String> {
        if let Some(mode) = std::env::var_os("SIFR_TOOL_SQL_COMPILER") {
            let grants = std::env::var("SIFR_TOOL_CAPABILITIES").unwrap_or_default();
            if mode != "stdin-v1" || !grants.split(',').any(|grant| grant == "sql-compiler") {
                return Err("SQL compiler channel is missing its declared capability".into());
            }
            #[cfg(unix)]
            return socket::SocketSession::from_stdin(self.source_identity, OPERATION_DEADLINE)
                .map(Session::Inherited);
            #[cfg(not(unix))]
            return Err("inherited SQL compiler channels require a supported host sandbox".into());
        }
        if std::env::var_os("SIFR_TOOL_NAMESPACE").is_some() {
            return Err("this host tool requires the sql-compiler capability".into());
        }
        let root = sifr_sysroot::resolve_sysroot(None).map_err(|error| error.boundary_message())?;
        let source_selection = std::env::var_os("SIFR_SQL_COMPILER");
        let compiler = match root.mode() {
            sifr_sysroot::SysrootMode::SourceTreeDevelopment => {
                let executable = source_selection.ok_or(
                    "source SQL tools require SIFR_SQL_COMPILER to select an absolute compiler executable",
                )?;
                SelectedCompiler::source(Path::new(&executable), root)?
            }
            sifr_sysroot::SysrootMode::InstalledToolchain => {
                if source_selection.is_some() {
                    return Err(
                        "an installed SQL tool must use its selected generation's compiler".into(),
                    );
                }
                SelectedCompiler::installed(root)?
            }
        };
        compiler
            .start(&self.root, self.source_identity, OPERATION_DEADLINE)
            .map(Session::Direct)
    }

    fn call<T>(
        &self,
        operation: impl FnOnce(&mut Session) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut state = self
            .state
            .try_borrow_mut()
            .map_err(|_| "SQL compiler request is already active")?;
        if state.failed {
            return Err("SQL compiler invocation already failed".into());
        }
        if state.session.is_none() {
            match self.connect() {
                Ok(session) => state.session = Some(session),
                Err(error) => {
                    state.failed = true;
                    return Err(error);
                }
            }
        }
        let result = operation(
            state
                .session
                .as_mut()
                .ok_or("SQL compiler session is missing")?,
        );
        if result.is_err() {
            state.failed = true;
            state.session = None;
        }
        result
    }

    pub fn profile(&self, root: &Path, name: &str) -> Result<ProfileAuthority, String> {
        if root.canonicalize().map_err(|error| error.to_string())? != self.root {
            return Err("SQL compiler request changed its admitted project".into());
        }
        self.call(|session| match session {
            Session::Direct(session) => session.profile(name),
            #[cfg(unix)]
            Session::Inherited(session) => session.profile(name),
        })
    }

    pub fn migration(&self, source: &str) -> Result<Vec<MigrationSourceDeclaration>, String> {
        self.call(|session| match session {
            Session::Direct(session) => session.migration(source),
            #[cfg(unix)]
            Session::Inherited(session) => session.migration(source),
        })
    }

    /// Callers exit with the returned signal status after owned worker cleanup.
    /// This also permits termination of a blocked database runtime at process exit.
    pub async fn run<F: std::future::Future>(&self, future: F) -> Result<F::Output, i32> {
        tokio::pin!(future);
        loop {
            let signal = process_signals::cancelled();
            if signal != 0 {
                self.abort();
                return Err(signal);
            }
            tokio::select! {
                value = &mut future => {
                    let signal = process_signals::cancelled();
                    if signal != 0 {
                        self.abort();
                        return Err(signal);
                    }
                    return Ok(value);
                }
                _ = tokio::time::sleep(Duration::from_millis(20)) => {}
            }
        }
    }

    fn abort(&self) {
        // No request holds a RefMut across an await. The cancellation branch runs
        // only while the caller's future is suspended or has returned.
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.failed = true;
            state.session = None;
        }
    }

    pub fn finish(self) -> Result<(), String> {
        let state = self.state.into_inner();
        if state.failed {
            return Err("SQL compiler invocation already failed".into());
        }
        match state.session {
            None => Ok(()),
            Some(Session::Direct(session)) => session.finish(),
            #[cfg(unix)]
            Some(Session::Inherited(session)) => session.finish(),
        }
    }
}
