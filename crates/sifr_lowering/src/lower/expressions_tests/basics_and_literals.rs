use super::*;
use crate::lower::simple_expr::lower_expr_simple;

#[test]
pub(super) fn test_simple_function() {
    let module = lower_source("def add(a: int, b: int) -> int:\n    return a + b\n").unwrap();
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "add");
    assert_eq!(module.functions[0].return_type, Type::Int);
}

#[test]
pub(super) fn test_large_integer_literals_lower_losslessly_from_source() {
    let source = "\
def main():
    decimal = 9223372036854775808
    decimal_big = 184467440737095516160
    decimal_underscored = 1_000_000_000_000_000_000_000
    hex_mid = 0xffffffffffffffff
    hexed = 0x10000000000000000
    hexed_underscored = 0xFFFF_FFFF_FFFF_FFFF_FFFF
    octaled = 0o2000000000000000000000
    binaried = 0b10000000000000000000000000000000000000000000000000000000000000000
";
    let module = lower_source(source).expect("large integer literals should lower");

    let expected = [
        ("decimal", "9223372036854775808"),
        ("decimal_big", "184467440737095516160"),
        ("decimal_underscored", "1000000000000000000000"),
        ("hex_mid", "18446744073709551615"),
        ("hexed", "18446744073709551616"),
        ("hexed_underscored", "1208925819614629174706175"),
        ("octaled", "18446744073709551616"),
        ("binaried", "18446744073709551616"),
    ];

    for (name, literal_text) in expected {
        match function_let_value(&module, name) {
            HirExpr::LargeIntLiteral(actual) => assert_eq!(actual, literal_text),
            other => panic!("expected large integer literal for {name}, got {other:?}"),
        }
    }
}

#[test]
pub(super) fn test_negative_large_integer_literal_lowers_as_unary_large_literal() {
    let source = "def main():\n    value = -9_223_372_036_854_775_809\n";
    let module = lower_source(source).expect("negative large integer literal should lower");

    match function_let_value(&module, "value") {
        HirExpr::UnaryOp { op, operand, ty } => {
            assert_eq!(op, "-");
            assert_eq!(ty, &Type::Int);
            assert!(
                matches!(operand.as_ref(), HirExpr::LargeIntLiteral(value) if value == "9223372036854775809"),
                "expected unary large integer operand, got {operand:?}",
            );
        }
        other => panic!("expected unary large integer literal, got {other:?}"),
    }
}

#[test]
pub(super) fn test_large_integer_default_arguments_lower_losslessly() {
    let source = "\
def identity(
    x: int = 9223372036854775808,
    y: int = -0x8000000000000001,
) -> int:
    return x
";
    let module = lower_source(source).expect("large integer defaults should lower");
    let params = &module.functions[0].params;

    assert!(
        matches!(params[0].default.as_ref(), Some(HirExpr::LargeIntLiteral(value)) if value == "9223372036854775808"),
        "expected positive large integer default, got {:?}",
        params[0].default
    );
    match params[1].default.as_ref() {
        Some(HirExpr::UnaryOp { op, operand, ty }) => {
            assert_eq!(op, "-");
            assert_eq!(ty, &Type::Int);
            assert!(
                matches!(operand.as_ref(), HirExpr::LargeIntLiteral(value) if value == "9223372036854775809"),
                "expected negative large integer default operand, got {operand:?}",
            );
        }
        other => panic!("expected negative large integer default, got {other:?}"),
    }
}

#[test]
pub(super) fn test_large_integer_literal_over_budget_has_int_code() {
    let literal = "1".repeat(4097);
    let source = format!("def main():\n    value = {literal}\n");
    let result = lower_source(&source);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::INT_EVAL_BUDGET_EXCEEDED)
            && error.message
                == "integer literal exceeds compile-time evaluation budget: 4097 decimal digits (max 4096)"
            && error.primary_range == Some(range_for(&source, &literal))
    }));
}

#[test]
pub(super) fn test_constructed_and_parsed_large_integer_literals_match_hir() {
    let token = "0xFFFF_FFFF_FFFF_FFFF_FFFF";
    let constructed_int =
        Int::from_str_radix("FFFFFFFFFFFFFFFFFFFF", 16, token).expect("valid hex literal");
    let constructed_expr = Expr::NumberLiteral(ExprNumberLiteral {
        node_index: AtomicNodeIndex::NONE,
        range: TextRange::default(),
        value: Number::Int(constructed_int),
    });
    let constructed_hir =
        lower_expr_simple(&constructed_expr).expect("constructed literal should lower");

    let source = "def main():\n    value = 0xFFFF_FFFF_FFFF_FFFF_FFFF\n";
    let parsed_module = lower_source(source).expect("parsed literal should lower");
    let parsed_hir = function_let_value(&parsed_module, "value");

    match (&constructed_hir, parsed_hir) {
        (HirExpr::LargeIntLiteral(constructed), HirExpr::LargeIntLiteral(parsed)) => {
            assert_eq!(constructed, parsed);
            assert_eq!(parsed, "1208925819614629174706175");
        }
        other => panic!("expected matching large integer HIR literals, got {other:?}"),
    }
}

#[test]
pub(super) fn test_type_mismatch_error() {
    let source = "def main():\n    x: int = \"hello\"\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("type mismatch")
        && e.code == Some(DiagnosticCode::TYPE_MISMATCH)
        && e.primary_range == Some(range_for(source, "\"hello\""))));
}

#[test]
pub(super) fn test_fixed_width_literal_assignment_fits() {
    let module = lower_source(
        "def main() -> uint8:\n    value: uint8 = 255\n    signed: int8 = -128\n    wide: uint64 = 18446744073709551615\n    return value\n",
    )
    .expect("fitting fixed-width literal assignments should lower");

    let main_fn = &module.functions[0];
    let HirStmt::Let { ty, value, .. } = &main_fn.body[0] else {
        panic!("expected first statement to be uint8 let");
    };
    assert_eq!(ty, &Type::FixedInt(FixedIntType::U8));
    assert!(matches!(value, HirExpr::IntLiteral(255)));

    let HirStmt::Let { ty, value, .. } = &main_fn.body[1] else {
        panic!("expected second statement to be int8 let");
    };
    assert_eq!(ty, &Type::FixedInt(FixedIntType::I8));
    assert!(matches!(value, HirExpr::IntLiteral(-128)));

    let HirStmt::Let { ty, value, .. } = &main_fn.body[2] else {
        panic!("expected third statement to be uint64 let");
    };
    assert_eq!(ty, &Type::FixedInt(FixedIntType::U64));
    assert!(matches!(value, HirExpr::LargeIntLiteral(value) if value == "18446744073709551615"));
}

#[test]
pub(super) fn test_fixed_width_literal_assignment_out_of_range_has_int_code() {
    let source = "def main():\n    too_wide: uint8 = 256\n    negative_unsigned: uint8 = -1\n    signed_high: int8 = 128\n    signed_low: int8 = -129\n";
    let errors = lower_source(source).expect_err("out-of-range fixed-width literals should fail");

    for (needle, target, min, max, value) in [
        ("256", "uint8", "0", "255", "256"),
        ("-1", "uint8", "0", "255", "-1"),
        ("128", "int8", "-128", "127", "128"),
        ("-129", "int8", "-128", "127", "-129"),
    ] {
        assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::INT_FIXED_WIDTH_OUT_OF_RANGE)
                && error.message
                    == format!(
                        "integer value {value} does not fit target type {target}; valid range is {min}..={max}"
                    )
                && error.primary_range == Some(range_for(source, needle))
        }));
    }
    assert!(
        errors
            .iter()
            .all(|error| error.code != Some(DiagnosticCode::TYPE_MISMATCH)),
        "range diagnostics should not be followed by generic type mismatches: {errors:?}"
    );
}

#[test]
pub(super) fn test_fixed_width_match_literal_out_of_range_has_int_code() {
    let source = "def main():\n    value: uint8 = 1\n    match value:\n        case 256:\n            pass\n        case _:\n            pass\n";
    let errors =
        lower_source(source).expect_err("out-of-range fixed-width match literal should fail");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::INT_FIXED_WIDTH_OUT_OF_RANGE)
            && error.message
                == "integer value 256 does not fit target type uint8; valid range is 0..=255"
            && error.primary_range == Some(range_for(source, "256"))
    }));
    assert!(
        errors
            .iter()
            .all(|error| error.code == Some(DiagnosticCode::INT_FIXED_WIDTH_OUT_OF_RANGE)),
        "expected only fixed-width range diagnostics, got {errors:?}"
    );
}

#[test]
pub(super) fn test_fixed_width_match_literal_in_range_lowers() {
    lower_source(
        "def main():\n    value: uint8 = 255\n    match value:\n        case 255:\n            pass\n        case _:\n            pass\n",
    )
    .expect("in-range fixed-width match literal should lower");
}

#[test]
pub(super) fn test_fixed_width_module_constant_out_of_range_has_int_code() {
    let source = "LIMIT: uint8 = 255\nTOO_HIGH: uint8 = 256\n\ndef main():\n    print(\"ok\")\n";
    let errors =
        lower_source(source).expect_err("out-of-range fixed-width module constant should fail");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::INT_FIXED_WIDTH_OUT_OF_RANGE)
            && error.message
                == "integer value 256 does not fit target type uint8; valid range is 0..=255"
            && error.primary_range == Some(range_for(source, "256"))
    }));
}

#[test]
pub(super) fn test_fixed_width_const_expression_assignment_fits_and_folds() {
    let module = lower_source(
        "def main() -> uint8:\n    value: uint8 = (1 + 2) * 40 + (20 >> 1)\n    shifted: uint8 = (1 << 6) + (9 // 2) + (9 % 2)\n    signed: int8 = -10 * 5\n    negated: int16 = -(100 + 27)\n    return value\n",
    )
    .expect("fitting fixed-width const expression assignment should lower");

    let main_fn = &module.functions[0];
    let HirStmt::Let { ty, value, .. } = &main_fn.body[0] else {
        panic!("expected uint8 let");
    };
    assert_eq!(ty, &Type::FixedInt(FixedIntType::U8));
    assert!(matches!(value, HirExpr::IntLiteral(130)));

    let HirStmt::Let { ty, value, .. } = &main_fn.body[1] else {
        panic!("expected shifted uint8 let");
    };
    assert_eq!(ty, &Type::FixedInt(FixedIntType::U8));
    assert!(matches!(value, HirExpr::IntLiteral(69)));

    let HirStmt::Let { ty, value, .. } = &main_fn.body[2] else {
        panic!("expected signed int8 let");
    };
    assert_eq!(ty, &Type::FixedInt(FixedIntType::I8));
    assert!(matches!(value, HirExpr::IntLiteral(-50)));

    let HirStmt::Let { ty, value, .. } = &main_fn.body[3] else {
        panic!("expected negated int16 let");
    };
    assert_eq!(ty, &Type::FixedInt(FixedIntType::I16));
    assert!(matches!(value, HirExpr::IntLiteral(-127)));
}

#[test]
pub(super) fn test_exact_int_division_by_unproven_divisor_requires_result_target() {
    let source = "\
def main(divisor: int) -> None:
    value: int = 10 // divisor
";
    let errors =
        lower_source(source).expect_err("unproven exact-int divisor should produce Result[int]");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error
                .message
                .contains("expected 'int', got 'Result[int, DivisionError]'")
            && error.primary_range == Some(range_for(source, "10 // divisor"))
    }));
}

#[test]
pub(super) fn test_exact_int_division_by_unproven_divisor_lowers_as_result() {
    let module = lower_source(
        "\
def main(divisor: int) -> None:
    value: Result[int, DivisionError] = 10 // divisor
",
    )
    .expect("unproven exact-int divisor should lower as Result[int, DivisionError]");

    let HirStmt::Let { value, .. } = &module.functions[0].body[0] else {
        panic!("expected result let");
    };
    assert!(matches!(
        value,
        HirExpr::BinOp {
            ty: Type::Result(ok, err),
            ..
        } if matches!(ok.as_ref(), Type::Int)
            && matches!(err.as_ref(), Type::Class { name, .. } if name == "DivisionError")
    ));
}

#[test]
pub(super) fn test_exact_int_true_division_requires_result_target() {
    let source = "\
def main(numerator: int, denominator: int) -> None:
    value: float = numerator / denominator
";
    let errors = lower_source(source).expect_err("exact-int true division should be fallible");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message.contains(
                "expected 'float', got 'Result[float, DivisionError | FloatOverflowError | FloatPrecisionLossError]'",
            )
            && error.primary_range == Some(range_for(source, "numerator / denominator"))
    }));
}

#[test]
pub(super) fn test_proven_safe_exact_int_true_division_lowers_as_float() {
    let module = lower_source(
        "\
def main() -> None:
    numerator: int = 10
    denominator: int = 3
    value: float = numerator / denominator
",
    )
    .expect("small exact-int constants should prove safe for true division");

    assert!(matches!(
        function_let_value(&module, "value"),
        HirExpr::BinOp {
            ty: Type::Float,
            ..
        }
    ));
}

#[test]
pub(super) fn test_large_exact_int_true_division_still_requires_handling() {
    let source = "\
def main() -> None:
    numerator: int = 9007199254740993
    denominator: int = 3
    value: float = numerator / denominator
";
    let errors = lower_source(source).expect_err("precision-losing exact-int division should fail");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error
                .message
                .contains("expected 'float', got 'Result[float,")
            && error.primary_range == Some(range_for(source, "numerator / denominator"))
    }));
}

#[test]
pub(super) fn test_exact_int_true_division_branch_reassignment_does_not_leak_const_proof() {
    let source = "\
def main(flag: bool) -> None:
    numerator: int = 10
    denominator: int = 3
    if flag:
        numerator = 9007199254740993
    value: float = numerator / denominator
";
    let errors = lower_source(source).expect_err("branch-dependent int should fail closed");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error
                .message
                .contains("expected 'float', got 'Result[float,")
            && error.primary_range == Some(range_for(source, "numerator / denominator"))
    }));
}

#[test]
pub(super) fn test_exact_int_true_division_augassign_reassignment_does_not_leak_const_proof() {
    let source = "\
def main(delta: int) -> None:
    numerator: int = 10
    denominator: int = 3
    numerator += delta
    value: float = numerator / denominator
";
    let errors = lower_source(source).expect_err("augassigned int should fail closed");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error
                .message
                .contains("expected 'float', got 'Result[float,")
            && error.primary_range == Some(range_for(source, "numerator / denominator"))
    }));
}

#[test]
pub(super) fn test_exact_int_true_division_loop_reassignment_does_not_leak_const_proof() {
    let source = "\
def main(items: list[int]) -> None:
    numerator: int = 10
    denominator: int = 3
    for item in items:
        numerator = item
    value: float = numerator / denominator
";
    let errors = lower_source(source).expect_err("loop-dependent int should fail closed");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error
                .message
                .contains("expected 'float', got 'Result[float,")
            && error.primary_range == Some(range_for(source, "numerator / denominator"))
    }));
}

#[test]
pub(super) fn test_exact_int_true_division_optional_narrowed_consts_lower_as_float() {
    let module = lower_source(
        "\
def main() -> None:
    total: int | None = 9
    count: int | None = 3
    if total is not None:
        if count is not None:
            value: float = total / count
",
    )
    .expect("narrowed optional exact-int constants should prove safe for true division");

    assert!(matches!(
        function_nested_let_value(&module, "value"),
        HirExpr::BinOp {
            ty: Type::Float,
            ..
        }
    ));
}

#[test]
pub(super) fn test_exact_int_mod_augassign_by_unproven_divisor_has_int0005() {
    let source = "\
def main(divisor: int) -> None:
    value: int = 10
    value %= divisor
";
    let errors =
        lower_source(source).expect_err("unproven exact-int augassign divisor should fail");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::INT_EXACT_DIVISION_REQUIRES_HANDLING)
            && error.primary_range == Some(range_for_after(source, "value %= ", "divisor"))
    }));
}

#[test]
pub(super) fn test_exact_int_division_by_nonzero_literal_still_lowers_as_int() {
    let module = lower_source(
        "\
def main() -> None:
    value: int = 10 // 2
    remainder: int = 10 % -3
",
    )
    .expect("proven non-zero literal divisors should lower");

    assert!(matches!(
        function_let_value(&module, "value"),
        HirExpr::BinOp { ty: Type::Int, .. }
    ));
    assert!(matches!(
        function_let_value(&module, "remainder"),
        HirExpr::BinOp { ty: Type::Int, .. }
    ));
}

#[test]
pub(super) fn test_exact_int_augassign_by_nonzero_literal_still_lowers() {
    let module = lower_source(
        "\
def main() -> None:
    value: int = 28
    value //= 3
    value %= -5
",
    )
    .expect("proven non-zero literal augassign divisors should lower");

    let main_fn = &module.functions[0];
    assert!(matches!(
        &main_fn.body[1],
        HirStmt::AugAssign {
            name,
            op,
            value: HirExpr::IntLiteral(3),
        } if name == "value" && op == "//="
    ));
    assert!(matches!(
        &main_fn.body[2],
        HirStmt::AugAssign {
            name,
            op,
            value: HirExpr::UnaryOp { ty: Type::Int, .. },
        } if name == "value" && op == "%="
    ));
}

#[test]
pub(super) fn test_fixed_width_floor_division_by_nonzero_literal_promotes_exactly() {
    let module = lower_source(
        "\
def main() -> None:
    left: uint8 = 10
    value: int = left // 2
",
    )
    .expect("a fixed-width value divided by a proven nonzero literal is exact");

    assert!(matches!(
        function_let_value(&module, "value"),
        HirExpr::BinOp { ty: Type::Int, .. }
    ));
}

#[test]
pub(super) fn test_fixed_width_mod_augassign_requires_handling() {
    let source = "\
def main() -> None:
    value: uint8 = 10
    divisor: uint8 = 3
    value %= divisor
";
    let errors = lower_source(source).expect_err("fixed-width modulo augassign should fail closed");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::INT_EXACT_DIVISION_REQUIRES_HANDLING)
            && error.primary_range == Some(range_for_after(source, "value %= ", "divisor"))
    }));
}

#[test]
pub(super) fn test_exact_int_power_by_negative_literal_requires_result_target() {
    let source = "\
def main() -> None:
    value: int = 2 ** -1
";
    let errors = lower_source(source).expect_err("negative exact-int exponent should fail closed");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message.contains("expected 'int', got 'Result[int,")
            && error.primary_range == Some(range_for(source, "2 ** -1"))
    }));
}

#[test]
pub(super) fn test_exact_int_power_by_unproven_exponent_requires_result_target() {
    let source = "\
def main(exponent: int) -> None:
    value: int = 2 ** exponent
";
    let errors = lower_source(source).expect_err("unproven exact-int exponent should fail closed");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message.contains("expected 'int', got 'Result[int,")
            && error.primary_range == Some(range_for(source, "2 ** exponent"))
    }));
}

#[test]
pub(super) fn test_fixed_width_power_requires_result_target() {
    let source = "\
def main() -> None:
    base: uint8 = 2
    value: int = base ** 3
";
    let errors = lower_source(source).expect_err("fixed-width exponentiation should fail closed");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message.contains("expected 'int', got 'Result[int,")
            && error.primary_range == Some(range_for(source, "base ** 3"))
    }));
}

#[test]
pub(super) fn test_fixed_width_power_augassign_requires_handling() {
    let source = "\
def main() -> None:
    value: uint8 = 2
    value **= 3
";
    let errors =
        lower_source(source).expect_err("fixed-width exponentiation augassign should fail closed");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::INT_EXACT_DIVISION_REQUIRES_HANDLING)
            && error.primary_range == Some(range_for_after(source, "value **= ", "3"))
    }));
}

#[test]
pub(super) fn test_statically_bounded_exact_int_power_still_lowers() {
    let module = lower_source(
        "\
def main() -> None:
    value: int = 2 ** 3
    value **= 0
",
    )
    .expect("statically bounded exact-int exponentiation should still lower");

    assert!(matches!(
        function_let_value(&module, "value"),
        HirExpr::BinOp { ty: Type::Int, .. }
    ));
    assert!(matches!(
        &module.functions[0].body[1],
        HirStmt::AugAssign {
            name,
            op,
            value: HirExpr::IntLiteral(0),
        } if name == "value" && op == "**="
    ));
}

#[test]
pub(super) fn test_bool_integer_equality_has_int0007() {
    let source = "\
def main() -> None:
    value: bool = True == 1
";
    let errors = lower_source(source).expect_err("bool/integer equality should fail");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::INT_BOOL_INTEGER_COMPARISON)
            && error.message == "cannot compare bool and integer values without explicit conversion"
            && error.primary_range == Some(range_for_after(source, "True == ", "1"))
    }));
}

#[test]
pub(super) fn test_bool_fixed_width_ordering_has_int0007() {
    let source = "\
def main() -> None:
    value: uint8 = 1
    flag: bool = True
    result: bool = value < flag
";
    let errors = lower_source(source).expect_err("bool/fixed-width ordering should fail");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::INT_BOOL_INTEGER_COMPARISON)
            && error.primary_range == Some(range_for_after(source, "value < ", "flag"))
    }));
}

#[test]
pub(super) fn test_unbounded_generic_addition_requires_addable_bound() {
    let source = "\
def add_same[T](left: T, right: T) -> T:
    return left + right
";
    let errors = lower_source(source).expect_err("unbounded generic addition should fail");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
            && error.message
                == "generic addition on type parameter 'T' requires an Addable bound with output assignable to T"
            && error.primary_range == Some(range_for(source, "left + right"))
    }));
}

#[test]
pub(super) fn test_addable_generic_addition_accepts_int() {
    lower_source(
        "\
def add_same[T: Addable](left: T, right: T) -> T:
    return left + right

def main() -> None:
    value: int = add_same(1, 2)
",
    )
    .expect("Addable generic addition should accept exact int");
}
