use std::collections::HashMap;

use crate::{lower_module, lower_module_with_externals, ExternalDefs, LoweringError};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

fn lower_errors(source: &str) -> Vec<LoweringError> {
    let parsed = parse_module(source).expect("parse failed");
    match lower_module(parsed.suite()) {
        Ok(_) => panic!("expected lowering error"),
        Err(errors) => errors,
    }
}

#[test]
fn undefined_variable_has_name_code() {
    let errors = lower_errors("def main():\n    print(x)\n");

    assert!(errors.iter().any(|error| {
        error.message == "undefined variable: 'x'"
            && error.code == Some(DiagnosticCode::NAME_UNDEFINED_VARIABLE)
    }));
}

#[test]
fn undefined_function_has_name_code() {
    let errors = lower_errors("def main():\n    foo()\n");

    assert!(errors.iter().any(|error| {
        error.message == "undefined function: 'foo'"
            && error.code == Some(DiagnosticCode::NAME_UNDEFINED_CALLABLE)
    }));
}

#[test]
fn missing_stdlib_member_has_name_code() {
    let parsed = parse_module("from local_math import nonexistent_func\n\ndef main():\n    pass\n")
        .expect("parse failed");
    let mut externals = ExternalDefs::default();
    externals
        .functions
        .insert("local_math".to_string(), HashMap::new());
    let errors = match lower_module_with_externals(parsed.suite(), &externals) {
        Ok(_) => panic!("expected lowering error"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.message == "module 'local_math' has no member 'nonexistent_func'"
            && error.code == Some(DiagnosticCode::NAME_MISSING_MODULE_MEMBER)
    }));
}

#[test]
fn forbidden_intrinsic_import_has_import_code() {
    let errors = lower_errors("from _sifr.io import read_text\n\ndef main():\n    pass\n");

    assert!(errors.iter().any(|error| {
        error.message
            == "cannot import from '_sifr.io' — _sifr.* modules are internal compiler intrinsics"
            && error.code == Some(DiagnosticCode::IMPORT_FORBIDDEN_INTRINSIC)
    }));
}

#[test]
fn unknown_module_import_has_import_code() {
    let errors = lower_errors("from missing_module import value\n\ndef main():\n    pass\n");

    assert!(errors.iter().any(|error| {
        error.message == "unknown import target: 'missing_module'"
            && error.code == Some(DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE)
    }));
}
