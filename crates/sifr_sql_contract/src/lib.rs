//! Provider-neutral compile-time SQL schema contracts.

mod bind;
mod cardinality;
mod codec;
mod component;
mod diagnostic;
mod diff;
mod effect;
mod error;
mod fingerprint;
mod generated;
mod normalization;
mod profile;
mod provider;
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
pub use diagnostic::CommonSqlDiagnostic;

pub use component::{
    SCHEMA_NORMALIZATION_OPERATION, SCHEMA_NORMALIZATION_PAYLOAD_TAG, SchemaNormalizationOutput,
    SchemaSourceArtifact, SchemaSourceInput, normalized_schema_from_response,
    schema_normalization_request, schema_source_fingerprint,
};
pub use diff::{ObjectChange, ObjectChangeKind, SchemaDiff, semantic_diff};
pub use effect::{EffectContract, QueryEffect};
pub use error::{SchemaContractError, SchemaContractErrorKind};
pub use fingerprint::{SchemaFingerprint, schema_fingerprint};
pub use generated::{
    COMPILER_KNOWN_PROFILE_EXPORTS, GeneratedProfileModule, GeneratedSchemaType,
    ProfileModuleMetadata, generate_profile_module,
};
pub use normalization::{SchemaDocument, SchemaDocumentKind, normalize_schema};
pub use profile::{
    PoolingMode, ProfileAuthority, ProfileFingerprint, RuntimeSchemaManifest, SchemaEvidence,
    SchemaProfile, SchemaStrictness, SessionContract, build_profile_authority,
    schema_context_artifact,
};
pub use provider::{
    DialectSemantics, ProviderAnalysis, ProviderAnalysisError, ProviderParameter,
    ProviderResultField,
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
