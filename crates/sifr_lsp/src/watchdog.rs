use crate::errors::{LspError, LspResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LspServerOptions {
    pub parent_pid: Option<u32>,
}

impl LspServerOptions {
    pub const fn stdio() -> Self {
        Self { parent_pid: None }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ParentWatchdog {
    parent_pid: Option<u32>,
}

impl ParentWatchdog {
    pub(crate) const fn new(parent_pid: Option<u32>) -> Self {
        Self { parent_pid }
    }

    pub(crate) fn check(self) -> LspResult<()> {
        let Some(parent_pid) = self.parent_pid else {
            return Ok(());
        };
        if parent_is_alive(parent_pid) {
            Ok(())
        } else {
            Err(LspError::request_cancelled(format!(
                "parent process {parent_pid} is no longer alive"
            )))
        }
    }
}

#[cfg(unix)]
fn parent_is_alive(parent_pid: u32) -> bool {
    std::process::Command::new("kill")
        .arg("-0")
        .arg(parent_pid.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(not(unix))]
fn parent_is_alive(_parent_pid: u32) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::{parent_is_alive, ParentWatchdog};

    #[test]
    fn missing_parent_pid_disables_watchdog() {
        ParentWatchdog::new(None)
            .check()
            .expect("disabled watchdog");
    }

    #[cfg(unix)]
    #[test]
    fn current_process_is_alive_for_watchdog() {
        let pid = std::process::id();
        assert!(parent_is_alive(pid));
        ParentWatchdog::new(Some(pid))
            .check()
            .expect("current process should be alive");
    }

    #[cfg(unix)]
    #[test]
    fn obviously_missing_parent_pid_cancels_server() {
        assert!(ParentWatchdog::new(Some(u32::MAX)).check().is_err());
    }
}
