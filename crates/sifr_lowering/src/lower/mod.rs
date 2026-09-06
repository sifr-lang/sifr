use crate::hir_nodes::{HirExpr, HirImport, HirModule};

mod mod_context;
pub use mod_context::*;
mod adapter_field_plans;
mod append_growth_shapes;
mod assignment_widening;
mod async_await;
mod async_comprehension_diagnostics;
mod async_comprehensions;
mod async_effects;
mod async_for;
mod async_generator_advances;
mod async_generator_methods;
mod async_with;
mod async_with_flow;
mod attached_api_declarations;
mod attached_api_surfaces;
mod attribute_access;
mod aug_assign_lowering;
mod binding_mutability;
mod blocking_executor_calls;
mod builtin_calls;
mod bytes_methods;
mod call_argument_ranges;
mod call_iterable_validation;
mod callable_fields;
mod class_field_inference;
#[cfg(test)]
mod class_inheritance_defaults_tests;
mod classes;
mod compiler_intrinsics;
#[cfg(test)]
mod compiler_intrinsics_tests;
mod container_literal_diagnostics;
mod container_literal_specialization;
mod contextual_list_literal_specialization;
mod control_flow_conditions;
mod decimal_methods;
mod declaration_hint_safety;
mod declaration_metadata;
mod default_args;
mod defaultdict_refinement;
mod descriptor_declarations;
#[cfg(test)]
mod diagnostic_transport_tests;
mod diagnostic_types;
mod diagnostics;
mod empty_collection_refinement;
mod empty_plain_dict_inference;
mod expression_abs;
mod expression_diagnostics;
mod expression_functional_builtins;
mod expression_iter_builtins;
mod expression_operators;
mod expression_sum_sorted;
mod expressions;
#[cfg(test)]
mod expressions_tests;
mod external_defs;
mod fixed_width_arithmetic_methods;
mod fixed_width_class_payload;
mod fixed_width_fitting;
mod flow_diagnostics;
mod flow_helpers;
mod for_loop_safety;
mod fstring_support;
mod function_flow;
mod function_scopes;
mod generic_constructor_specialization;
mod generic_inference;
mod generic_method_requirements;
mod generic_parent_representation;
mod generic_receiver_specialization;
mod guarded_index;
mod if_branch_bindings;
mod if_expression;
mod import_diagnostics;
mod import_resolution;
mod imported_class_identity;
mod imported_defaults;
mod imports;
mod integer_const_facts;
mod integer_failure_diagnostics;
mod integer_float_semantics;
mod integer_literal_diagnostics;
mod integer_literals;
mod integer_nonzero_guards;
mod ipc_payload_calls;
mod ipc_schema_extraction;
mod len_aliases;
mod match_diagnostics;
mod match_lowering;
mod method_call_args;
mod method_call_metadata;
mod method_call_verifier;
mod method_diagnostics;
mod method_receiver_analysis;
#[cfg(test)]
mod method_receiver_analysis_tests;
mod method_receiver_diagnostics;
#[cfg(test)]
mod method_receiver_diagnostics_tests;
#[cfg(test)]
mod method_receiver_footprint_tests;
#[cfg(test)]
mod method_receiver_nested_helper_tests;
mod method_receiver_places;
mod min_max_validation;
mod mod_impl;
mod module_body_lowering;
mod module_constants_lowering;
mod module_function_inference;
mod module_function_registry;
mod must_use_obligations;
#[cfg(test)]
mod mutable_call_sequence_guard_tests;
mod mutating_methods;
mod name_diagnostics;
#[cfg(test)]
mod name_import_diagnostics_tests;
mod narrowing;
mod nested_function_inference;
#[cfg(test)]
mod nested_function_tests;
mod nonempty_method_narrowing;
mod nonlocal_support;
mod numeric_sentinels;
mod offload_worker_captures;
#[cfg(test)]
mod own_mut_param_tests;
#[cfg(test)]
mod own_mut_semantics_tests;
mod ownership_diagnostics;
mod parallel_calls;
mod private_stdlib_imports;
mod protocol_diagnostics;
#[cfg(test)]
mod python_arrow_contract_tests;
#[cfg(test)]
mod python_async_context_contract_tests;
#[cfg(test)]
mod python_async_tests;
#[cfg(test)]
mod python_bridge_tests;
#[cfg(test)]
mod python_buffer_contract_tests;
#[cfg(test)]
mod python_context_expression_tests;
#[cfg(test)]
mod python_context_flow_tests;
#[cfg(test)]
mod python_coroutine_contract_tests;
#[cfg(test)]
mod python_dlpack_contract_tests;
mod python_interop;
#[cfg(test)]
mod python_interop_callback_tests;
#[cfg(test)]
mod python_interop_tests;
#[cfg(test)]
mod python_interop_validation_tests;
#[cfg(test)]
mod python_trust_tests;
#[cfg(test)]
mod required_error_message_tests;
mod result_diagnostics;
#[cfg(test)]
mod result_diagnostics_tests;
mod return_lowering;
mod rust_callback_callsite;
#[cfg(test)]
mod rust_callback_callsite_tests;
#[cfg(test)]
mod rust_callback_expression_capture_tests;
mod rust_interop;
mod rust_interop_structural;
#[cfg(test)]
mod rust_interop_structural_tests;
#[cfg(test)]
mod rust_interop_tests;
#[cfg(test)]
mod rust_opaque_constructor_tests;
mod scope_helpers;
mod sequence_guard_detection;
mod sequence_guard_updates;
mod sequence_guards;
mod sequence_pointers;
mod sequence_shapes;
mod simple_expr;
mod statement_diagnostics;
#[cfg(test)]
mod statement_diagnostics_tests;
mod statements;
mod subscript_type;
mod task_calls;
mod task_context_keywords;
mod task_handle_calls;
mod task_join_set_calls;
mod task_owner_scope_state;
mod task_scope_calls;
mod task_scope_offload_calls;
mod template_string_support;
#[cfg(test)]
mod template_string_tests;
mod try_error_propagation;
mod tuple_unpack;
#[cfg(test)]
mod type_alias_tests;
mod type_aliases;
mod type_bounds;
mod type_var_collection;
mod typevar_annotations;
mod typevar_shape_compat;
mod typing_and_functions;
mod warning_helpers;
mod workload_annotations;

use classes::{collect_class_type, lower_class};
use default_args::collect_function_defaults;
pub(in crate::lower) use diagnostic_types::{
    HirDiagnostic, LoweringWarningDiagnostic, RevealTypeDiagnostic, fallback_error_type,
};
pub use external_defs::{ExternalDefs, StructuralMethodExport, StructuralMethodExports};

pub fn localize_user_import_type(
    ty: &Type,
    module: &str,
    class_aliases: &HashMap<String, String>,
) -> Type {
    imported_class_identity::type_for_import(ty, module, class_aliases)
}

pub fn localize_user_import_function_type(
    function: &FunctionType,
    module: &str,
    class_aliases: &HashMap<String, String>,
) -> FunctionType {
    imported_class_identity::function_type_for_import(function, module, class_aliases)
}

pub fn canonicalize_user_export_type(ty: &Type, local_classes: &HashMap<String, String>) -> Type {
    imported_class_identity::canonicalize_export_type(ty, local_classes)
}

fn declared_class_identity(module: Option<&str>, name: &str) -> Option<String> {
    module.map(|module| format!("{module}.{name}"))
}

pub fn canonicalize_user_export_type_in_place(
    ty: &mut Type,
    local_classes: &HashMap<String, String>,
) {
    imported_class_identity::canonicalize_export_type_in_place(ty, local_classes);
}

pub fn canonicalize_user_export_function_type(
    function: &FunctionType,
    local_classes: &HashMap<String, String>,
) -> FunctionType {
    imported_class_identity::canonicalize_export_function_type(function, local_classes)
}
use generic_inference::infer_type_var_bindings;
use imports::resolve_imports_early;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, Stmt};
use sifr_type_system::{FunctionType, Type};
pub use sifr_type_system::{substitute_type_vars, substitute_type_vars_with_class_scopes};
use std::collections::HashMap;
use type_aliases::{collect_type_alias_decls, predeclare_type_aliases, resolve_type_aliases};
use type_var_collection::collect_type_vars;
pub(in crate::lower) use typevar_annotations::{
    decode_typevar_constraint, encode_typevar_constraint, parse_typevar_bound_expr,
    parse_typevar_declaration_specs,
};
use typing_and_functions::{
    extract_function_type, function_body_contains_yield, lower_function, register_builtins,
};

use crate::hir_nodes::{HirIteratorOp, HirParam};
use sifr_python_ast::{
    ExprAttribute, ExprCall, ExprDictComp, ExprGenerator, ExprLambda, ExprListComp, ExprNamed,
    ExprSetComp, str,
};
use sifr_type_system::ParamConvention;
