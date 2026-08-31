use sifr_sql_postgresql_tools::run_schema_command;
use std::io::Write as _;
use std::process::ExitCode;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments
        .first()
        .is_some_and(|argument| argument == "schema")
    {
        arguments.remove(0);
    }
    let workspace_root = if let Ok(path) = std::env::current_dir() {
        path
    } else {
        let _ = writeln!(std::io::stderr(), "cannot determine the SQL tool workspace");
        return ExitCode::FAILURE;
    };
    let connection = std::env::var("SIFR_SQL_DATABASE_URL").ok();
    match run_schema_command(&arguments, &workspace_root, connection.as_deref()).await {
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
