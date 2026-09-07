//! Sifr Code Generation: translates typed HIR into Rust source code.
#![allow(dead_code)]
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

mod lib_modules_and_codegen;
pub use lib_modules_and_codegen::*;
mod builtin_errors;
pub(crate) use builtin_errors::{BUILTIN_ERROR_CLASSES, builtin_error_identity};
mod discardability;
mod generated_dependency_metadata;
mod generated_rust_canonicalizer;
mod generated_visibility;
mod task_local_support;
pub(crate) use generated_dependency_metadata::retain_generated_dependency_metadata;
pub use generated_rust_canonicalizer::{
    canonicalize_generated_rust_identifier, canonicalize_generated_rust_project,
    canonicalize_generated_rust_source, discover_project_const_function_names,
    finalize_formatted_generated_rust_source,
    finalize_formatted_generated_rust_source_with_project_consts,
};
pub(crate) use generated_rust_canonicalizer::{
    import_generated_support_in_project_nominals,
    import_project_prelude_bindings_in_generated_support, prune_generated_project_owners,
};
mod lib_async_main_cancellation;
mod lib_runtime_needs;
pub(crate) use generated_visibility::{
    crate_visible_generated_support_source, publicize_generated_module_source,
};
pub(crate) use lib_async_main_cancellation::scope_async_main_cancellation;
pub(crate) use lib_runtime_needs::{
    annotate_async_main_entrypoint, body_contains_await, replace_sync_channel_runtime_items,
    sync_channel_runtime_needed,
};
mod lib_project_codegen;
mod lib_project_signatures;
mod lib_test_project_codegen;
mod project_structural_record_codegen;
mod rust_interop_error_mapping;
pub use lib_project_codegen::*;
pub use lib_test_project_codegen::*;
pub(crate) use project_structural_record_codegen::render_project_structural_record_prelude;
mod lib_emitter_state;
pub use lib_emitter_state::*;
mod runtime_need_state;
pub(crate) use runtime_need_state::RuntimeNeeds;
mod runtime_support_demand;
pub(crate) use runtime_support_demand::RuntimeSupportDemand;
#[cfg(test)]
mod runtime_support_tests;
mod support_plan;
pub(crate) use support_plan::{
    ModuleSupportDemand, add_import_features, render_import_items, render_support,
};
mod class_emitter;
mod class_error_emitter;
mod class_field_emitter;
mod class_inheritance_impls;
mod class_method_emitter;
mod class_method_receiver_analysis;
mod class_trait_capabilities;
mod context;
mod structured_stmt_entrypoints;
pub use context::*;
mod entrypoints;
mod error_refs;
mod expr_ref_emitter;
mod expr_render_helpers;
mod field_analysis_helpers;
mod function_emitter;
mod function_generic_bounds;
mod function_like_lowering;
mod generic_bounds_helpers;
mod helpers;
pub(crate) use helpers::{
    collect_locally_defined_vars, collect_mutated_vars_with_sigs,
    collect_referenced_vars_with_types, default_param_convention,
};
mod body_analysis;
mod borrowed_string_compare;
mod checked_place;
mod checked_place_mutation;
mod exact_integer_float_compare;
pub(crate) use exact_integer_float_compare::lower_exact_integer_float_compare;
mod hir_analysis;
mod hoisted_literals;
mod intrinsic_method_emitters;
mod intrinsics;
mod ir_imports;
mod ir_optimize;
mod ir_validate;
mod lib_support;
pub(crate) use lib_modules_and_codegen::{
    IsinstanceUnionMatch, ModuleFuncSignatures, NestedFnCapture,
};
pub(crate) use lib_support::{
    homogeneous_large_tuple_backing_array, resolve_alias_type_for_plain_call,
    try_lower_leaf_or_name_expr_result,
};
pub(crate) use sifr_ir::{HirExpr, HirFunction, HirModule, HirStmt};
pub(crate) use sifr_type_system::{ParamConvention, Type};
pub(crate) use std::cell::{Cell, RefCell};
pub(crate) use std::collections::{HashMap, HashSet};
mod lower_expr;
pub use lower_expr::*;
mod lower_item;
pub use lower_item::*;
mod lower_stmt;
pub use lower_stmt::*;
mod match_guard_helpers;
mod method_call_emitter;
mod methods;
mod module_body;
mod module_constants;
mod module_prescan;
mod nested_list_element;
mod operator_protocol_emitters;
mod operator_type_rendering;
mod option_binding_mutability;
mod output_helpers;
mod ownership_plan;
mod place_emitter;
mod preamble;
mod project_constants;
mod project_stdlib_nominals;
mod project_union_prelude;
mod protocol_bridge_emitter;
mod structural_record_codegen;
pub use preamble::*;
pub(crate) use structural_record_codegen::structural_record_rust_type;
mod python_arrow_codegen;
#[cfg(test)]
mod python_arrow_codegen_tests;
mod python_buffer_codegen;
#[cfg(test)]
mod python_buffer_codegen_tests;
mod python_dlpack_codegen;
#[cfg(test)]
mod python_dlpack_codegen_tests;
mod python_interop_async;
#[cfg(test)]
mod python_interop_async_tests;
mod python_interop_callbacks;
mod python_interop_common;
mod python_interop_direct;
mod python_interop_direct_conversions;
mod python_interop_direct_helpers;
#[cfg(test)]
mod python_interop_direct_tests;
#[cfg(test)]
mod python_interop_entrypoints;
mod python_interop_plan;
#[cfg(test)]
mod python_interop_plan_tests;
mod python_interop_runtime_exprs;
mod python_raw_api_codegen;
mod python_zero_copy_arguments;
mod retained_callback_closure;
pub use python_interop_plan::{
    PythonBridgeImportPlan, PythonBridgeModulePlan, PythonBridgePackagePlan,
    PythonCallbackAttachmentPlan, PythonInteropPlan, PythonInteropPlanDeclaration,
    PythonTargetProbe, PythonTargetProbeStatus,
};
mod render;
pub use render::*;
mod rust_interop_bridge_callback_contract;
mod rust_interop_bridge_contract;
mod rust_interop_bridge_contract_serialization;
mod rust_interop_bridge_panic_contract;
mod rust_interop_callback;
mod rust_interop_direct;
mod rust_interop_direct_args;
mod rust_interop_direct_collections;
#[cfg(test)]
mod rust_interop_direct_tests;
mod rust_interop_panic;
mod rust_interop_plan;
pub use rust_interop_bridge_contract::{
    RustBridgeContractPlan, RustBridgeMethodSlotContract, RustBridgePanicErrorContract,
    RustBridgeParamContract, RustBridgeParamConvention, RustBridgeSignatureContract,
    RustBridgeTypeContract, RustBridgeTypeKind, RustGeneratedBridgeField, RustGeneratedBridgeType,
    RustGeneratedBridgeTypeKind, RustGeneratedBridgeVariant, is_rust_generated_bridge_type_path,
    rust_opaque_handle_type,
};
pub use rust_interop_bridge_panic_contract::rust_bridge_panic_error_contract;
pub use rust_interop_plan::{
    InteropBuildPlan, RustBridgeProbe, RustBridgeProbeKind, RustBridgeProbePlan,
    RustBridgeSourceDigest, RustGeneratedBridgeModule, RustInteropCargoInputs, RustInteropOwner,
    RustInteropPlan, RustInteropPlanDeclaration, RustInteropResolvedRoot,
    RustInteropResolvedTarget, RustInteropTrustRequirement, RustInteropTrustRequirementKind,
    RustStructuralShapeIdentity, interop_build_plan_for_named_modules,
};
mod rust_ir;
pub use rust_ir::*;
mod stdlib_codegen_metadata;
mod stdlib_filter;
pub use stdlib_codegen_metadata::StdlibCode;
mod stdlib_import_signatures;
mod stdlib_rust_source;
pub use stdlib_rust_source::StdlibRustSource;
mod static_program_codegen;
mod static_program_slots_codegen;
mod stdlib_demand_plan;
mod stmt_support_emitter;
mod string_char_cache;
mod string_char_cache_scan;
mod structural_identity_codegen;
mod structural_impl_codegen;
mod structural_record_fields;
pub use static_program_codegen::{
    emit_static_specialization_programs, method_slot_cache_fragment, static_program_cache_fragment,
    structural_static_program_owners, structural_static_program_owners_for_project,
};
mod try_error_carrier;
mod type_emitters;
mod union_type_helpers;

#[cfg(test)]
mod lib_codegen_tests;
#[cfg(test)]
mod type_conversion_tests;
