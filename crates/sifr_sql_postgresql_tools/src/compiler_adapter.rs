//! Default in-process compiler adapter. Native operations only consume explicit inputs.
use crate::{CommandError, CommandOutcome};
use sifr_sql_contract::ProfileAuthority;
use std::path::Path;

pub async fn run_schema_command(
    arguments: &[String],
    workspace_root: &Path,
    connection_url: Option<&str>,
    emit: &mut impl FnMut(&str) -> Result<(), CommandError>,
) -> Result<CommandOutcome, CommandError> {
    crate::run_schema_command_with_profile_loader(
        &load_authority,
        arguments,
        workspace_root,
        connection_url,
        emit,
    )
    .await
}

pub async fn run_migration_command(
    compiler: &sifr_semantic_services::SemanticContext,
    arguments: &[String],
    workspace_root: &Path,
    connection_url: Option<&str>,
) -> Result<CommandOutcome, CommandError> {
    crate::run_migration_command_with_compiler(
        &|source| {
            sifr_semantic_services::compile_sql_migration_source(compiler, source).map_err(
                |failures| {
                    failures
                        .into_iter()
                        .map(|failure| failure.message)
                        .collect::<Vec<_>>()
                        .join("\n")
                },
            )
        },
        &load_authority,
        arguments,
        workspace_root,
        connection_url,
    )
    .await
}

fn load_authority(workspace_root: &Path, profile: &str) -> Result<ProfileAuthority, String> {
    let prepared =
        sifr_semantic_services::load_sql_editor_profiles(workspace_root, workspace_root, None)
            .map_err(|failures| {
                failures
                    .into_iter()
                    .map(|failure| failure.message)
                    .collect::<Vec<_>>()
                    .join("\n")
            })?;
    prepared
        .registry()
        .profile(profile)
        .map(|registered| registered.authority().clone())
        .map_err(|error| error.to_string())
}
