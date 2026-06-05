use super::support::parse_suite;
use crate::{collect_project_hir_modules, compile_stdlib};
use sifr_lowering::{HirExpr, HirStmt};
use std::collections::HashMap;

#[test]
fn stdlib_heapq_exports_allowlisted_private_max_heap_helpers() {
    let compiled = compile_stdlib().expect("stdlib should compile");
    let heapq_functions = compiled
        .defs
        .functions
        .get("sifr.heapq")
        .expect("sifr.heapq exports should exist");

    for name in ["_heapify_max", "_heappop_max", "_heapreplace_max"] {
        assert!(
            heapq_functions.contains_key(name),
            "expected sifr.heapq export '{name}' to be visible for compat imports"
        );
    }
}

#[test]
fn stdlib_integer_constants_fold_in_project_fixed_width_initializers() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from sifr.logging import DEBUG

def main() -> uint8:
    value: uint8 = DEBUG + 1
    return value
"#,
        ),
    );

    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
        .expect("project lowering should fit imported stdlib integer constants");
    let main_module = result
        .hir_modules
        .get("main")
        .expect("main module should lower");
    let main_fn = main_module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function should lower");
    let HirStmt::Let { ty, value, .. } = &main_fn.body[0] else {
        panic!("expected first statement to be fitted let");
    };
    assert_eq!(ty.display_name(), "uint8");
    assert!(matches!(value, HirExpr::IntLiteral(11)));
}
