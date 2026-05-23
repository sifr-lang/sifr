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
use sifr_hir::{HirExpr, HirFunction, HirModule, HirParam, HirStmt};
use sifr_type_system::{make_union, OwnershipKind, ParamConvention, Type};
use std::collections::{HashMap, HashSet};


include!("function_emitter/scope_and_function_types.rs");
include!("function_emitter/generator_bodies.rs");
include!("function_emitter/sifr_int_analysis.rs");
include!("function_emitter/tests.rs");
