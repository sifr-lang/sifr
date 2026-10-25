//! Test-only process for the SQL compiler session lifecycle; no compiler semantics.
use sifr_sql_contract::{MigrationNodeId, MigrationSourceDeclaration, ProfileAuthority};
use sifr_sql_tool::compiler_protocol::{CompilerOperations, serve};
use std::io::{self, Read, Write};
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

mod native_client;

const SOURCE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
struct Pipes;
impl Read for Pipes {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        io::stdin().read(bytes)
    }
}
impl Write for Pipes {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        io::stdout().write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()
    }
}
struct Operations {
    mode: String,
}
impl CompilerOperations for Operations {
    fn profile(&mut self, _: &str) -> Result<ProfileAuthority, String> {
        Err("fixture has no profile".into())
    }
    fn migration(&mut self, _: &str) -> Result<Vec<MigrationSourceDeclaration>, String> {
        if self.mode == "hang-request" {
            std::thread::sleep(Duration::from_secs(30));
        }
        if self.mode == "diagnostic" {
            return Err("line 3: test diagnostic\nline 4: test detail".into());
        }
        Ok(vec![MigrationSourceDeclaration {
            id: MigrationNodeId::new("fixture").map_err(|error| error.to_string())?,
            parents: vec![MigrationNodeId::new("base").map_err(|error| error.to_string())?],
            function: "apply".into(),
            author: "fixture".into(),
            created_at: "2026-10-25".into(),
            steps: Vec::new(),
            rollback: None,
        }])
    }
}
fn main() -> ExitCode {
    let mode = std::env::args().nth(1).unwrap_or_default();
    if mode.starts_with("native-") {
        return native_client::run(&mode);
    }
    let mode = if mode == "--sysroot" {
        if let Some(path) = std::env::var_os("SIFR_FIXTURE_WORKER_PID") {
            if std::fs::write(path, std::process::id().to_string()).is_err() {
                return ExitCode::FAILURE;
            }
        }
        "serve".to_string()
    } else {
        mode
    };
    if mode == "silent" {
        std::thread::sleep(Duration::from_secs(30));
    }
    if mode == "death" {
        return ExitCode::from(7);
    }
    if mode == "descendant-pipe" {
        let executable = match std::env::current_exe() {
            Ok(executable) => executable,
            Err(_) => return ExitCode::FAILURE,
        };
        if Command::new(executable)
            .arg("silent")
            .stdin(Stdio::null())
            .spawn()
            .is_err()
        {
            return ExitCode::FAILURE;
        }
        return ExitCode::from(7);
    }
    let source = if mode == "wrong-identity" {
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    } else {
        SOURCE
    };
    if serve(&mut Pipes, source, &mut Operations { mode: mode.clone() }).is_err() {
        return ExitCode::FAILURE;
    }
    if mode == "hang-exit" {
        std::thread::sleep(Duration::from_secs(30));
    }
    if mode == "bad-exit" {
        return ExitCode::from(7);
    }
    ExitCode::SUCCESS
}
