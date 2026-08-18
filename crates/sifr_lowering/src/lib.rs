//! Sifr HIR (High-level Intermediate Representation)
//!
//! Lowers the untyped Python AST into a typed IR with:
//! - Name resolution (every name reference resolved to a definition)
//! - Type checking (every expression carries its resolved type)
//! - Ownership tracking (move vs copy semantics)
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

pub mod cfg;
pub mod flow_graph;
mod hir_nodes;
#[cfg(test)]
mod hir_snapshot_expr_projection;
#[cfg(test)]
mod hir_snapshot_tests;
mod lower;
#[cfg(test)]
mod name_resolution_snapshot_tests;
mod scope;

pub use hir_nodes::*;
pub use lower::{
    canonicalize_user_export_function_type, canonicalize_user_export_type,
    canonicalize_user_export_type_in_place, localize_user_import_function_type,
    localize_user_import_type, lower_module,
    lower_module_sysroot_private_declaration_with_externals, lower_module_sysroot_public_stdlib,
    lower_module_sysroot_public_stdlib_with_externals, lower_module_with_externals,
    lower_module_with_externals_and_name, lower_module_with_externals_name_and_options,
    substitute_type_vars, ExternalDefs, LoweringOptions, LoweringSourceOrigin,
    PythonBridgeTargetAuthority, PythonTrustPolicy, StructuralMethodExport,
    StructuralMethodExports,
};
pub use scope::{NarrowingSnapshot, Scope};
pub use sifr_ir::{
    rust_opaque_close_method, rust_opaque_structural_mapping, rust_opaque_type_path,
    AdapterFieldDefault, AdapterFieldPlan, AdapterHandlerPlan, AppliedAdapterMetadata,
    AttachedApiDeclaration, AttachedApiReceiver, AttachedApiSetDeclaration, AttachedApiSetIdentity,
    CallableIdentity, ClassAdapterMarkerDeclaration, ClassAdapterProviderDeclaration,
    ClassAdapterSelection, ConstSpecializationRequest, DeclarationDescriptorFunction,
    DeclarationDescriptorKind, DeclarationMetadataTargetKind, HirDiagnostic, LoweringOutcome,
    LoweringResult, LoweringWarningDiagnostic, RevealTypeDiagnostic, RustInteropDecoratorKind,
    SourceOriginId, StaticMethodParam, StaticMethodSlot, StaticMethodSlotContext,
    StaticProgramValue, StaticSpecializationOutput, TypedDeclarationDescriptor,
    TypedDeclarationMetadata,
};
