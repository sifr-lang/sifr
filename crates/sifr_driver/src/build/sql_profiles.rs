//! Driver exports for SQL profile preparation and editor diagnostics.
pub(crate) use sifr_compiler_services::sql_editor::sql_profile_import_diagnostics_for_suite;
pub use sifr_compiler_services::sql_editor::{
    PreparedSqlProfiles, load_sql_editor_profiles, prepare_sql_profiles,
    sql_profile_import_diagnostics, sql_profile_import_diagnostics_for_names,
};
