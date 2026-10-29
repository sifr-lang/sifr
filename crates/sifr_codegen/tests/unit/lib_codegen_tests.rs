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

#[path = "../support/codegen.rs"]
mod codegen_support;
pub(crate) use codegen_support::{
    empty_module, generate_rust_from_source, generate_rust_from_source_with_stdlib_collections,
};
#[cfg(test)]
mod class_trait_contract_codegen_tests;
#[cfg(test)]
mod collections_and_stdlib_codegen_tests;
#[cfg(test)]
mod corpus_repair_codegen_tests;
#[cfg(test)]
mod emitted_rust_quality_codegen_tests;
#[cfg(test)]
mod exact_integer_architecture_codegen_tests;
#[cfg(test)]
mod iterators_and_generators_codegen_tests;
#[cfg(test)]
mod length_alias_codegen_tests;
#[cfg(test)]
mod performance_codegen_tests;
#[cfg(test)]
mod receiver_codegen_tests;
#[cfg(test)]
mod reusable_value_codegen_tests;
#[cfg(test)]
mod structural_impl_demand_codegen_tests;
#[cfg(test)]
mod structural_inheritance_codegen_tests;
#[cfg(test)]
mod structural_stdlib_impl_codegen_tests;
#[cfg(test)]
mod structured_intrinsic_codegen_tests;
#[cfg(test)]
mod structured_lowering_codegen_tests;
#[cfg(test)]
mod support_assembly_codegen_tests;

