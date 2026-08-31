use sifr_sql_sqlite_tools::{
    CommandError, cleanup_test_database, provision_test_database, run_migration_command,
    run_schema_command,
};
use std::io::Write as _;
use std::process::ExitCode;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let namespace = arguments.first().cloned();
    if matches!(namespace.as_deref(), Some("schema" | "migration" | "test")) {
        arguments.remove(0);
    }
    let workspace_root = match std::env::current_dir() {
        Ok(path) => path,
        Err(_) => return fail("cannot determine the SQL tool workspace"),
    };
    let connection = std::env::var("SIFR_SQL_DATABASE_PATH").ok();
    let mut emit = |value: &str| {
        let mut stdout = std::io::stdout();
        stdout
            .write_all(value.as_bytes())
            .and_then(|()| stdout.flush())
            .map_err(|_| CommandError {
                message: "cannot write schema command output".to_string(),
            })
    };
    let outcome: Result<(u8, String), CommandError> = match namespace.as_deref() {
        Some("schema") => run_schema_command(
            &arguments,
            &workspace_root,
            connection.as_deref(),
            &mut emit,
        )
        .await
        .map(|outcome| (outcome.exit_code, outcome.stdout)),
        Some("migration") => {
            run_migration_command(&arguments, &workspace_root, connection.as_deref())
                .await
                .map(|outcome| (outcome.exit_code, outcome.stdout))
        }
        Some("test") => run_test_command(&arguments, &workspace_root, connection.as_deref())
            .await
            .map(|stdout| (0, stdout)),
        _ => Err(CommandError {
            message: "usage: schema <command> | migration <command> | test <command>".to_string(),
        }),
    };
    match outcome {
        Ok((exit_code, stdout)) => {
            if std::io::stdout().write_all(stdout.as_bytes()).is_err() {
                ExitCode::FAILURE
            } else {
                ExitCode::from(exit_code)
            }
        }
        Err(error) => fail(&error.message),
    }
}

async fn run_test_command(
    arguments: &[String],
    workspace_root: &std::path::Path,
    _database_path: Option<&str>,
) -> Result<String, CommandError> {
    match arguments {
        [command, profile_flag, profile]
            if command == "provision" && profile_flag == "--profile" =>
        {
            provision_test_database(workspace_root, profile)
                .await?
                .to_canonical_json()
                .map(|json| format!("{json}\n"))
                .map_err(|error| CommandError {
                    message: error.to_string(),
                })
        }
        [command, resource_flag, resource]
            if command == "cleanup" && resource_flag == "--resource-id" =>
        {
            cleanup_test_database(workspace_root, resource).await?;
            Ok(String::new())
        }
        _ => Err(CommandError {
            message: "usage: test provision --profile <name> | test cleanup --resource-id <id>"
                .to_string(),
        }),
    }
}

fn fail(message: &str) -> ExitCode {
    let _ = writeln!(std::io::stderr(), "{message}");
    ExitCode::FAILURE
}
