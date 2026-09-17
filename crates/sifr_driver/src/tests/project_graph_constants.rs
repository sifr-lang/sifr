use super::*;

#[test]
fn test_collect_project_modules_exports_local_constants() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from consumer import get

def main():
    print(get())
"#,
        ),
    );
    parsed_modules.insert(
        "consumer".to_string(),
        parse_suite(
            r#"
from constants_mod import ANSWER

def get() -> int:
    return ANSWER
"#,
        ),
    );
    parsed_modules.insert(
        "constants_mod".to_string(),
        parse_suite(
            r#"
ANSWER: int = 42
"#,
        ),
    );

    let stdlib_defs =
        external_defs(&crate::CompilerContext::for_test()).expect("stdlib should compile");
    let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
        .expect("project lowering should resolve local constant imports");
    let constants = result
        .external_defs
        .constants
        .get("constants_mod")
        .expect("constants module exports should exist");
    assert_eq!(constants.get("ANSWER"), Some(&Type::Int));
    let constant_values = result
        .external_defs
        .constant_integer_values
        .get("constants_mod")
        .expect("integer constant values should be exported");
    assert_eq!(
        constant_values
            .get("ANSWER")
            .map(std::string::ToString::to_string),
        Some("42".to_string())
    );
}

#[test]
fn test_project_lowering_fits_imported_integer_constants() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from constants_mod import BASE as LIMIT

def main() -> uint8:
    value: uint8 = LIMIT + 1
    return value
"#,
        ),
    );
    parsed_modules.insert(
        "constants_mod".to_string(),
        parse_suite(
            r#"
BASE: int = 250 + 4
"#,
        ),
    );

    let stdlib_defs =
        external_defs(&crate::CompilerContext::for_test()).expect("stdlib should compile");
    let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
        .expect("project lowering should fit imported integer constants");
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
    assert!(matches!(value, HirExpr::IntLiteral(255)));
}

#[test]
fn test_project_lowering_does_not_fold_shadowed_imported_integer_constant() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from constants_mod import BASE

def main():
    BASE: int = 100
    value: uint8 = BASE + 1
"#,
        ),
    );
    parsed_modules.insert(
        "constants_mod".to_string(),
        parse_suite(
            r#"
BASE: int = 254
"#,
        ),
    );

    let stdlib_defs =
        external_defs(&crate::CompilerContext::for_test()).expect("stdlib should compile");
    let Err(errors) = collect_project_hir_modules(&parsed_modules, stdlib_defs) else {
        panic!("shadowed imported integer constant should not fit");
    };
    assert!(errors.iter().any(|error| {
        error.code == DiagnosticCode::TYPE_MISMATCH.code()
            && error
                .message
                .contains("[main] type mismatch: expected 'uint8', got 'int'")
    }));
}
