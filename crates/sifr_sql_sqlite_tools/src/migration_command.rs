use crate::command::{CommandError, CommandOutcome, command_error, json_line, required_connection};
use crate::{SqliteMigrationPlan, connect_migration_runtime, validate_sqlite_migration_plan};
use std::fs;
use std::path::Path;

enum MigrationCommand {
    Plan { profile: String },
    Apply { profile: String },
}

pub async fn run_migration_command(
    arguments: &[String],
    workspace_root: &Path,
    database_path: Option<&str>,
) -> Result<CommandOutcome, CommandError> {
    let command = parse_command(arguments)?;
    let profile = match &command {
        MigrationCommand::Plan { profile } | MigrationCommand::Apply { profile } => profile,
    };
    let plan = read_plan(workspace_root, profile)?;
    validate_sqlite_migration_plan(&plan).map_err(|failure| command_error(failure.message))?;
    if matches!(command, MigrationCommand::Plan { .. }) {
        return Ok(CommandOutcome {
            exit_code: 0,
            stdout: json_line(&plan)?,
        });
    }
    let path = required_connection(database_path)?.to_string();
    tokio::task::spawn_blocking(move || {
        let mut runtime =
            connect_migration_runtime(Path::new(&path)).map_err(|failure| failure.message)?;
        runtime.apply(&plan).map_err(|failure| failure.message)
    })
    .await
    .map_err(|_| command_error("SQLite migration worker stopped"))?
    .map_err(command_error)?;
    Ok(CommandOutcome {
        exit_code: 0,
        stdout: String::new(),
    })
}

fn read_plan(root: &Path, profile: &str) -> Result<SqliteMigrationPlan, CommandError> {
    let path = root
        .join(".sifr")
        .join("sql-migrations")
        .join(profile)
        .join("sqlite-plan.json");
    serde_json::from_slice(
        &fs::read(&path).map_err(|_| {
            command_error(format!("cannot read migration plan '{}'", path.display()))
        })?,
    )
    .map_err(|_| command_error(format!("migration plan '{}' is invalid", path.display())))
}

fn parse_command(arguments: &[String]) -> Result<MigrationCommand, CommandError> {
    let [command, profile_flag, profile] = arguments else {
        return Err(usage());
    };
    if profile_flag != "--profile" || profile.is_empty() {
        return Err(usage());
    }
    match command.as_str() {
        "plan" => Ok(MigrationCommand::Plan {
            profile: profile.clone(),
        }),
        "apply" => Ok(MigrationCommand::Apply {
            profile: profile.clone(),
        }),
        _ => Err(usage()),
    }
}

fn usage() -> CommandError {
    command_error("usage: migration plan --profile <name> | migration apply --profile <name>")
}
