use sifr_sql_postgresql_tools::{run_migration_command, run_schema_command};
use std::io::Write as _;
use std::process::ExitCode;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let namespace = arguments.first().cloned();
    if matches!(namespace.as_deref(), Some("schema" | "migration")) {
        arguments.remove(0);
    }
    let workspace_root = if let Ok(path) = std::env::current_dir() {
        path
    } else {
        let _ = writeln!(std::io::stderr(), "cannot determine the SQL tool workspace");
        return ExitCode::FAILURE;
    };
    let connection = std::env::var("SIFR_SQL_DATABASE_URL").ok();
    let mut emit = |value: &str| {
        let mut stdout = std::io::stdout();
        stdout
            .write_all(value.as_bytes())
            .and_then(|()| stdout.flush())
            .map_err(|_| sifr_sql_postgresql_tools::CommandError {
                message: "cannot write schema command output".to_string(),
            })
    };
    let outcome = match namespace.as_deref() {
        Some("schema") => {
            run_schema_command(
                &arguments,
                &workspace_root,
                connection.as_deref(),
                &mut emit,
            )
            .await
        }
        Some("migration") => {
            run_migration_command(&arguments, &workspace_root, connection.as_deref()).await
        }
        _ => Err(sifr_sql_postgresql_tools::CommandError {
            message: "usage: schema <command> | migration <command>".to_string(),
        }),
    };
    match outcome {
        Ok(outcome) => {
            let _ = write!(std::io::stdout(), "{}", outcome.stdout);
            ExitCode::from(outcome.exit_code)
        }
        Err(failure) => {
            let _ = writeln!(std::io::stderr(), "{}", failure.message);
            ExitCode::FAILURE
        }
    }
}
