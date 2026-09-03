use crate::NestedFnCapture;
use crate::{
    RustEmitter, RustExpr, RustItem, RustLiteral, RustParam, RustStmt, RustType, RustTypeParam,
    Visibility, body_contains_yield, collect_mutated_vars_with_sigs,
};
use crate::{
    helpers::{
        collect_locally_defined_vars, collect_reassigned_vars, collect_referenced_vars_with_types,
    },
    hir_analysis::traversal::{self, TraversalConfig},
};
use sifr_ir::{HirExpr, HirFunction, HirModule, HirParam, HirStmt};
use sifr_type_system::{OwnershipKind, ParamConvention, Type, make_union};
use std::collections::{HashMap, HashSet};

mod function_types;
mod generator_bodies;
mod generic_bounds;
mod local_binding_registry;
mod nested_function_block;
mod python_callback_bounds;
mod resumable_generator_bodies;
mod scope_and_function_types;
mod sifr_int_analysis;
pub(crate) use sifr_int_analysis::*;
#[cfg(test)]
mod tests;
