use super::diagnostics::{
    collect_enum_variants, get_newtype_inner, has_decorator, is_enum_class, is_operator_dunder,
    is_protocol_class,
};
use super::protocol_diagnostics;
use super::statements::lower_function_stmts;
use super::typing_and_functions::{function_body_contains_yield, resolve_annotation_expr};
use super::{LowerCtx, parse_typevar_bound_expr};
use super::{
    async_await, class_field_inference, diagnostics, simple_expr, str, typing_and_functions,
};
use crate::hir_nodes::{
    HirClass, HirClassKind, HirExpr, HirFunction, HirParam, HirPattern, HirStmt,
    HirTupleTargetBinding, MethodKind,
};
use ruff_text_size::Ranged;
use sifr_python_ast::{Expr, Stmt, StmtClassDef};
use sifr_type_system::{FunctionType, ParamConvention, Type};

mod class_declaration_diagnostics;
mod class_type_collection;
use class_declaration_diagnostics::{
    missing_method_param_annotation, unsupported_class_declaration,
};
mod error_message_contract;
pub(in crate::lower) use class_type_collection::*;
pub(in crate::lower) use error_message_contract::root_error_type;
mod class_iteration_protocol;
use class_iteration_protocol::validate_iteration_protocol_methods;
mod class_semantics;
mod class_shape_metadata;
mod class_type_helpers;
pub(in crate::lower) use class_semantics::*;
mod class_body_lowering;
pub(in crate::lower) use class_body_lowering::*;
mod parameter_conventions;
pub(in crate::lower) use parameter_conventions::fixed_trait_receiver_convention;
mod python_cleanup_validation;
mod rust_opaque_validation;
