use sifr_stdlib_artifact::{StdlibCode, StdlibRustSource};
use crate::{
    RustEmitter, RustExpr, RustStmt,
    generate_project_with_deps_and_crates, generate_rust, generate_rust_multi,
    generate_rust_multi_with_metadata, generate_rust_test, generate_rust_with_metadata,
    generate_rust_with_stdlib_for_module,
};
use sifr_ir::{
    HirClass, HirClassKind, HirExceptHandler, HirExpr, HirFunction, HirImport, HirMatchArm,
    HirModule, HirParam, HirPattern, HirStmt, HirWithItem, HirWithItemKind, MethodKind,
};
use sifr_lowering::lower_module;
use sifr_python_parser::parse_module;
use sifr_type_system::{ParamConvention, Type};
use std::collections::HashSet;

fn trait_impl_fixture_stdlib_code() -> StdlibCode {
    let mut code = StdlibCode::default();
    code.module_rust_code.insert(
        "sifr.tomllib".to_string(),
        StdlibRustSource {
            module: "sifr.tomllib".to_string(),
            source_path: "stdlib/sifr/tomllib.sifr".to_string(),
            source_sha256: "fixture".to_string(),
            nominal_types: HashSet::new(),
            rust: r#"
struct TOMLDecodeError { message: String }
impl std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
fn loads(_text: String) -> Result<String, TOMLDecodeError> {
    Err(TOMLDecodeError { message: "fixture".to_string() })
}
"#
            .to_string(),
        },
    );
    code
}

#[path = "../support/codegen.rs"]
mod codegen_support;
pub(crate) use codegen_support::{empty_module, generate_rust_from_source, generate_rust_from_source_with_stdlib_collections};
mod async_control_codegen_tests;
mod async_runtime_codegen_tests;
mod async_task_runtime_codegen_tests;
mod callable_field_codegen_tests;
mod canonical_codegen_tests;
mod character_comparison_codegen_tests;
mod checked_place_read_codegen_tests;
mod class_method_checked_place_codegen_tests;
mod class_trait_codegen_tests;
mod classes_and_basics_codegen_tests;
mod classes_and_basics_multi_module_codegen_tests;
mod control_flow_codegen_tests;
mod defaultdict_augassign_codegen_tests;
mod defaultdict_order_independent_codegen_tests;
mod empty_plain_dict_codegen_tests;
mod error_identity_codegen_tests;
mod generic_inheritance_codegen_tests;
mod integer_field_augassign_codegen_tests;
mod multi_module_stdlib_feature_tests;
mod native_async_cleanup_codegen_tests;
mod nested_container_capture_codegen_tests;
mod nested_function_signature_scope_codegen_tests;
mod ownership_quality_codegen_tests;
mod performance_collection_borrow_codegen_tests;
mod performance_nested_mutation_codegen_tests;
mod portable_secure_codegen_tests;
mod recursive_node_codegen_tests;
mod resumable_generator_codegen_tests;
mod retained_semantics_codegen_tests;
mod sequential_try_binding_codegen_tests;
mod string_char_cache_branch_codegen_tests;
mod string_char_cache_control_flow_codegen_tests;
mod string_concat_codegen_tests;
mod structural_record_codegen_tests;
mod structured_condition_codegen_tests;
mod structured_delete_codegen_tests;
mod structured_path_codegen_tests;
mod task_spawn_ownership_codegen_tests;
mod template_string_codegen_tests;
mod union_representation_codegen_tests;
