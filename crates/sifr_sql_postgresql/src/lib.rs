mod analysis;
mod analysis_types;
mod analyzer;
mod ast;
mod cardinality_analysis;
mod catalog;
mod catalog_advanced;
mod catalog_metadata;
mod catalog_snapshot;
mod component;
mod ddl_constraints;
mod diagnostic;
mod expression_operators;
mod ffi;
mod from_analysis;
#[cfg(target_family = "wasm")]
mod guest;
mod locking_analysis;
mod migration;
mod nullability_analysis;
mod parameters;
mod raw_adapter;
mod raw_advanced;
mod raw_helpers;
mod raw_writes;
mod result_analysis;
mod scope;
mod semantic_helpers;
mod source;
mod types;
mod window_analysis;
mod writes;

pub use analysis::PostgresAnalysisError;
pub use analyzer::PostgresAnalyzer;
pub use ast::{
    Assignment, CaseBranch, CommonTableExpression, ConflictAction, ConflictClause,
    CreateCompositeStatement, CreateRangeStatement, CteMaterialization, Expression, ExpressionKind,
    FromItem, JoinKind, LockStrength, LockWait, LockingClause, OrderDirection, PostgresStatement,
    PostgresTypeName, SelectItem, SetOperator, StatementKind, SubqueryQuantifier,
    WindowSpecification,
};
pub use catalog::{
    CatalogCast, CatalogColumn, CatalogFunction, CatalogOperator, CatalogRelation, PostgresCatalog,
};
pub use catalog_snapshot::CatalogSnapshot;
pub use component::{
    POSTGRESQL_QUERY_OPERATION, POSTGRESQL_SCHEMA_ARTIFACT_KIND, PostgresCompilerComponent,
    PostgresComponentRequest, PostgresComponentResponse, component_artifact_path,
    component_registration, execute_embedded_request, into_embedded_response,
    postgresql_capabilities, provider_diagnostics,
};
pub use diagnostic::{
    PostgresDiagnostic, PostgresDiagnosticCode, PostgresDiagnosticSpan, PostgresSpanKind,
};
pub use migration::{PostgresDdlExecutionClass, PostgresMigrationDialect, classify_migration_ddl};
pub use parameters::{ParameterRewriteError, rewrite_parameter_slots};
pub use raw_adapter::{LibpgQueryParser, PostgresParseError, PostgresParser};
pub use source::{
    LibpgQuerySource, SUPPORTED_POSTGRESQL_MAJORS, embedded_source, embedded_sources,
};
pub use types::{PostgresType, PostgresTypeRegistry, generated_sifr_type};
