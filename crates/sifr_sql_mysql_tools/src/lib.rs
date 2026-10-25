//! MySQL schema, migration, and test-provision host tools.

mod catalog;
mod command;
#[cfg(feature = "compiler")]
mod compiler_adapter;
mod migration_command;
mod migration_plan;
mod migration_runtime;
mod provision;

pub use catalog::{MysqlCatalogError, pull_live_catalog, pull_live_catalog_from_connection};
pub use command::{CommandError, CommandOutcome, run_schema_command_with_profile_loader};
#[cfg(feature = "compiler")]
pub use compiler_adapter::provision_test_database;
#[cfg(feature = "compiler")]
pub use compiler_adapter::{run_migration_command, run_schema_command};
pub use migration_command::run_migration_command_with_compiler;
pub use migration_plan::{
    MysqlMigrationAction, MysqlMigrationActionKind, MysqlMigrationOperatorPlan,
    MysqlMigrationPlanError, validate_mysql_migration_plan,
};
pub use migration_runtime::{MysqlMigrationRuntime, connect_migration_runtime};
pub use provision::{cleanup_test_database, provision_test_database_with_profile_loader};

fn lower_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
