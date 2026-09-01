use super::diagnostics::{format_type_name, is_valid_error_type};
use super::expressions::lower_expr;
use super::function_flow::{collect_yield_types, infer_function_return_type};
use super::nonlocal_support::collect_declared_nonlocals;
use super::ownership_diagnostics;
use super::statements::lower_function_stmts;
use super::workload_annotations;
use super::{LowerCtx, substitute_type_vars};
use super::{async_effects, flow_diagnostics, simple_expr, str};
use crate::hir_nodes::{HirFunction, HirParam, MethodKind};
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, Number, Operator, StmtFunctionDef};
use sifr_type_system::infer::resolve_type_annotation;
use sifr_type_system::{
    FunctionType, OwnershipKind, ParamConvention, PythonArrowKind, Type, make_union,
};
use std::collections::HashMap;

mod signatures_and_effects;
pub(in crate::lower) use signatures_and_effects::*;
mod python_buffer_annotations;
use python_buffer_annotations::resolve_python_buffer_annotation;
mod python_arrow_annotations;
use python_arrow_annotations::resolve_python_arrow_annotation;
mod async_generator_validation;
mod python_dlpack_annotations;
pub(in crate::lower) use async_generator_validation::{
    reject_declared_async_generator_boundary, reject_unsupported_nested_generator,
};
use python_dlpack_annotations::{
    resolve_python_dlpack_tensor_annotation, resolve_python_resource_attribute_annotation,
};
mod function_exit_validation;
use function_exit_validation::{
    reject_live_join_sets_at_function_exit, reject_live_must_use_bindings_at_function_exit,
};
mod annotations_and_function_lowering;
pub(in crate::lower) use annotations_and_function_lowering::*;
mod annotation_resolution;
pub(in crate::lower) use annotation_resolution::resolve_annotation_expr;
mod annotation_union_validation;
