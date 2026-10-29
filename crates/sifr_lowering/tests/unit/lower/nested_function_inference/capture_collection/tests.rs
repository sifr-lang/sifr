
use super::*;
use sifr_python_parser::parse_module;

#[test]
fn nested_function_all_parameter_kinds_shadow_outer_names() {
    let parsed = parse_module(
            "def outer() -> None:\n    def inner(pos, /, regular, *items, keyword, **extras):\n        return (pos, regular, items, keyword, extras, captured)\n    return None\n",
        )
        .expect("source should parse");
    let Stmt::FunctionDef(outer) = &parsed.suite()[0] else {
        panic!("outer function missing");
    };
    let mut names = HashSet::new();
    collect_referenced_names_in_stmts(&outer.body, &mut names);

    assert_eq!(names, HashSet::from([String::from("captured")]));
}
