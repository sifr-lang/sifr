use crate::{
    generate_project_with_deps_and_crates, generate_rust, generate_rust_multi_with_metadata,
    generate_rust_with_metadata, generate_rust_with_stdlib_for_module,
};
use sifr_ir::{HirExceptHandler, HirExpr, HirFunction, HirImport, HirModule, HirStmt, MethodKind};
use sifr_lowering::lower_module;
use sifr_python_parser::parse_module;
use sifr_stdlib_artifact::{StdlibCode, StdlibRustSource};
use sifr_type_system::{ParamConvention, Type};
use std::collections::HashSet;

#[path = "../support/codegen.rs"]
mod codegen_support;
pub(crate) use codegen_support::{
    empty_module, generate_rust_from_source, generate_rust_from_source_with_stdlib_collections,
    trait_impl_fixture_stdlib_code,
};
#[path = "lib_codegen_tests/async_control_codegen_tests.rs"]
mod async_control_codegen_tests;
#[path = "lib_codegen_tests/async_runtime_codegen_tests.rs"]
mod async_runtime_codegen_tests;
#[path = "lib_codegen_tests/async_task_runtime_codegen_tests.rs"]
mod async_task_runtime_codegen_tests;
#[path = "lib_codegen_tests/callable_field_codegen_tests.rs"]
mod callable_field_codegen_tests;
#[path = "lib_codegen_tests/canonical_codegen_tests.rs"]
mod canonical_codegen_tests;
#[path = "lib_codegen_tests/character_comparison_codegen_tests.rs"]
mod character_comparison_codegen_tests;
#[path = "lib_codegen_tests/checked_place_read_codegen_tests.rs"]
mod checked_place_read_codegen_tests;
#[path = "lib_codegen_tests/class_method_checked_place_codegen_tests.rs"]
mod class_method_checked_place_codegen_tests;
#[path = "lib_codegen_tests/class_trait_codegen_tests.rs"]
mod class_trait_codegen_tests;
#[path = "lib_codegen_tests/classes_and_basics_multi_module_codegen_tests.rs"]
mod classes_and_basics_multi_module_codegen_tests;
#[path = "lib_codegen_tests/control_flow_codegen_tests.rs"]
mod control_flow_codegen_tests;
#[path = "lib_codegen_tests/defaultdict_augassign_codegen_tests.rs"]
mod defaultdict_augassign_codegen_tests;
#[path = "lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs"]
mod defaultdict_order_independent_codegen_tests;
#[path = "lib_codegen_tests/empty_plain_dict_codegen_tests.rs"]
mod empty_plain_dict_codegen_tests;
#[path = "lib_codegen_tests/error_identity_codegen_tests.rs"]
mod error_identity_codegen_tests;
#[path = "lib_codegen_tests/generic_inheritance_codegen_tests.rs"]
mod generic_inheritance_codegen_tests;
#[path = "lib_codegen_tests/integer_field_augassign_codegen_tests.rs"]
mod integer_field_augassign_codegen_tests;
#[path = "lib_codegen_tests/multi_module_stdlib_feature_tests.rs"]
mod multi_module_stdlib_feature_tests;
#[path = "lib_codegen_tests/native_async_cleanup_codegen_tests.rs"]
mod native_async_cleanup_codegen_tests;
#[path = "lib_codegen_tests/nested_container_capture_codegen_tests.rs"]
mod nested_container_capture_codegen_tests;
#[path = "lib_codegen_tests/nested_function_signature_scope_codegen_tests.rs"]
mod nested_function_signature_scope_codegen_tests;
#[path = "lib_codegen_tests/ownership_quality_codegen_tests.rs"]
mod ownership_quality_codegen_tests;
#[path = "lib_codegen_tests/performance_collection_borrow_codegen_tests.rs"]
mod performance_collection_borrow_codegen_tests;
#[path = "lib_codegen_tests/performance_nested_mutation_codegen_tests.rs"]
mod performance_nested_mutation_codegen_tests;
#[path = "lib_codegen_tests/portable_secure_codegen_tests.rs"]
mod portable_secure_codegen_tests;
#[path = "lib_codegen_tests/recursive_node_codegen_tests.rs"]
mod recursive_node_codegen_tests;
#[path = "lib_codegen_tests/resumable_generator_codegen_tests.rs"]
mod resumable_generator_codegen_tests;
#[path = "lib_codegen_tests/retained_semantics_codegen_tests.rs"]
mod retained_semantics_codegen_tests;
#[path = "lib_codegen_tests/sequential_try_binding_codegen_tests.rs"]
mod sequential_try_binding_codegen_tests;
#[path = "lib_codegen_tests/string_char_cache_branch_codegen_tests.rs"]
mod string_char_cache_branch_codegen_tests;
#[path = "lib_codegen_tests/string_char_cache_control_flow_codegen_tests.rs"]
mod string_char_cache_control_flow_codegen_tests;
#[path = "lib_codegen_tests/string_concat_codegen_tests.rs"]
mod string_concat_codegen_tests;
#[path = "lib_codegen_tests/structural_record_codegen_tests.rs"]
mod structural_record_codegen_tests;
#[path = "lib_codegen_tests/structured_path_codegen_tests.rs"]
mod structured_path_codegen_tests;
#[path = "lib_codegen_tests/task_spawn_ownership_codegen_tests.rs"]
mod task_spawn_ownership_codegen_tests;
#[path = "lib_codegen_tests/template_string_codegen_tests.rs"]
mod template_string_codegen_tests;
#[path = "lib_codegen_tests/union_representation_codegen_tests.rs"]
mod union_representation_codegen_tests;
