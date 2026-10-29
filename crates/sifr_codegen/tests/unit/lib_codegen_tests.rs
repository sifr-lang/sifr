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

#[path = "../support/codegen.rs"]
mod codegen_support;
pub(crate) use codegen_support::{
    empty_module, generate_rust_from_source, generate_rust_from_source_with_stdlib_collections,
    trait_impl_fixture_stdlib_code,
};
#[cfg(test)]
#[path = "lib_codegen_tests/class_trait_contract_codegen_tests.rs"]
mod class_trait_contract_codegen_tests;
#[cfg(test)]
#[path = "lib_codegen_tests/collections_and_stdlib_codegen_tests.rs"]
mod collections_and_stdlib_codegen_tests;
#[cfg(test)]
#[path = "lib_codegen_tests/corpus_repair_codegen_tests.rs"]
mod corpus_repair_codegen_tests;
#[cfg(test)]
#[path = "lib_codegen_tests/emitted_rust_quality_codegen_tests.rs"]
mod emitted_rust_quality_codegen_tests;
#[cfg(test)]
#[path = "lib_codegen_tests/exact_integer_architecture_codegen_tests.rs"]
mod exact_integer_architecture_codegen_tests;
#[cfg(test)]
#[path = "lib_codegen_tests/iterators_and_generators_codegen_tests.rs"]
mod iterators_and_generators_codegen_tests;
#[cfg(test)]
#[path = "lib_codegen_tests/length_alias_codegen_tests.rs"]
mod length_alias_codegen_tests;
#[cfg(test)]
#[path = "lib_codegen_tests/performance_codegen_tests.rs"]
mod performance_codegen_tests;
#[cfg(test)]
#[path = "lib_codegen_tests/receiver_codegen_tests.rs"]
mod receiver_codegen_tests;
#[cfg(test)]
#[path = "lib_codegen_tests/reusable_value_codegen_tests.rs"]
mod reusable_value_codegen_tests;
#[cfg(test)]
#[path = "lib_codegen_tests/structural_impl_demand_codegen_tests.rs"]
mod structural_impl_demand_codegen_tests;
#[cfg(test)]
#[path = "lib_codegen_tests/structural_inheritance_codegen_tests.rs"]
mod structural_inheritance_codegen_tests;
#[cfg(test)]
#[path = "lib_codegen_tests/structural_stdlib_impl_codegen_tests.rs"]
mod structural_stdlib_impl_codegen_tests;
#[cfg(test)]
#[path = "lib_codegen_tests/structured_intrinsic_codegen_tests.rs"]
mod structured_intrinsic_codegen_tests;
#[cfg(test)]
#[path = "lib_codegen_tests/structured_lowering_codegen_tests.rs"]
mod structured_lowering_codegen_tests;
#[cfg(test)]
#[path = "lib_codegen_tests/support_assembly_codegen_tests.rs"]
mod support_assembly_codegen_tests;

#[path = "lib_codegen_tests/classes_and_basics_codegen_tests.rs"]
mod classes_and_basics_codegen_tests;

#[path = "lib_codegen_tests/structured_condition_codegen_tests.rs"]
mod structured_condition_codegen_tests;

#[path = "lib_codegen_tests/structured_delete_codegen_tests.rs"]
mod structured_delete_codegen_tests;
