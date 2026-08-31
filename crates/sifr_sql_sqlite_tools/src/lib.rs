//! SQLite schema, migration, and test-provision host tools.

mod catalog;
mod command;
mod migration_command;
mod migration_plan;
mod migration_runtime;
mod provision;

pub use catalog::{
    SqliteCatalogError, pull_live_catalog, pull_live_catalog_from_connection,
    pull_live_catalog_from_path,
};
pub use command::{CommandError, CommandOutcome, run_schema_command};
pub use migration_command::run_migration_command;
pub use migration_plan::{
    SqliteMigrationAction, SqliteMigrationPlan, SqliteMigrationPlanError,
    validate_sqlite_migration_plan,
};
pub use migration_runtime::{
    SqliteMigrationError, SqliteMigrationRuntime, connect_migration_runtime,
};
pub use provision::{cleanup_test_database, provision_test_database};

fn lower_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
