//! PostgreSQL schema lifecycle host tools.

mod catalog;
mod command;
#[cfg(feature = "compiler")]
mod compiler_adapter;
mod migration_command;
mod migration_plan;
mod migration_runtime;
mod normalization;

pub use catalog::{PostgresCatalogError, pull_catalog_from_client, pull_live_catalog};
pub use command::{CommandError, CommandOutcome, run_schema_command_with_profile_loader};
#[cfg(feature = "compiler")]
pub use compiler_adapter::{run_migration_command, run_schema_command};
pub use migration_command::run_migration_command_with_compiler;
pub use migration_plan::{
    PostgresMigrationAction, PostgresMigrationActionKind, PostgresMigrationOperatorPlan,
    PostgresMigrationPlanError, validate_postgres_migration_plan,
};
pub use migration_runtime::{PostgresMigrationRuntime, connect_migration_runtime};
