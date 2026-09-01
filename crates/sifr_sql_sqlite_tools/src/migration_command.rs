use crate::command::{
    CommandError, CommandOutcome, command_error, json_line, load_authority, required_connection,
};
use crate::{connect_migration_runtime, validate_sqlite_execution_plan};
use sifr_sql_contract::{SchemaIr, SchemaObjectKind};
use sifr_sql_runtime::{
    MigrationEngine, MigrationExecutionLimits, MigrationExecutionPlan, MigrationId,
};
use sifr_sql_sqlite::{
    SUPPORTED_SQLITE_SERIES, SqliteAnalyzer, SqliteMigrationDialect, SqliteParser,
    SqliteSchemaOptions,
};
use sifr_sql_tool::{
    build_migration_artifacts, compile_migration_sources, load_migration_source_inputs,
    write_migration_artifacts_atomically,
};
use std::fs;
use std::path::Path;

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
    database_path: Option<&str>,
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
    validate_sqlite_execution_plan(&plan).map_err(|error| command_error(error.message))?;
    if matches!(command, MigrationCommand::Plan { .. }) {
        return Ok(CommandOutcome {
            exit_code: 0,
            stdout: json_line(&plan)?,
        });
    }
    let authority = load_authority(workspace_root, profile)?;
    let path = required_connection(database_path)?.to_string();
    let profile_name = profile.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut runtime = connect_migration_runtime(
            Path::new(&path),
            authority.profile.schema.provider,
            authority.profile.schema.dialect,
            profile_name,
        )?;
        match command {
            MigrationCommand::Import { baseline, .. } => runtime
                .import_baseline(&plan, &checked_id(&baseline)?)
                .and_then(|ledger| serialize(&ledger)),
            MigrationCommand::Apply { .. } => {
                MigrationEngine::new(MigrationExecutionLimits::default())
                    .execute(&plan, &mut runtime)
                    .map_err(|error| error.message)
                    .and_then(|report| serialize(&report))
            }
            MigrationCommand::Rollback { .. } => {
                MigrationEngine::new(MigrationExecutionLimits::default())
                    .rollback_last(&plan, &mut runtime)
                    .map_err(|error| error.message)
                    .and_then(|report| serialize(&report))
            }
            MigrationCommand::Plan { .. } => {
                Err("migration plan entered live execution".to_string())
            }
            MigrationCommand::Build { .. } => {
                Err("migration build entered live execution".to_string())
            }
        }
    })
    .await
    .map_err(|_| command_error("SQLite migration worker stopped"))?
    .map_err(command_error)?;
    Ok(CommandOutcome {
        exit_code: 0,
        stdout: result,
    })
}

fn build(workspace_root: &Path, profile: &str) -> Result<CommandOutcome, CommandError> {
    let authority = load_authority(workspace_root, profile)?;
    let target = authority.profile.schema;
    let (parser, options) = sqlite_build_context(&target)?;
    let dialect = SqliteMigrationDialect::new(parser.clone(), options);
    let inputs = load_migration_source_inputs(workspace_root, profile, compile_migration_source)
        .map_err(|failure| command_error(failure.message))?;
    let graph = compile_migration_sources(
        &dialect,
        target.clone(),
        inputs.baselines,
        inputs.declarations,
        |schema, statement| {
            SqliteAnalyzer::new(&parser, schema)
                .and_then(|analyzer| analyzer.analyze_query(statement))
                .map_err(|failure| failure.message)
        },
    )
    .map_err(|failure| command_error(failure.message))?;
    let artifacts = build_migration_artifacts(&graph, &target)
        .map_err(|failure| command_error(failure.message))?;
    let output = workspace_root
        .join(".sifr")
        .join("sql-migrations")
        .join(profile);
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

fn sqlite_build_context(
    schema: &SchemaIr,
) -> Result<(SqliteParser, SqliteSchemaOptions), CommandError> {
    let series = SUPPORTED_SQLITE_SERIES
        .into_iter()
        .find(|series| series.version() == schema.dialect.server_version)
        .ok_or_else(|| command_error("migration profile selects an unsupported SQLite series"))?;
    let parser = SqliteParser::new(series, schema.dialect.modes.clone())
        .map_err(|failure| command_error(failure.to_string()))?;
    let attached_schemas = schema
        .objects
        .values()
        .filter(|object| object.kind == SchemaObjectKind::Namespace)
        .map(|object| object.identity.to_string())
        .filter(|name| name != "main")
        .collect();
    Ok((
        parser,
        SqliteSchemaOptions {
            default_schema: "main".to_string(),
            compile_flags: schema.dialect.modes.clone(),
            attached_schemas,
            required_features: schema.dialect.features.clone(),
            extensions: std::collections::BTreeSet::new(),
        },
    ))
}

fn read_plan(root: &Path, profile: &str) -> Result<MigrationExecutionPlan, CommandError> {
    let path = root
        .join(".sifr")
        .join("sql-migrations")
        .join(profile)
        .join("graph.json");
    serde_json::from_slice(
        &fs::read(&path).map_err(|_| {
            command_error(format!("cannot read migration plan '{}'", path.display()))
        })?,
    )
    .map_err(|_| command_error(format!("migration plan '{}' is invalid", path.display())))
}

fn parse_command(arguments: &[String]) -> Result<MigrationCommand, CommandError> {
    let command = arguments.first().map(String::as_str).ok_or_else(usage)?;
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
        .filter(|value| !value.is_empty())
        .ok_or_else(usage)?;
    match command {
        "build" if baseline.is_none() => Ok(MigrationCommand::Build { profile }),
        "plan" if baseline.is_none() => Ok(MigrationCommand::Plan { profile }),
        "import" => Ok(MigrationCommand::Import {
            profile,
            baseline: baseline
                .filter(|value| !value.is_empty())
                .ok_or_else(usage)?,
        }),
        "apply" if baseline.is_none() => Ok(MigrationCommand::Apply { profile }),
        "rollback" if baseline.is_none() => Ok(MigrationCommand::Rollback { profile }),
        _ => Err(usage()),
    }
}

fn checked_id(value: &str) -> Result<MigrationId, String> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err("migration baseline identity is invalid".to_string());
    }
    Ok(MigrationId::new(value))
}

fn serialize(value: &impl serde::Serialize) -> Result<String, String> {
    serde_json::to_string_pretty(value)
        .map(|value| format!("{value}\n"))
        .map_err(|_| "cannot serialize migration result".to_string())
}

fn usage() -> CommandError {
    command_error(
        "usage: migration build --profile <name> | migration plan --profile <name> | migration import --profile <name> --baseline <id> | migration apply --profile <name> | migration rollback --profile <name>",
    )
}
