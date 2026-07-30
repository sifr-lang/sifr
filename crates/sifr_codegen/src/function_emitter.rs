use crate::NestedFnCapture;
use crate::{
    body_contains_yield, collect_mutated_vars_with_sigs, RustEmitter, RustExpr, RustItem,
    RustLiteral, RustParam, RustStmt, RustType, RustTypeParam, Visibility,
};
use crate::{
    helpers::{
        collect_locally_defined_vars, collect_reassigned_vars, collect_referenced_vars_with_types,
    },
    hir_analysis::traversal::{self, TraversalConfig},
};
use sifr_ir::{HirExpr, HirFunction, HirModule, HirParam, HirStmt};
use sifr_type_system::{make_union, OwnershipKind, ParamConvention, Type};
use std::collections::{HashMap, HashSet};

mod generator_bodies;
mod local_binding_registry;
mod nested_function_block;
mod python_callback_bounds;
mod scope_and_function_types;
mod sifr_int_analysis;
pub(crate) use sifr_int_analysis::*;
#[cfg(test)]
mod tests;
