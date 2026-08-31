//! Provider-neutral schema lifecycle contracts for Sifr host tools.

mod artifacts;
mod lifecycle;
mod migration_artifacts;
mod transaction;

pub use artifacts::{
    ARTIFACT_MANIFEST_PATH, DEPENDENCY_INDEX_PATH, FINGERPRINT_PATH, GENERATED_METADATA_PATH,
    GENERATED_MODULE_PATH, RUNTIME_MANIFEST_PATH, SCHEMA_ARTIFACT_FORMAT_VERSION, SNAPSHOT_PATH,
    SchemaArtifactManifest, SchemaArtifactRecord, SchemaBuildArtifacts, SchemaDependencyIndex,
    build_schema_artifacts,
};
pub use lifecycle::{
    AuthorityMergeRule, NamedProfileAuthority, NamedSchema, SchemaAuthorityDiff,
    SchemaLifecycleError, SchemaLifecycleErrorKind, SchemaPullPlan, SchemaValidationReport,
    affected_queries, plan_pull, resolve_build_authority, validate_schema_authorities,
};
pub use migration_artifacts::{
    MIGRATION_ARTIFACT_MANIFEST_PATH, MIGRATION_GRAPH_PATH, MIGRATION_IMPACT_PATH,
    MIGRATION_SCHEMA_PATH, MigrationArtifactManifest, MigrationBuildArtifacts,
    build_migration_artifacts, lower_migration_execution_plan,
};
pub use transaction::{write_artifacts_atomically, write_migration_artifacts_atomically};
