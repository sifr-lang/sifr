//! Exact MySQL compiler component for Sifr SQL.
//!
//! The crate owns its lexer, LALRPOP grammar, AST, schema normalization,
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

pub use analyzer::MysqlAnalyzer;
pub use ast::{
    MysqlColumnDefinition, MysqlCreateTable, MysqlExpression, MysqlQuery, MysqlStatement,
    MysqlStatementKind, MysqlTypeName, SqlSpan,
};
pub use codec::mysql_codec_registry;
pub use component::{
    MYSQL_QUERY_OPERATION, MYSQL_SCHEMA_ARTIFACT_KIND, MysqlCompilerComponent,
    MysqlComponentRequest, MysqlComponentResponse, component_artifact_path, component_registration,
    execute_embedded_request, mysql_capabilities, provider_diagnostics,
};
pub use diagnostic::{MysqlDiagnostic, MysqlDiagnosticCode};
pub use editor::{MysqlEditorFacts, MysqlRecoveryDocument, recover_document};
pub use lexer::{LexError, SpannedToken, Token, tokenize};
pub use migration::{MysqlDdlExecutionClass, MysqlMigrationDialect, classify_migration_ddl};
pub use parser::{MysqlParseError, MysqlParser};
pub use schema::{MysqlSchemaOptions, normalize_mysql_documents};
pub use types::{MysqlServerSeries, MysqlType, SUPPORTED_MYSQL_SERIES};
