//! Sifr Code Generation: translates typed HIR into Rust source code.
#![allow(dead_code)]
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

mod lib_modules_and_codegen;
pub use lib_modules_and_codegen::*;
mod builtin_errors;
pub(crate) use builtin_errors::BUILTIN_ERROR_CLASSES;
mod lib_async_main_cancellation;
mod lib_runtime_needs;
pub(crate) use lib_async_main_cancellation::scope_async_main_cancellation;
pub(crate) use lib_runtime_needs::{
    annotate_async_main_entrypoint, body_contains_await, module_uses_async_exit_cause_type,
    module_uses_async_generator_type, module_uses_cancellation_error_type,
    module_uses_failure_type, module_uses_join_set, module_uses_spawn_cpu, module_uses_task_scope,
    module_uses_task_sleep, module_uses_timeout_result_type, publicize_generated_module_source,
    replace_sync_channel_runtime_items, sync_channel_runtime_needed,
};
mod lib_join_set_needs;
pub(crate) use lib_join_set_needs::module_uses_join_set_spawn_cpu;
mod lib_task_scope_offload_needs;
pub(crate) use lib_task_scope_offload_needs::{
    module_uses_task_scope_offload, module_uses_task_scope_process,
    module_uses_task_scope_spawn_cpu,
};
mod lib_project_codegen;
mod lib_project_signatures;
mod rust_interop_error_mapping;
pub use lib_project_codegen::*;
mod lib_emitter_state;
pub use lib_emitter_state::*;
mod class_emitter;
mod class_error_emitter;
mod class_field_emitter;
mod class_inheritance_impls;
mod class_method_emitter;
mod class_method_receiver_analysis;
mod class_trait_capabilities;
mod context;
pub use context::*;
mod entrypoints;
mod error_refs;
mod expr_ref_emitter;
mod expr_render_helpers;
mod field_analysis_helpers;
mod function_emitter;
mod function_like_lowering;
mod generic_bounds_helpers;
mod helpers;
pub(crate) use helpers::{
    collect_locally_defined_vars, collect_mutated_vars_with_sigs,
    collect_referenced_vars_with_types, default_param_convention, module_uses_bigint,
};
mod borrowed_string_compare;
mod hir_analysis;
mod hoisted_literals;
mod intrinsic_method_emitters;
mod intrinsics;
mod ir_imports;
mod ir_optimize;
mod ir_validate;
mod lib_support;
pub(crate) use lib_modules_and_codegen::{
    IsNoneUnionMatch, IsinstanceUnionMatch, ModuleFuncSignatures, NestedFnCapture,
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
mod output_helpers;
mod preamble;
pub use preamble::*;
mod python_buffer_codegen;
#[cfg(test)]
mod python_buffer_codegen_tests;
mod python_interop_async;
#[cfg(test)]
mod python_interop_async_tests;
mod python_interop_callbacks;
mod python_interop_common;
mod python_interop_direct;
mod python_interop_direct_conversions;
#[cfg(test)]
mod python_interop_direct_tests;
mod python_interop_plan;
#[cfg(test)]
mod python_interop_plan_tests;
mod python_interop_runtime_exprs;
pub use python_interop_plan::{
    PythonBridgeImportPlan, PythonBridgeModulePlan, PythonBridgePackagePlan,
    PythonCallbackAttachmentPlan, PythonInteropPlan, PythonInteropPlanDeclaration,
    PythonTargetProbe, PythonTargetProbeStatus,
};
mod render;
pub use render::*;
mod rust_interop_bridge_contract;
mod rust_interop_direct;
#[cfg(test)]
mod rust_interop_direct_tests;
mod rust_interop_plan;
pub use rust_interop_bridge_contract::{
    RustBridgeContractPlan, RustBridgeParamContract, RustBridgeParamConvention,
    RustBridgeSignatureContract, RustBridgeTypeContract, RustBridgeTypeKind,
    RustGeneratedBridgeField, RustGeneratedBridgeType, RustGeneratedBridgeTypeKind,
    RustGeneratedBridgeVariant,
};
pub use rust_interop_plan::{
    interop_build_plan_for_named_modules, InteropBuildPlan, RustBridgeProbe, RustBridgeProbeKind,
    RustBridgeProbePlan, RustBridgeSourceDigest, RustGeneratedBridgeModule, RustInteropCargoInputs,
    RustInteropOwner, RustInteropPlan, RustInteropPlanDeclaration, RustInteropResolvedRoot,
    RustInteropResolvedTarget, RustInteropTrustRequirement, RustInteropTrustRequirementKind,
};
mod rust_ir;
pub use rust_ir::*;
mod stdlib_filter;
mod stdlib_import_signatures;
mod stdlib_rust_source;
pub use stdlib_rust_source::StdlibRustSource;
mod stmt_support_emitter;
mod string_char_cache;
mod string_char_cache_scan;
mod try_error_carrier;
mod type_emitters;
mod union_type_helpers;

#[cfg(test)]
mod lib_codegen_tests;
