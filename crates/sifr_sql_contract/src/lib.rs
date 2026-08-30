//! Provider-neutral compile-time SQL schema contracts.

mod diff;
mod error;
mod fingerprint;
mod generated;
mod normalization;
mod profile;
mod schema;
mod slice;

pub use diff::{ObjectChange, ObjectChangeKind, SchemaDiff, semantic_diff};
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
pub use schema::{
    DialectIdentity, ObjectId, ProviderIdentity, SchemaIr, SchemaObject, SchemaObjectKind,
    SchemaSourceLocation, SemanticValue,
};
pub use slice::{
    AbsenceFact, ObjectRequirement, SchemaDependencyRequest, SchemaSlice, minimum_schema_slice,
    verify_compatible_slice,
};

/// The canonical serialization and fingerprint contract for schema graphs.
pub const SCHEMA_IR_FORMAT_VERSION: u32 = 1;
