//! Sifr Code Generation: translates typed HIR into Rust source code.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

mod lib_modules_and_codegen;
pub use lib_modules_and_codegen::{
    CodegenResult, LoweringStats, MultiModuleCodegenResult, generate_rust, generate_rust_test,
    generate_rust_with_metadata, generate_rust_with_stdlib, generate_rust_with_stdlib_for_module,
};
mod builtin_errors;
pub(crate) use builtin_errors::BUILTIN_ERROR_CLASSES;
mod lib_async_main_cancellation;
mod lib_runtime_needs;
pub(crate) use lib_async_main_cancellation::scope_async_main_cancellation;
pub(crate) use lib_runtime_needs::{
    annotate_async_main_entrypoint, body_contains_await, module_uses_async_exit_cause_type,
    module_uses_async_generator_type, module_uses_cancellation_error_type,
    module_uses_failure_type, module_uses_join_set, module_uses_spawn_cpu, module_uses_task_scope,
    module_uses_task_sleep, module_uses_timeout_result_type, replace_sync_channel_runtime_items,
    sync_channel_runtime_needed,
};
mod generated_module_publicize;
pub(crate) use generated_module_publicize::publicize_generated_module_source;
mod lib_join_set_needs;
pub(crate) use lib_join_set_needs::module_uses_join_set_spawn_cpu;
mod lib_task_scope_offload_needs;
pub(crate) use lib_task_scope_offload_needs::{
    module_uses_task_scope_offload, module_uses_task_scope_process,
    module_uses_task_scope_spawn_cpu,
};
mod lib_project_codegen;
mod lib_project_signatures;
mod lib_test_project_codegen;
mod rust_interop_error_mapping;
pub use lib_project_codegen::{
    generate_project, generate_project_with_deps, generate_project_with_deps_and_crates,
    generate_rust_multi, generate_rust_multi_with_metadata,
};
pub use lib_test_project_codegen::{
    TestProjectCodegenResult, generate_rust_test_project_with_metadata,
};
mod lib_emitter_state;
pub(crate) use lib_emitter_state::RustEmitter;
pub use lib_emitter_state::body_contains_yield;
mod class_emitter;
mod class_error_emitter;
mod class_field_emitter;
mod class_inheritance_impls;
mod class_method_emitter;
mod class_method_receiver_analysis;
mod class_trait_capabilities;
mod context;
mod structured_stmt_entrypoints;
pub use context::CodegenError;
pub(crate) use context::{ClassScope, CodegenOutcome, ScopeContext};
mod entrypoints;
mod error_refs;
mod expr_render_helpers;
mod field_analysis_helpers;
mod function_emitter;
mod function_like_lowering;
mod generic_bounds_helpers;
mod helpers;
pub(crate) use helpers::{
    collect_locally_defined_vars, collect_mutated_vars_with_sigs,
    collect_referenced_vars_with_types, default_param_convention,
};
mod borrowed_string_compare;
mod generated_source_validate;
mod hir_analysis;
mod hoisted_literals;
mod intrinsic_method_emitters;
mod intrinsics;
mod ir_imports;
mod ir_optimize;
mod ir_validate;
pub use generated_source_validate::validate_generated_rust_source;
mod lib_support;
pub(crate) use lib_modules_and_codegen::{
    FuncSignature, IsinstanceUnionMatch, ModuleFuncSignatures, NestedFnCapture,
    generate_rust_with_stdlib_for_module_with_project_policy,
    generate_rust_with_stdlib_for_module_with_structural_policy, module_class_fields,
    module_func_signatures,
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
pub(crate) use lower_expr::{
    fixed_width_literal_expr_for_target, is_leaf_expr_candidate, try_lower_leaf_expr,
    try_lower_leaf_expr_result, try_lower_task_duration_expr, with_allowed_plain_calls,
};
mod lower_item;
pub(crate) use lower_item::try_lower_simple_module_constant_item_result;
mod lower_stmt;
pub(crate) use lower_stmt::{
    build_dict_subscript_assign_stmt, build_list_subscript_assign_stmt,
    build_normalized_list_index_i64_expr, is_simple_stmt_candidate, lower_tuple_unpack_targets,
    try_lower_simple_stmt_with_scope_result_and_bindings, tuple_unpack_source_is_borrowed,
};
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
mod place_emitter;
mod preamble;
mod project_stdlib_nominals;
mod project_union_prelude;
mod protocol_bridge_emitter;
#[cfg(test)]
pub(crate) use preamble::sifr_type_to_rust_field_type;
pub(crate) use preamble::{
    build_async_exit_cause_type_items, build_async_generator_type_items,
    build_cancellation_error_type_items, build_cpu_offload_items, build_error_into_error_impl,
    build_error_type_items, build_failure_type_items, build_file_handle_infra_items,
    build_file_handle_struct_items, build_io_error_items, build_join_set_cpu_items,
    build_join_set_items, build_task_cancellation_items, build_task_context_scope_extension_items,
    build_task_current_context_items, build_task_scope_cpu_offload_items, build_task_scope_items,
    build_task_scope_offload_items, build_task_scope_process_items, build_task_supervisor_items,
    build_timeout_result_type_items, build_worker_panic_hook_items, replace_parallel_runtime_items,
    rust_type_base_name, sifr_type_to_rust_type,
};
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
pub(crate) use render::{Renderer, render_expr, render_items, render_stmts, render_type};
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
pub(crate) use rust_ir::{
    CompilerFragment, RustEnumVariant, RustExpr, RustFile, RustItem, RustLiteral, RustMatchArm,
    RustParam, RustPattern, RustStmt, RustTrait, RustType, RustTypeParam, RustWithItem, Visibility,
    user_callable_rust_name,
};
mod stdlib_codegen_metadata;
mod stdlib_filter;
pub use stdlib_codegen_metadata::StdlibCode;
mod stdlib_import_signatures;
mod stdlib_rust_source;
pub use stdlib_rust_source::StdlibRustSource;
mod static_program_codegen;
mod static_program_slots_codegen;
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
