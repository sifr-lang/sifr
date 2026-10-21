//! Canonical semantic compiler capabilities without native Rust emission.
mod context;
pub mod diagnostics;
pub mod editor;
pub mod export_policy;
mod identity;
pub mod metadata;
pub mod package_diagnostics;
pub mod private_re_exports;
pub mod single_file;
pub mod sql_diagnostics;
pub mod sql_editor;
pub mod stdlib;
pub use context::SemanticContext;
pub use identity::compiled_input_tokens;
pub use sifr_sql_contract::QUERY_SIGNATURE_ARTIFACT_NAME;
pub use single_file::compile_sql_migration_source;
pub use sql_editor::load_sql_editor_profiles;
