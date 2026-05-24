use crate::RustEmitter;
use sifr_hir::HirExpr;
use sifr_type_system::Type;

mod field_and_stdlib_rewrites;
mod operator_rewrites;
mod sifr_int_parse_helpers;
use sifr_int_parse_helpers::{
    is_legacy_i64_type, is_proven_nonzero_integer_expr, is_result_legacy_i64_type,
    is_sifr_int_arithmetic_op, is_sifr_int_checked_floor_op, is_sifr_int_comparison_op,
    is_sifr_int_operand_coercion_op, result_i64_type_to_sifr_int, rust_expr_identifier_path,
};
#[cfg(test)]
mod tests;
