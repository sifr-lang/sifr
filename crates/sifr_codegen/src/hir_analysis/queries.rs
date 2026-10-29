mod queries_impl;
pub(crate) use queries_impl::*;
mod value_liveness;
pub(crate) use value_liveness::*;
mod checked_place_reads;
pub(crate) use checked_place_reads::*;
#[cfg(test)]
#[path = "../../tests/unit/hir_analysis/queries/default_expression_tests.rs"]
mod default_expression_tests;
#[cfg(test)]
#[path = "../../tests/unit/hir_analysis/queries/python_context_flow_tests.rs"]
mod python_context_flow_tests;
#[cfg(test)]
#[path = "../../tests/unit/hir_analysis/queries/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "../../tests/unit/hir_analysis/queries/type_and_operator_tests.rs"]
mod type_and_operator_tests;
