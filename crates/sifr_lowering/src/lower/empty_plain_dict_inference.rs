use super::declaration_hint_safety::safe_direct_assignment_names;
use super::statements::empty_collection_literal_kind;
use sifr_python_ast::Stmt;
use std::collections::HashSet;

pub(in crate::lower) fn safe_hint_names_for_block(stmts: &[Stmt]) -> HashSet<String> {
    safe_direct_assignment_names(stmts, |value| {
        empty_collection_literal_kind(value) == Some("dict")
    })
}
