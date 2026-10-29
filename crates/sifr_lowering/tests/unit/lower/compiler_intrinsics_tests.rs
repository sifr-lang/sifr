use crate::{
    CompilerIntrinsicId, ExternalDefs, HirExpr, HirStmt,
    lower_module_sysroot_private_declaration_with_externals, lower_module_sysroot_public_stdlib,
    lower_module_with_externals,
};
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

fn public_module(source: &str) -> crate::HirModule {
    let parsed = parse_module(source).expect("source should parse");
    lower_module_sysroot_public_stdlib(parsed.suite())
        .expect("public sysroot declaration should lower")
        .module
}

fn errors<T>(result: Result<T, Vec<crate::HirDiagnostic>>) -> Vec<crate::HirDiagnostic> {
    match result {
        Ok(_) => panic!("lowering unexpectedly succeeded"),
        Err(errors) => errors,
    }
}

fn source_range(source: &str, needle: &str) -> TextRange {
    let start = source.find(needle).expect("needle should exist");
    TextRange::new(
        TextSize::try_from(start).expect("test source offset fits in TextSize"),
        TextSize::try_from(start + needle.len()).expect("test source offset fits in TextSize"),
    )
}

fn assert_structured_rejection(errors: &[crate::HirDiagnostic], needle: &str) {
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM)
            && error.message.contains(needle)
    }));
}

#[test]
fn canonical_public_declaration_records_typed_identity_and_no_body() {
    let module = public_module(
        "@compiler_intrinsic(test_assert_equal)\ndef assert_eq[T](actual: T, expected: T) -> None:\n    ...\n",
    );
    let function = &module.functions[0];
    assert_eq!(
        function.compiler_intrinsic,
        Some(CompilerIntrinsicId::TestAssertEqual)
    );
    assert!(function.body.is_empty());
}

#[test]
fn user_and_private_sysroot_declarations_are_rejected() {
    let source =
        "@compiler_intrinsic(test_assert_true)\ndef assert_true(value: bool) -> None:\n    ...\n";
    let parsed = parse_module(source).expect("source should parse");
    let user_errors = errors(lower_module_with_externals(
        parsed.suite(),
        &ExternalDefs::default(),
    ));
    assert_structured_rejection(&user_errors, "reserved for canonical public sysroot");

    let private_errors = errors(lower_module_sysroot_private_declaration_with_externals(
        parsed.suite(),
        &ExternalDefs::default(),
    ));
    assert_structured_rejection(&private_errors, "reserved for canonical public sysroot");
}

#[test]
fn malformed_unknown_synthesized_and_runtime_body_declarations_are_rejected() {
    for (source, needle) in [
        (
            "@compiler_intrinsic\ndef f(value: bool) -> None:\n    ...\n",
            "must be a call",
        ),
        (
            "@compiler_intrinsic(not_an_id)\ndef f(value: bool) -> None:\n    ...\n",
            "unknown compiler intrinsic",
        ),
        (
            "@compiler_intrinsic(bytes_from_hex)\ndef f(value: str) -> bytes:\n    ...\n",
            "synthesized by lowering",
        ),
        (
            "@compiler_intrinsic(test_assert_true)\ndef f(value: bool) -> None:\n    return None\n",
            "exactly one ellipsis",
        ),
    ] {
        let parsed = parse_module(source).expect("source should parse");
        let diagnostics = errors(lower_module_sysroot_public_stdlib(parsed.suite()));
        assert_structured_rejection(&diagnostics, needle);
    }
}

#[test]
fn imported_alias_call_preserves_identity_and_source_ranges() {
    let source =
        "from sifr.test import assert_eq as same\n\ndef check() -> None:\n    same(10, 20)\n";
    let parsed = parse_module(source).expect("source should parse");
    let mut externals = ExternalDefs::default();
    externals
        .functions
        .entry("sifr.test".to_string())
        .or_default()
        .insert(
            "assert_eq".to_string(),
            sifr_type_system::FunctionType {
                receiver: None,
                params: vec![
                    (
                        "actual".to_string(),
                        sifr_type_system::Type::Int,
                        sifr_type_system::ParamConvention::borrow(),
                    ),
                    (
                        "expected".to_string(),
                        sifr_type_system::Type::Int,
                        sifr_type_system::ParamConvention::borrow(),
                    ),
                ],
                return_type: Box::new(sifr_type_system::Type::None),
            },
        );
    externals
        .compiler_intrinsics
        .entry("sifr.test".to_string())
        .or_default()
        .insert(
            "assert_eq".to_string(),
            CompilerIntrinsicId::TestAssertEqual,
        );

    let module = lower_module_with_externals(parsed.suite(), &externals)
        .expect("alias call should lower")
        .module;
    let HirStmt::Expr {
        expr:
            HirExpr::IntrinsicCall {
                intrinsic,
                call_range,
                arg_ranges,
                ..
            },
    } = &module.functions[0].body[0]
    else {
        panic!("expected typed intrinsic call")
    };
    assert_eq!(*intrinsic, CompilerIntrinsicId::TestAssertEqual);
    assert_eq!(*call_range, source_range(source, "same(10, 20)"));
    assert_eq!(
        arg_ranges,
        &vec![source_range(source, "10"), source_range(source, "20")]
    );
}

#[test]
fn source_declared_intrinsic_is_not_a_first_class_value() {
    let source = "from sifr.test import assert_true as verify\n\ndef capture() -> None:\n    value = verify\n";
    let parsed = parse_module(source).expect("source should parse");
    let mut externals = ExternalDefs::default();
    externals
        .functions
        .entry("sifr.test".to_string())
        .or_default()
        .insert(
            "assert_true".to_string(),
            sifr_type_system::FunctionType {
                receiver: None,
                params: vec![(
                    "value".to_string(),
                    sifr_type_system::Type::Bool,
                    sifr_type_system::ParamConvention::borrow(),
                )],
                return_type: Box::new(sifr_type_system::Type::None),
            },
        );
    externals
        .compiler_intrinsics
        .entry("sifr.test".to_string())
        .or_default()
        .insert(
            "assert_true".to_string(),
            CompilerIntrinsicId::TestAssertTrue,
        );
    let diagnostics = errors(lower_module_with_externals(parsed.suite(), &externals));
    assert_structured_rejection(&diagnostics, "may only be used as a direct call");
}

#[test]
fn imported_former_intrinsic_name_without_metadata_remains_an_ordinary_call() {
    let source =
        "from helper import assert_eq\n\ndef check() -> int:\n    return assert_eq(10, 20)\n";
    let parsed = parse_module(source).expect("source should parse");
    let mut externals = ExternalDefs::default();
    externals
        .functions
        .entry("helper".to_string())
        .or_default()
        .insert(
            "assert_eq".to_string(),
            sifr_type_system::FunctionType {
                receiver: None,
                params: vec![
                    (
                        "left".to_string(),
                        sifr_type_system::Type::Int,
                        sifr_type_system::ParamConvention::borrow(),
                    ),
                    (
                        "right".to_string(),
                        sifr_type_system::Type::Int,
                        sifr_type_system::ParamConvention::borrow(),
                    ),
                ],
                return_type: Box::new(sifr_type_system::Type::Int),
            },
        );
    let module = lower_module_with_externals(parsed.suite(), &externals)
        .expect("ordinary imported collision should lower")
        .module;
    assert!(matches!(
        &module.functions[0].body[0],
        HirStmt::Return {
            value: Some(HirExpr::Call { func, .. })
        } if func == "assert_eq"
    ));
}

#[test]
fn local_function_declaration_shadows_unaliased_imported_intrinsic_identity() {
    let source = "from sifr.test import assert_eq\n\ndef assert_eq(actual: int, expected: int) -> None:\n    return None\n\ndef check() -> None:\n    assert_eq(10, 20)\n";
    let parsed = parse_module(source).expect("source should parse");
    let mut externals = ExternalDefs::default();
    let signature = sifr_type_system::FunctionType {
        receiver: None,
        params: vec![
            (
                "actual".to_string(),
                sifr_type_system::Type::Int,
                sifr_type_system::ParamConvention::borrow(),
            ),
            (
                "expected".to_string(),
                sifr_type_system::Type::Int,
                sifr_type_system::ParamConvention::borrow(),
            ),
        ],
        return_type: Box::new(sifr_type_system::Type::None),
    };
    externals
        .functions
        .entry("sifr.test".to_string())
        .or_default()
        .insert("assert_eq".to_string(), signature);
    externals
        .compiler_intrinsics
        .entry("sifr.test".to_string())
        .or_default()
        .insert(
            "assert_eq".to_string(),
            CompilerIntrinsicId::TestAssertEqual,
        );

    let module = lower_module_with_externals(parsed.suite(), &externals)
        .expect("local declaration should shadow imported compiler identity")
        .module;
    assert!(matches!(
        &module.functions[1].body[0],
        HirStmt::Expr {
            expr: HirExpr::Call { func, .. }
        } if func == "assert_eq"
    ));
}
