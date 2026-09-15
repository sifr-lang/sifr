use crate::{
    RustEmitter, RustExpr, RustStmt, StdlibCode, StdlibRustSource,
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

#[cfg(test)]
mod async_control_codegen_tests;
pub(crate) use async_control_codegen_tests::{
    empty_module, generate_rust_from_source, generate_rust_from_source_with_stdlib_collections,
};
#[cfg(test)]
mod async_runtime_codegen_tests;
#[cfg(test)]
mod async_task_runtime_codegen_tests;
#[cfg(test)]
mod canonical_codegen_tests;
#[cfg(test)]
mod character_comparison_codegen_tests;
#[cfg(test)]
mod checked_place_read_codegen_tests;
#[cfg(test)]
mod class_trait_codegen_tests;
#[cfg(test)]
mod class_trait_contract_codegen_tests;
#[cfg(test)]
mod classes_and_basics_codegen_tests;
#[cfg(test)]
mod classes_and_basics_multi_module_codegen_tests;
#[cfg(test)]
mod collections_and_stdlib_codegen_tests;
#[cfg(test)]
mod control_flow_codegen_tests;
#[cfg(test)]
mod corpus_repair_codegen_tests;
#[cfg(test)]
mod defaultdict_augassign_codegen_tests;
#[cfg(test)]
mod defaultdict_order_independent_codegen_tests;
#[cfg(test)]
mod emitted_rust_quality_codegen_tests;
#[cfg(test)]
mod empty_plain_dict_codegen_tests;
#[cfg(test)]
mod error_identity_codegen_tests;
#[cfg(test)]
mod exact_integer_architecture_codegen_tests;
#[cfg(test)]
mod generic_inheritance_codegen_tests;
#[cfg(test)]
mod iterators_and_generators_codegen_tests;
#[cfg(test)]
mod multi_module_stdlib_feature_tests;
#[cfg(test)]
mod native_async_cleanup_codegen_tests;
#[cfg(test)]
mod nested_container_capture_codegen_tests;
#[cfg(test)]
mod nested_function_signature_scope_codegen_tests;
#[cfg(test)]
mod ownership_quality_codegen_tests;
#[cfg(test)]
mod performance_codegen_tests;
#[cfg(test)]
mod performance_collection_borrow_codegen_tests;
#[cfg(test)]
mod performance_nested_mutation_codegen_tests;
#[cfg(test)]
mod portable_secure_codegen_tests;
#[cfg(test)]
mod receiver_codegen_tests;
#[cfg(test)]
mod recursive_node_codegen_tests;
#[cfg(test)]
mod resumable_generator_codegen_tests;
#[cfg(test)]
mod reusable_value_codegen_tests;
#[cfg(test)]
mod sequential_try_binding_codegen_tests;
#[cfg(test)]
mod string_char_cache_branch_codegen_tests;
#[cfg(test)]
mod string_char_cache_control_flow_codegen_tests;
#[cfg(test)]
mod string_concat_codegen_tests;
#[cfg(test)]
mod structural_impl_demand_codegen_tests;
#[cfg(test)]
mod structural_inheritance_codegen_tests;
#[cfg(test)]
mod structural_record_codegen_tests;
#[cfg(test)]
mod structural_stdlib_impl_codegen_tests;
#[cfg(test)]
mod structured_condition_codegen_tests;
#[cfg(test)]
mod structured_delete_codegen_tests;
#[cfg(test)]
mod structured_intrinsic_codegen_tests;
#[cfg(test)]
mod structured_lowering_codegen_tests;
#[cfg(test)]
mod structured_path_codegen_tests;
#[cfg(test)]
mod support_assembly_codegen_tests;
#[cfg(test)]
mod task_spawn_ownership_codegen_tests;
#[cfg(test)]
mod template_string_codegen_tests;
#[cfg(test)]
mod union_representation_codegen_tests;
