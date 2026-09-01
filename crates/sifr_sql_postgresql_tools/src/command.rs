use crate::pull_live_catalog;
use serde::Serialize;
use sifr_driver::{QUERY_SIGNATURE_ARTIFACT_NAME, load_sql_editor_profiles};
use sifr_sql_contract::{
    ProfileAuthority, QuerySignatureArtifact, SchemaEvidence, SchemaIr, build_profile_authority,
    schema_source_fingerprint,
};
use sifr_sql_tool::{
    AuthorityMergeRule, NamedProfileAuthority, NamedSchema, SNAPSHOT_PATH, SchemaLifecycleError,
    build_schema_artifacts, plan_pull, resolve_build_authority, validate_schema_authorities,
    write_artifacts_atomically,
};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

const CONNECTION_ENVIRONMENT: &str = "SIFR_SQL_DATABASE_URL";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandError {
    pub message: String,
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CommandError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandOutcome {
    pub exit_code: u8,
    pub stdout: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SchemaCommand {
    Pull { profile: String, accept: bool },
    Validate { profile: String, live: bool },
    Build { profile: String },
}

pub async fn run_schema_command(
    arguments: &[String],
    workspace_root: &Path,
    connection_url: Option<&str>,
    emit: &mut impl FnMut(&str) -> Result<(), CommandError>,
) -> Result<CommandOutcome, CommandError> {
    let command = parse_command(arguments)?;
    let profile_name = match &command {
        SchemaCommand::Pull { profile, .. }
        | SchemaCommand::Validate { profile, .. }
        | SchemaCommand::Build { profile } => profile,
    };
    let authority = load_authority(workspace_root, profile_name)?;
    let output = artifact_directory(workspace_root, profile_name);
    match command {
        SchemaCommand::Pull { accept, .. } => {
            pull(&authority, &output, connection_url, accept, emit).await
        }
        SchemaCommand::Validate { live, .. } => {
            validate(&authority, &output, workspace_root, connection_url, live).await
        }
        SchemaCommand::Build { .. } => build(&authority, &output, workspace_root, profile_name),
    }
}

async fn pull(
    authority: &ProfileAuthority,
    output: &Path,
    connection_url: Option<&str>,
    accept: bool,
    emit: &mut impl FnMut(&str) -> Result<(), CommandError>,
) -> Result<CommandOutcome, CommandError> {
    let connection_url = required_connection(connection_url)?;
    let live = live_schema(authority, connection_url).await?;
    let checked = read_snapshot(output)?.unwrap_or_else(|| authority.profile.schema.clone());
    let plan = plan_pull(&checked, live, accept);
    let stdout = json_line(&plan)?;
    emit(&stdout)?;
    if plan.requires_acceptance {
        return Ok(CommandOutcome {
            exit_code: 2,
            stdout: String::new(),
        });
    }
    if let Some(schema) = plan.replacement {
        let replacement = authority_with_schema(authority, schema)?;
        let artifacts = build_schema_artifacts(&replacement).map_err(lifecycle_error)?;
        write_artifacts_atomically(output, &artifacts).map_err(lifecycle_error)?;
    }
    Ok(CommandOutcome {
        exit_code: 0,
        stdout: String::new(),
    })
}

async fn validate(
    authority: &ProfileAuthority,
    output: &Path,
    workspace_root: &Path,
    connection_url: Option<&str>,
    requested_live: bool,
) -> Result<CommandOutcome, CommandError> {
    let snapshot = read_snapshot(output)?.ok_or_else(|| {
        command_error("schema validate requires a checked-in canonical snapshot; run schema build")
    })?;
    let mut comparisons = vec![NamedSchema {
        authority: "canonical-snapshot".to_string(),
        schema: snapshot,
    }];
    let migration_path = migration_schema_path(workspace_root, &authority.profile.name);
    if migration_path.is_file() {
        comparisons.push(NamedSchema {
            authority: "migration-head".to_string(),
            schema: read_schema(&migration_path)?,
        });
    } else if authority.profile.evidence == SchemaEvidence::MigrationHead {
        return Err(command_error(
            "migration-head policy requires .sifr/sql-migrations/<profile>/schema.json",
        ));
    }
    let needs_live = requested_live || authority.profile.evidence == SchemaEvidence::Introspection;
    if needs_live {
        comparisons.push(NamedSchema {
            authority: "live-catalog".to_string(),
            schema: live_schema(authority, required_connection(connection_url)?).await?,
        });
    }
    let signatures = read_query_signatures(workspace_root)?;
    let report =
        validate_schema_authorities(&authority.profile.schema, comparisons, signatures.as_ref())
            .map_err(lifecycle_error)?;
    Ok(CommandOutcome {
        exit_code: u8::from(!report.valid),
        stdout: json_line(&report)?,
    })
}

fn build(
    authority: &ProfileAuthority,
    output: &Path,
    workspace_root: &Path,
    profile_name: &str,
) -> Result<CommandOutcome, CommandError> {
    let mut candidates = vec![NamedProfileAuthority {
        name: "declarative-source".to_string(),
        authority: authority.clone(),
    }];
    let migration_path = migration_schema_path(workspace_root, profile_name);
    if migration_path.is_file() {
        candidates.push(NamedProfileAuthority {
            name: "migration-head".to_string(),
            authority: authority_with_schema(authority, read_schema(&migration_path)?)?,
        });
    } else if authority.profile.evidence == SchemaEvidence::MigrationHead {
        return Err(command_error(
            "migration-head policy requires .sifr/sql-migrations/<profile>/schema.json",
        ));
    }
    let merge_rule = if candidates.len() == 1 {
        AuthorityMergeRule::RequireSingle
    } else {
        AuthorityMergeRule::IdenticalSchemas
    };
    let resolved = resolve_build_authority(candidates, merge_rule).map_err(lifecycle_error)?;
    let artifacts = build_schema_artifacts(&resolved).map_err(lifecycle_error)?;
    write_artifacts_atomically(output, &artifacts).map_err(lifecycle_error)?;
    Ok(CommandOutcome {
        exit_code: 0,
        stdout: json_line(&artifacts.manifest)?,
    })
}

async fn live_schema(
    authority: &ProfileAuthority,
    connection_url: &str,
) -> Result<SchemaIr, CommandError> {
    pull_live_catalog(
        connection_url,
        authority.profile.schema.provider.clone(),
        authority.profile.schema.dialect.clone(),
    )
    .await
    .map_err(|failure| command_error(failure.message))
}

pub(crate) fn load_authority(
    workspace_root: &Path,
    profile_name: &str,
) -> Result<ProfileAuthority, CommandError> {
    let prepared =
        load_sql_editor_profiles(workspace_root, workspace_root).map_err(|failures| {
            command_error(
                failures
                    .into_iter()
                    .map(|failure| failure.message)
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        })?;
    prepared
        .registry()
        .profile(profile_name)
        .map(|registered| registered.authority().clone())
        .map_err(|failure| command_error(failure.to_string()))
}

fn authority_with_schema(
    authority: &ProfileAuthority,
    schema: SchemaIr,
) -> Result<ProfileAuthority, CommandError> {
    let mut profile = authority.profile.clone();
    let source = format!(".sifr/live-schema/{}.json", profile.name);
    let bytes = serde_json::to_vec(&schema)
        .map_err(|_| command_error("cannot fingerprint accepted live schema"))?;
    profile.source_files = [source.clone()].into_iter().collect();
    profile.source_fingerprints = [(source, schema_source_fingerprint(&bytes))]
        .into_iter()
        .collect();
    profile.evidence = SchemaEvidence::Introspection;
    profile.schema = schema;
    build_profile_authority(profile).map_err(|failure| command_error(failure.to_string()))
}

fn read_snapshot(output: &Path) -> Result<Option<SchemaIr>, CommandError> {
    let path = output.join(SNAPSHOT_PATH);
    if !path.exists() {
        return Ok(None);
    }
    read_schema(&path).map(Some)
}

fn read_schema(path: &Path) -> Result<SchemaIr, CommandError> {
    let bytes = fs::read(path)
        .map_err(|_| command_error(format!("cannot read schema artifact '{}'", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|_| command_error(format!("schema artifact '{}' is invalid", path.display())))
}

fn read_query_signatures(
    workspace_root: &Path,
) -> Result<Option<QuerySignatureArtifact>, CommandError> {
    let path = workspace_root.join(QUERY_SIGNATURE_ARTIFACT_NAME);
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = fs::read(&path)
        .map_err(|_| command_error("cannot read the application query signature artifact"))?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|_| command_error("the application query signature artifact is invalid"))
}

fn artifact_directory(workspace_root: &Path, profile: &str) -> PathBuf {
    workspace_root.join(".sifr").join("sql").join(profile)
}

fn migration_schema_path(workspace_root: &Path, profile: &str) -> PathBuf {
    workspace_root
        .join(".sifr")
        .join("sql-migrations")
        .join(profile)
        .join("schema.json")
}

pub(crate) fn required_connection(connection_url: Option<&str>) -> Result<&str, CommandError> {
    connection_url
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| command_error(format!("{CONNECTION_ENVIRONMENT} is required")))
}

fn parse_command(arguments: &[String]) -> Result<SchemaCommand, CommandError> {
    let Some(command) = arguments.first().map(String::as_str) else {
        return Err(usage());
    };
    let mut profile = None;
    let mut accept = false;
    let mut live = false;
    let mut index = 1;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--profile" if profile.is_none() => {
                index += 1;
                profile = arguments.get(index).cloned();
            }
            "--accept" if !accept => accept = true,
            "--live" if !live => live = true,
            _ => return Err(usage()),
        }
        index += 1;
    }
    let profile = profile
        .filter(|value| !value.is_empty() && !value.starts_with("--"))
        .ok_or_else(usage)?;
    match command {
        "pull" if !live => Ok(SchemaCommand::Pull { profile, accept }),
        "validate" if !accept => Ok(SchemaCommand::Validate { profile, live }),
        "build" if !accept && !live => Ok(SchemaCommand::Build { profile }),
        _ => Err(usage()),
    }
}

pub(crate) fn json_line(value: &impl Serialize) -> Result<String, CommandError> {
    serde_json::to_string_pretty(value)
        .map(|value| format!("{value}\n"))
        .map_err(|_| command_error("cannot serialize schema command output"))
}

fn lifecycle_error(failure: SchemaLifecycleError) -> CommandError {
    command_error(failure.message)
}

fn usage() -> CommandError {
    command_error(
        "usage: schema pull --profile <name> [--accept] | schema validate --profile <name> [--live] | schema build --profile <name>",
    )
}

pub(crate) fn command_error(message: impl Into<String>) -> CommandError {
    CommandError {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{SchemaCommand, parse_command};

    #[test]
    fn command_parser_accepts_only_the_closed_schema_surface() {
        assert_eq!(
            parse_command(&words("pull --profile app --accept")).expect("pull"),
            SchemaCommand::Pull {
                profile: "app".to_string(),
                accept: true,
            }
        );
        assert_eq!(
            parse_command(&words("validate --profile app --live")).expect("validate"),
            SchemaCommand::Validate {
                profile: "app".to_string(),
                live: true,
            }
        );
        assert!(parse_command(&words("build --profile app --accept")).is_err());
        assert!(parse_command(&words("pull --profile app --unknown")).is_err());
        assert!(parse_command(&words("pull --profile --accept")).is_err());
        assert!(parse_command(&words("build --profile")).is_err());
    }

    fn words(value: &str) -> Vec<String> {
        value.split_whitespace().map(str::to_string).collect()
    }
}
