//! Exact SQLite compiler component for Sifr SQL.
//!
//! The crate pins SQLite's Syntaqlite grammar and owns AST lowering, schema normalization,
//! semantic analysis, recovery tokens, migration classification, and editor
//! facts. It does not contain a runtime driver or use a generic SQL parser.

mod analyzer;
mod ast;
mod codec;
mod component;
mod diagnostic;
mod editor;
#[cfg(target_family = "wasm")]
mod guest;
mod lexer;
mod migration;
mod parser;
mod schema;
mod syntax_lowering;
mod types;

pub(crate) fn lower_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

pub use analyzer::SqliteAnalyzer;
pub use ast::{
    SqlSpan, SqliteColumnDefinition, SqliteConflictForm, SqliteCreateTable, SqliteExpression,
    SqliteQuery, SqliteStatement, SqliteStatementKind, SqliteTypeName,
};
pub use codec::sqlite_codec_registry;
pub use component::{
    SQLITE_QUERY_OPERATION, SQLITE_SCHEMA_ARTIFACT_KIND, SqliteCompilerComponent,
    SqliteComponentRequest, SqliteComponentResponse, component_artifact_path,
    component_registration, execute_embedded_request, provider_diagnostics, sqlite_capabilities,
};
pub use diagnostic::{SqliteDiagnostic, SqliteDiagnosticCode};
pub use editor::{SqliteEditorFacts, SqliteRecoveryDocument, recover_document};
pub use lexer::{LexError, SpannedToken, Token, tokenize};
pub use migration::{SqliteDdlExecutionClass, SqliteMigrationDialect, classify_migration_ddl};
pub use parser::{SqliteParseError, SqliteParser};
pub use schema::{SqliteSchemaOptions, normalize_sqlite_documents};
pub use types::{
    SUPPORTED_SQLITE_SERIES, SqliteAffinity, SqliteServerSeries, SqliteType, affinity,
};
