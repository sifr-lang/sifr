#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::field_reassign_with_default,
        clippy::format_collect,
        clippy::uninlined_format_args,
        clippy::unwrap_used
    )
)]

mod cache;
mod diagnostics;
mod error;
mod fingerprint;
#[cfg(not(target_family = "wasm"))]
mod host;
mod protocol;
mod registration;
#[cfg(not(target_family = "wasm"))]
mod validation;

pub use cache::{AnalysisCache, CacheKey, DEFAULT_COMPONENT_CACHE_CAPACITY_BYTES};
pub use diagnostics::{DiagnosticCodeDeclaration, DiagnosticRegistry, DiagnosticRegistryOwner};
pub use error::{ComponentError, ComponentErrorKind};
pub use fingerprint::compute_plan_fingerprint;
#[cfg(not(target_family = "wasm"))]
pub use host::{ComponentHost, ComponentHostLimits, ComponentRun};
pub use protocol::{
    AnalysisContext, COMPONENT_PROTOCOL_MAJOR, ClosedType, ContextArtifact, DependencyDescriptor,
    DiagnosticLifecycle, DiagnosticSeverity, EmbeddedAnalysisRequest, EmbeddedAnalysisResponse,
    EmbeddedDiagnostic, EmbeddedPlan, HoleDescriptor, PlanKind, RecordField, RuntimeLowering,
    SemanticOperation, SourceMapEntry, SourceSpan, TemplatePart,
};
pub use registration::{
    ComponentIdentity, ComponentRegistration, ComponentRequirement, ProtocolRange,
    ResolvedComponent, resolve_component,
};
#[cfg(not(target_family = "wasm"))]
pub use validation::{validate_request, validate_response};

pub const COMPILER_COMPONENT_WIT: &str = include_str!("../wit/compiler-component.wit");

pub const SUPPORTED_COMPONENT_TARGETS: &[&str] = &[
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
];

#[cfg(test)]
mod tests;
