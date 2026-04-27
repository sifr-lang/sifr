use sifr_python_ast::Stmt;
use sifr_python_parser::parse_module;

pub(super) fn parse_suite(source: &str) -> Vec<Stmt> {
    let parsed = parse_module(source).unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert!(
        parsed.has_valid_syntax(),
        "invalid test source: {:?}",
        parsed.errors()
    );
    parsed.into_suite()
}
