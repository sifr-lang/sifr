use sifr_python_ast::Suite;
use sifr_syntax::parse_module_suite;

pub(super) fn parse_suite(source: &str) -> Suite {
    parse_module_suite(source, Some("driver test source"))
        .unwrap_or_else(|e| panic!("parse failed: {e:?}"))
}
