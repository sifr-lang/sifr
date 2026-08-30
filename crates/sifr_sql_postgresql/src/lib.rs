mod analysis;
mod analyzer;
mod ast;
mod catalog;
mod catalog_snapshot;
mod component;
mod ddl_constraints;
mod diagnostic;
mod ffi;
#[cfg(target_family = "wasm")]
mod guest;
mod parameters;
mod raw_adapter;
mod raw_helpers;
mod scope;
mod semantic_helpers;
mod source;
mod types;
mod writes;

pub use analysis::PostgresAnalysisError;
pub use analyzer::PostgresAnalyzer;
pub use ast::{
    Assignment, ConflictAction, ConflictClause, Expression, ExpressionKind, FromItem, JoinKind,
    OrderDirection, PostgresStatement, PostgresTypeName, SelectItem, SetOperator, StatementKind,
    SubqueryQuantifier,
};
pub use catalog::{
    CatalogCast, CatalogColumn, CatalogFunction, CatalogOperator, CatalogRelation, PostgresCatalog,
};
pub use catalog_snapshot::CatalogSnapshot;
pub use component::{
    POSTGRESQL_QUERY_OPERATION, POSTGRESQL_SCHEMA_ARTIFACT_KIND, PostgresCompilerComponent,
    PostgresComponentRequest, PostgresComponentResponse, component_artifact_path,
    component_registration, execute_embedded_request, into_embedded_response, provider_diagnostics,
};
pub use diagnostic::{
    PostgresDiagnostic, PostgresDiagnosticCode, PostgresDiagnosticSpan, PostgresSpanKind,
};
pub use parameters::{ParameterRewriteError, rewrite_parameter_slots};
pub use raw_adapter::{LibpgQueryParser, PostgresParseError, PostgresParser};
pub use source::{
    LibpgQuerySource, SUPPORTED_POSTGRESQL_MAJORS, embedded_source, embedded_sources,
};
pub use types::{PostgresType, PostgresTypeRegistry};
