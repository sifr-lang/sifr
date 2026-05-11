use std::collections::HashMap;

use crate::{lower_module, lower_module_with_externals, ExternalDefs, HirDiagnostic};
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

fn lower_errors(source: &str) -> Vec<HirDiagnostic> {
    let parsed = parse_module(source).expect("parse failed");
    match lower_module(parsed.suite()) {
        Ok(_) => panic!("expected lowering error"),
        Err(errors) => errors,
    }
}

fn range_for(source: &str, needle: &str) -> TextRange {
    let start = source.find(needle).expect("needle should exist") as u32;
    TextRange::new(
        TextSize::new(start),
        TextSize::new(start + needle.len() as u32),
    )
}

#[test]
fn undefined_variable_has_name_code() {
    let source = "def main():\n    print(x)\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message == "undefined variable: 'x'"
            && error.code == Some(DiagnosticCode::NAME_UNDEFINED_VARIABLE)
            && error.primary_range == Some(range_for(source, "x"))
    }));
}

#[test]
fn undefined_function_has_name_code() {
    let source = "def main():\n    foo()\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message == "undefined function: 'foo'"
            && error.code == Some(DiagnosticCode::NAME_UNDEFINED_CALLABLE)
            && error.primary_range == Some(range_for(source, "foo"))
    }));
}

#[test]
fn missing_stdlib_member_has_name_code() {
    let source = "from local_math import nonexistent_func\n\ndef main():\n    pass\n";
    let parsed = parse_module(source).expect("parse failed");
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
            && error.primary_range == Some(range_for(source, "nonexistent_func"))
    }));
}

#[test]
fn deferred_stdlib_member_has_name_code() {
    let source = "from sifr.asyncio import get_event_loop_policy\n\ndef main():\n    pass\n";
    let parsed = parse_module(source).expect("parse failed");
    let mut externals = ExternalDefs::default();
    externals
        .functions
        .insert("sifr.asyncio".to_string(), HashMap::new());
    let errors = match lower_module_with_externals(parsed.suite(), &externals) {
        Ok(_) => panic!("expected lowering error"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.message == "'sifr.asyncio.get_event_loop_policy' is intentionally deferred: event loop policies are deferred; Sifr exposes structured task scopes instead"
            && error.code == Some(DiagnosticCode::NAME_MISSING_MODULE_MEMBER)
            && error.primary_range == Some(range_for(source, "get_event_loop_policy"))
    }));
}

#[test]
fn deferred_stdlib_module_has_import_code() {
    let source = "from sifr.contextvars import ContextVar\n\ndef main():\n    pass\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message
            == "module 'sifr.contextvars' is intentionally deferred: context-local state is deferred; pass task state explicitly"
            && error.code == Some(DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE)
            && error.primary_range == Some(range_for(source, "from sifr.contextvars import ContextVar"))
    }));
}

#[test]
fn forbidden_intrinsic_import_has_import_code() {
    let source = "from _sifr.io import read_text\n\ndef main():\n    pass\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message
            == "cannot import from '_sifr.io' — _sifr.* modules are internal compiler intrinsics"
            && error.code == Some(DiagnosticCode::IMPORT_FORBIDDEN_INTRINSIC)
            && error.primary_range == Some(range_for(source, "from _sifr.io import read_text"))
    }));
}

#[test]
fn unknown_module_import_has_import_code() {
    let source = "from missing_module import value\n\ndef main():\n    pass\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message == "unknown import target: 'missing_module'"
            && error.code == Some(DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE)
            && error.primary_range == Some(range_for(source, "from missing_module import value"))
    }));
}

#[test]
fn unsupported_import_statement_has_import_code() {
    let source = "import local_math\n\ndef main():\n    pass\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message
            == "unsupported import form: import local_math; use 'from local_math import <name>'"
            && error.code == Some(DiagnosticCode::IMPORT_UNSUPPORTED_FORM)
            && error.primary_range == Some(range_for(source, "local_math"))
    }));
}

#[test]
fn private_import_member_has_import_code() {
    let source = "from local_math import _secret\n\ndef main():\n    pass\n";
    let parsed = parse_module(source).expect("parse failed");
    let mut externals = ExternalDefs::default();
    externals
        .functions
        .insert("local_math".to_string(), HashMap::new());
    let errors = match lower_module_with_externals(parsed.suite(), &externals) {
        Ok(_) => panic!("expected lowering error"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.message == "cannot import private name '_secret' from module 'local_math'"
            && error.code == Some(DiagnosticCode::IMPORT_PRIVATE_MEMBER)
            && error.primary_range == Some(range_for(source, "_secret"))
    }));
}
