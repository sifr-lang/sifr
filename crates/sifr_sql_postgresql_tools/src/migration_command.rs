use crate::command::{
    CommandError, CommandOutcome, command_error, json_line, load_authority, required_connection,
};
use crate::{connect_migration_runtime, validate_postgres_migration_plan};
use sifr_sql_postgresql::{
    LibpgQueryParser, PostgresAnalyzer, PostgresCatalog, PostgresMigrationDialect, PostgresParser,
    PostgresTypeRegistry, postgresql_capabilities,
};
use sifr_sql_runtime::{
    MigrationEngine, MigrationExecutionLimits, MigrationExecutionPlan, MigrationId,
};
use sifr_sql_tool::{
    build_migration_artifacts, compile_migration_sources, load_migration_source_inputs,
    write_migration_artifacts_atomically,
};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
enum MigrationCommand {
    Build { profile: String },
    Plan { profile: String },
    Import { profile: String, baseline: String },
    Apply { profile: String },
    Rollback { profile: String },
}

pub async fn run_migration_command(
    arguments: &[String],
    workspace_root: &Path,
    connection_url: Option<&str>,
) -> Result<CommandOutcome, CommandError> {
    let command = parse_command(arguments)?;
    let profile = match &command {
        MigrationCommand::Build { profile }
        | MigrationCommand::Plan { profile }
        | MigrationCommand::Import { profile, .. }
        | MigrationCommand::Apply { profile }
        | MigrationCommand::Rollback { profile } => profile,
    };
    if matches!(command, MigrationCommand::Build { .. }) {
        return build(workspace_root, profile);
    }
    let plan = read_plan(workspace_root, profile)?;
    let operator_plan = validate_postgres_migration_plan(&plan)
        .map_err(|failure| command_error(failure.message))?;
    if matches!(command, MigrationCommand::Plan { .. }) {
        return Ok(CommandOutcome {
            exit_code: 0,
            stdout: json_line(&operator_plan)?,
        });
    }
    let authority = load_authority(workspace_root, profile)?;
    let connection = required_connection(connection_url)?.to_string();
    let profile_name = profile.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut runtime = connect_migration_runtime(
            &connection,
            authority.profile.schema.provider,
            authority.profile.schema.dialect,
            profile_name,
        )?;
        match command {
            MigrationCommand::Import { baseline, .. } => {
                let baseline = checked_id(&baseline)?;
                runtime
                    .import_baseline(&plan, &baseline)
                    .and_then(|ledger| {
                        serde_json::to_string_pretty(&ledger)
                            .map(|value| format!("{value}\n"))
                            .map_err(|_| "cannot serialize migration import result".to_string())
                    })
            }
            MigrationCommand::Apply { .. } => {
                MigrationEngine::new(MigrationExecutionLimits::default())
                    .execute(&plan, &mut runtime)
                    .map_err(|failure| failure.message)
                    .and_then(|report| {
                        serde_json::to_string_pretty(&report)
                            .map(|value| format!("{value}\n"))
                            .map_err(|_| "cannot serialize migration execution result".to_string())
                    })
            }
            MigrationCommand::Rollback { .. } => {
                MigrationEngine::new(MigrationExecutionLimits::default())
                    .rollback_last(&plan, &mut runtime)
                    .map_err(|failure| failure.message)
                    .and_then(|report| {
                        serde_json::to_string_pretty(&report)
                            .map(|value| format!("{value}\n"))
                            .map_err(|_| "cannot serialize migration rollback result".to_string())
                    })
            }
            MigrationCommand::Plan { .. } => {
                Err("migration plan command entered live execution".to_string())
            }
            MigrationCommand::Build { .. } => {
                Err("migration build command entered live execution".to_string())
            }
        }
    })
    .await
    .map_err(|_| command_error("PostgreSQL migration worker stopped"))?
    .map_err(command_error)?;
    Ok(CommandOutcome {
        exit_code: 0,
        stdout: result,
    })
}

fn build(workspace_root: &Path, profile: &str) -> Result<CommandOutcome, CommandError> {
    let authority = load_authority(workspace_root, profile)?;
    let target = authority.profile.schema;
    let parser = LibpgQueryParser;
    let dialect = PostgresMigrationDialect::new(
        parser,
        target.dialect.server_version.clone(),
        postgresql_capabilities(),
    );
    let inputs = load_migration_source_inputs(workspace_root, profile, compile_migration_source)
        .map_err(|failure| command_error(failure.message))?;
    let graph = compile_migration_sources(
        &dialect,
        target.clone(),
        inputs.baselines,
        inputs.declarations,
        |schema, statement| {
            let catalog = PostgresCatalog::from_schema(
                schema,
                PostgresTypeRegistry::new(parser.server_major()),
            )
            .map_err(|failure| failure.message)?;
            PostgresAnalyzer::new(parser, catalog)
                .analyze_query(statement)
                .map_err(|failure| failure.to_string())
        },
    )
    .map_err(|failure| command_error(failure.message))?;
    let artifacts = build_migration_artifacts(&graph, &target)
        .map_err(|failure| command_error(failure.message))?;
    let output = migration_directory(workspace_root, profile);
    write_migration_artifacts_atomically(&output, &artifacts)
        .map_err(|failure| command_error(failure.message))?;
    Ok(CommandOutcome {
        exit_code: 0,
        stdout: json_line(&artifacts.manifest)?,
    })
}

fn compile_migration_source(
    source: &str,
) -> Result<Vec<sifr_sql_contract::MigrationSourceDeclaration>, String> {
    sifr_driver::compile_sql_migration_source(source).map_err(|failures| {
        failures
            .into_iter()
            .map(|failure| failure.message)
            .collect::<Vec<_>>()
            .join("\n")
    })
}

fn migration_directory(workspace_root: &Path, profile: &str) -> std::path::PathBuf {
    workspace_root
        .join(".sifr")
        .join("sql-migrations")
        .join(profile)
}

fn read_plan(workspace_root: &Path, profile: &str) -> Result<MigrationExecutionPlan, CommandError> {
    let path = workspace_root
        .join(".sifr")
        .join("sql-migrations")
        .join(profile)
        .join("graph.json");
    let bytes = fs::read(&path).map_err(|_| {
        command_error(format!(
            "cannot read migration execution plan '{}'",
            path.display()
        ))
    })?;
    serde_json::from_slice(&bytes).map_err(|_| {
        command_error(format!(
            "migration execution plan '{}' is invalid",
            path.display()
        ))
    })
}

fn parse_command(arguments: &[String]) -> Result<MigrationCommand, CommandError> {
    let Some(command) = arguments.first().map(String::as_str) else {
        return Err(usage());
    };
    let mut profile = None;
    let mut baseline = None;
    let mut index = 1;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--profile" if profile.is_none() => {
                index += 1;
                profile = arguments.get(index).cloned();
            }
            "--baseline" if baseline.is_none() => {
                index += 1;
                baseline = arguments.get(index).cloned();
            }
            _ => return Err(usage()),
        }
        index += 1;
    }
    let profile = profile
        .filter(|value| !value.is_empty() && !value.starts_with("--"))
        .ok_or_else(usage)?;
    match command {
        "build" if baseline.is_none() => Ok(MigrationCommand::Build { profile }),
        "plan" if baseline.is_none() => Ok(MigrationCommand::Plan { profile }),
        "import" => Ok(MigrationCommand::Import {
            profile,
            baseline: baseline
                .filter(|value| !value.is_empty() && !value.starts_with("--"))
                .ok_or_else(usage)?,
        }),
        "apply" if baseline.is_none() => Ok(MigrationCommand::Apply { profile }),
        "rollback" if baseline.is_none() => Ok(MigrationCommand::Rollback { profile }),
        _ => Err(usage()),
    }
}

fn checked_id(value: &str) -> Result<MigrationId, String> {
    if value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err("migration baseline identity is invalid".to_string());
    }
    Ok(MigrationId::new(value))
}

fn usage() -> CommandError {
    command_error(
        "usage: migration build --profile <name> | migration plan --profile <name> | migration import --profile <name> --baseline <id> | migration apply --profile <name> | migration rollback --profile <name>",
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{MigrationCommand, parse_command};

    #[test]
    fn parser_accepts_only_the_closed_migration_surface() {
        assert_eq!(
            parse_command(&words("import --profile app --baseline production")).expect("import"),
            MigrationCommand::Import {
                profile: "app".to_string(),
                baseline: "production".to_string(),
            }
        );
        assert!(parse_command(&words("apply --profile app")).is_ok());
        assert!(parse_command(&words("rollback --profile app")).is_ok());
        assert!(parse_command(&words("plan --profile app")).is_ok());
        assert!(parse_command(&words("build --profile app")).is_ok());
        assert!(parse_command(&words("apply --profile app --baseline old")).is_err());
    }

    fn words(value: &str) -> Vec<String> {
        value.split_whitespace().map(str::to_string).collect()
    }
}
