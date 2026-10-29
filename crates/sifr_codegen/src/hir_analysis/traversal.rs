mod traversal_impl;
pub(crate) use traversal_impl::*;
#[cfg(test)]
#[path = "../../tests/unit/hir_analysis/traversal/tests.rs"]
mod tests;
