use crate::{lower_module, HirModule, LoweringError};
use sifr_python_parser::parse_module;

fn lower_source(source: &str) -> Result<HirModule, Vec<LoweringError>> {
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).map(|result| result.module)
}

#[test]
fn test_forward_type_alias_resolves_independent_of_declaration_order() {
    let result = lower_source(
        "type Payload = Response\ntype Response = list[int]\n\ndef main():\n    data: Payload = [1, 2, 3]\n    print(len(data))\n",
    );
    assert!(
        result.is_ok(),
        "forward type aliases should resolve deterministically"
    );
}

#[test]
fn test_recursive_type_alias_name_resolves_without_unknown_type_error() {
    let result = lower_source(
        "type Json = None | bool | int | float | str | list[Json] | dict[str, Json]\n\ndef main():\n    print(\"ok\")\n",
    );
    assert!(
        result.is_ok(),
        "recursive alias names should be predeclared before alias body resolution"
    );
}

#[test]
fn test_unresolved_type_alias_dependency_still_errors() {
    let result = lower_source("type Payload = Missing\n\ndef main():\n    print(\"ok\")\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("unknown type: 'Missing'")),
        "missing type names outside the alias predeclaration set should still error"
    );
}
