//! Shared bounded consumption and captured identities, without native production.
use sifr_metadata as wire;
use wire::Result;
pub mod consumer;
pub mod inputs;
pub mod installed;
mod project_types;
pub mod reader;
pub use consumer::{PreparedMetadata, open_consumer};
pub use project_types::{ProjectInterfacePayload, ProjectModuleReferences, ProjectTypedArtifact};
