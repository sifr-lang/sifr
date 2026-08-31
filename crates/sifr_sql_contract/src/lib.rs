//! Provider-neutral compile-time SQL schema contracts.

mod bind;
mod cardinality;
mod codec;
mod codec_binding;
mod component;
mod diagnostic;
mod diff;
mod effect;
mod error;
mod fingerprint;
mod fragment;
mod fragment_batches;
mod generated;
mod identifier;
mod migration;
mod normalization;
mod profile;
mod profile_registry;
mod provider;
mod provision;
mod query;
mod query_signature;
mod requirement;
mod schema;
mod slice;
mod sql_type;

pub use bind::{
    BindCompatibility, BindRejection, EncodeCheck, InputType, ParameterType, bind_compatibility,
};
pub use cardinality::{Cardinality, CardinalityError, FetchMethod};
pub use codec::{
    CodecContract, CodecIdentity, CodecRegistry, NullCodecBehavior, PanicContainment,
    WireFormatIdentity,
};
pub use codec_binding::{
    CheckedCodecBinding, CodecBindingError, CodecDecoderSignature, CodecEncoderSignature,
    CodecFunctionIdentity,
};
pub use diagnostic::CommonSqlDiagnostic;

pub use component::{
    PROVIDER_ANALYSIS_PAYLOAD_TAG, SCHEMA_NORMALIZATION_OPERATION,
    SCHEMA_NORMALIZATION_PAYLOAD_TAG, SchemaNormalizationOutput, SchemaNormalizationResult,
    SchemaSourceArtifact, SchemaSourceInput, normalized_schema_from_response,
    provider_analysis_from_response, schema_normalization_from_response,
    schema_normalization_request, schema_source_fingerprint,
};
pub use diff::{ObjectChange, ObjectChangeKind, SchemaDiff, semantic_diff};
pub use effect::{EffectContract, QueryEffect};
pub use error::{SchemaContractError, SchemaContractErrorKind};
pub use fingerprint::{SchemaFingerprint, schema_fingerprint, schema_object_fingerprint};
pub use fragment::{
    AliasIdentity, EffectTransformation, FragmentCategory, FragmentDraft, FragmentIdentity,
    PackageCapabilityResolver, PredicateContext, QueryDefinitionScope, RelationAlias,
    ResultTransformation, SqlFragment, SqlPrecedence, StaticFragmentOrigin, UnsafeSyntaxAudit,
    UnsafeSyntaxGrant, UnsafeSyntaxLint, all_predicates, any_predicates, not_predicate,
};
pub use fragment_batches::{
    AssignmentFragmentContract, BatchChunk, ConflictBatchBehavior, FragmentBatchError,
    ProviderParameterLimit, ValuesFragmentContract,
};
pub use generated::{
    COMPILER_KNOWN_PROFILE_EXPORTS, GeneratedProfileModule, GeneratedSchemaType,
    ProfileModuleMetadata, SchemaWitnessMetadata, generate_profile_module,
};
pub use identifier::{
    decode_generated_identifier, decode_generated_path, encode_generated_identifier,
    encode_generated_path,
};
pub use migration::{
    BackfillContract, CompiledMigration, CompiledMigrationGraph, CompiledMigrationPath,
    CompiledMigrationStep, CompiledStepKind, DataCallbackContract, DdlReflection, DdlRisk,
    MIGRATION_GRAPH_FORMAT_VERSION, MigrationBaseline, MigrationCompileError,
    MigrationCompileErrorKind, MigrationCompiler, MigrationDb, MigrationDefinition,
    MigrationDialect, MigrationGraphDefinition, MigrationImpact, MigrationNodeId, MigrationPlan,
    MigrationProviderConstraint, MigrationState, MigrationStateIdentity, MigrationStepDefinition,
    MigrationStepKind, ReplayPolicy, TransactionBoundary, TransactionRequirement,
    topological_order,
};
pub use normalization::{SchemaDocument, SchemaDocumentKind, normalize_schema};
pub use profile::{
    PoolingMode, ProfileAuthority, ProfileFingerprint, RuntimeSchemaManifest, SchemaEvidence,
    SchemaProfile, SchemaStrictness, SessionContract, build_profile_authority,
    dialect_modes_for_session, schema_context_artifact,
};
pub use profile_registry::{ProfileModuleRegistry, RegisteredProfileModule};
pub use provider::{
    DialectSemantics, ProviderAnalysis, ProviderAnalysisError, ProviderDiagnosticSpan,
    ProviderParameter, ProviderResultField, ProviderSemanticDiagnostic,
};
pub use provision::{
    ProvisionedCleanup, ProvisionedConnection, ProvisionedCredential,
    TEST_CONNECTION_MANIFEST_VERSION, TestConnectionManifest,
};
pub use query::{
    QueryAdapter, QueryContractError, QueryContractErrorKind, QueryOrigin, QueryParameterSlot,
    QuerySignatureRegistry, QuerySymbol, QuerySymbolKind, QueryTemplateContract,
    QueryTemplateDraft, QueryTemplateIdentity, QueryWarning, RowOfType, effect_can_unify,
};
pub use query_signature::{
    ProjectionPolicy, ProjectionStability, PublicQueryChange, PublicQueryChangeKind,
    QUERY_SIGNATURE_FORMAT_VERSION, QuerySignatureArtifact, QuerySignatureEntry,
    QuerySignatureError, QuerySignatureFormat, compare_query_signatures,
};
pub use requirement::{
    ProviderSchemaRequirement, SCHEMA_REQUIREMENT_FORMAT_VERSION, SchemaRequirement,
    SchemaRequirementError, SchemaRequirementErrorKind, SchemaRequirementIdentity,
    SchemaRequirementProof, SchemaRequirementRegistry, build_provider_schema_requirement,
    project_provider_requirement_schema,
};
pub use schema::{
    DialectIdentity, ObjectId, ProviderIdentity, SchemaIr, SchemaObject, SchemaObjectKind,
    SchemaSourceLocation, SemanticValue,
};
pub use slice::{
    AbsenceFact, ObjectRequirement, OverloadSetKind, SchemaDependencyRequest, SchemaSlice,
    minimum_schema_slice, verify_compatible_slice,
};
pub use sql_type::{
    DatabaseType, DecimalRepresentation, IntegerSign, IntegerWidth, Nullability, SifrType,
    SqliteStorageClass, canonical_read_type, canonical_read_type_in,
    canonical_read_type_with_nullability, canonical_read_type_with_nullability_in,
};

/// The canonical serialization and fingerprint contract for schema graphs.
pub const SCHEMA_IR_FORMAT_VERSION: u32 = 1;
