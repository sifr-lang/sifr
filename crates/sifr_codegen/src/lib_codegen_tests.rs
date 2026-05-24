use crate::{
    generate_project_with_deps_and_crates, generate_rust, generate_rust_multi,
    generate_rust_multi_with_metadata, generate_rust_test, generate_rust_with_metadata,
    RustEmitter, RustExpr, RustStmt, RustType, StdlibCode,
};
use sifr_hir::{
    lower_module, HirClass, HirClassKind, HirExceptHandler, HirExpr, HirFunction, HirImport,
    HirMatchArm, HirModule, HirParam, HirPattern, HirStmt, MethodKind,
};
use sifr_python_parser::parse_module;
use sifr_type_system::{ParamConvention, Type};
use std::collections::HashSet;

#[cfg(test)]
mod async_control_codegen_tests;
pub(crate) use async_control_codegen_tests::{empty_module, generate_rust_from_source};
#[cfg(test)]
mod async_runtime_codegen_tests;
#[cfg(test)]
mod classes_and_basics_codegen_tests;
#[cfg(test)]
mod collections_and_stdlib_codegen_tests;
#[cfg(test)]
mod iterators_and_generators_codegen_tests;
#[cfg(test)]
mod structured_lowering_codegen_tests;
