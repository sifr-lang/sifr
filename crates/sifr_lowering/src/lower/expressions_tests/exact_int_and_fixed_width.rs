use super::*;
#[test]
pub(super) fn test_exact_int_division_after_zero_guard_early_exit_lowers() {
    let module = lower_source(
        "\
def main() -> None:
    divisor: int = 3
    if divisor == 0:
        return
    value: int = 10 // divisor
",
    )
    .expect("early-exit zero guard should prove the divisor is non-zero after the if");

    assert!(matches!(
        function_let_value(&module, "value"),
        HirExpr::BinOp { ty: Type::Int, .. }
    ));
}

#[test]
pub(super) fn test_exact_int_modulo_inside_nonzero_while_guard_lowers() {
    let module = lower_source(
        "\
def main() -> None:
    divisor: int = 3
    while divisor != 0:
        value: int = 10 % divisor
        divisor = 0
",
    )
    .expect("while non-zero guard should prove the divisor is non-zero in the body");

    let HirStmt::While { body, .. } = &module.functions[0].body[1] else {
        panic!("expected while statement");
    };
    let HirStmt::Let { name, value, .. } = &body[0] else {
        panic!("expected value binding inside while body");
    };
    assert_eq!(name, "value");
    assert!(matches!(value, HirExpr::BinOp { ty: Type::Int, .. }));
}

#[test]
pub(super) fn test_exact_int_division_after_elif_zero_guard_early_exit_lowers() {
    let module = lower_source(
        "\
def main(flag: bool) -> None:
    divisor: int = 3
    if flag:
        return
    elif divisor == 0:
        return
    value: int = 10 // divisor
",
    )
    .expect("elif early-exit zero guard should prove the divisor is non-zero after the chain");

    assert!(matches!(
        function_let_value(&module, "value"),
        HirExpr::BinOp { ty: Type::Int, .. }
    ));
}

#[test]
pub(super) fn test_exact_int_division_inside_nested_nonzero_guard_lowers() {
    let module = lower_source(
        "\
def main() -> None:
    left: int = 3
    right: int = 2
    if not (left == 0 or right == 0):
        a: int = 10 // left
        b: int = 10 % right
",
    )
    .expect("nested boolean zero guard should prove both divisors are non-zero inside the branch");

    let HirStmt::If { then_body, .. } = &module.functions[0].body[2] else {
        panic!("expected if statement");
    };
    let HirStmt::Let { value: a, .. } = &then_body[0] else {
        panic!("expected first guarded let");
    };
    let HirStmt::Let { value: b, .. } = &then_body[1] else {
        panic!("expected second guarded let");
    };
    assert!(matches!(a, HirExpr::BinOp { ty: Type::Int, .. }));
    assert!(matches!(b, HirExpr::BinOp { ty: Type::Int, .. }));
}

#[test]
pub(super) fn test_exact_int_nonzero_guard_is_cleared_after_reassignment() {
    let source = "\
def main() -> None:
    divisor: int = 3
    if divisor != 0:
        divisor = 0
        value: int = 10 // divisor
";
    let errors = lower_source(source).expect_err("reassignment should clear non-zero proof");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error
                .message
                .contains("expected 'int', got 'Result[int, DivisionError]'")
            && error.primary_range == Some(range_for(source, "10 // divisor"))
    }));
}

#[test]
pub(super) fn test_fixed_width_const_expression_uses_module_integer_constants() {
    let module = lower_source(
        "BASE: int = 250 + 4\n\ndef main() -> uint8:\n    value: uint8 = BASE + 1\n    return value\n",
    )
    .expect("module integer constants should participate in const fitting");

    let main_fn = &module.functions[0];
    let HirStmt::Let { ty, value, .. } = &main_fn.body[0] else {
        panic!("expected uint8 let");
    };
    assert_eq!(ty, &Type::FixedInt(FixedIntType::U8));
    assert!(matches!(value, HirExpr::IntLiteral(255)));
}

#[test]
pub(super) fn test_fixed_width_const_expression_does_not_fold_shadowed_module_constant() {
    let source = "\
BASE: int = 254

def main():
    BASE: int = 100
    value: uint8 = BASE + 1
";
    let errors = lower_source(source)
        .expect_err("shadowed module constant should not participate in const fitting");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message == "type mismatch: expected 'uint8', got 'int'"
            && error.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "value: uint8 = ",
                    "BASE + 1",
                ))
    }));
    assert!(
        errors
            .iter()
            .all(|error| error.code != Some(DiagnosticCode::INT_FIXED_WIDTH_OUT_OF_RANGE)),
        "shadowed module constants should not be folded into range diagnostics: {errors:?}"
    );
}

#[test]
pub(super) fn test_fixed_width_scalar_add_sub_mul_promote_to_int() {
    let module = lower_source(
        "\
def main() -> None:
    tiny: int8 = 2
    small: int16 = 3
    left: int32 = 5
    wide: int64 = 7
    byte: uint8 = 11
    mid: uint16 = 13
    large: uint32 = 17
    pointer: isize = 19
    tiny_total: int = tiny + small
    total: int = left + wide
    diff: int = mid - byte
    product: int = 2 * large
    pointer_total: int = pointer + 1
",
    )
    .expect("ordinary fixed-width scalar arithmetic should promote to int");

    assert!(matches!(
        function_let_value(&module, "tiny_total"),
        HirExpr::BinOp { ty: Type::Int, .. }
    ));
    assert!(matches!(
        function_let_value(&module, "total"),
        HirExpr::BinOp { ty: Type::Int, .. }
    ));
    assert!(matches!(
        function_let_value(&module, "diff"),
        HirExpr::BinOp { ty: Type::Int, .. }
    ));
    assert!(matches!(
        function_let_value(&module, "product"),
        HirExpr::BinOp { ty: Type::Int, .. }
    ));
    assert!(matches!(
        function_let_value(&module, "pointer_total"),
        HirExpr::BinOp { ty: Type::Int, .. }
    ));
}

#[test]
pub(super) fn test_fixed_width_add_apis_have_representation_preserving_types() {
    let module = lower_source(
        "\
def main() -> None:
    high: int8 = 127
    one: int8 = 1
    checked: Result[int8, OverflowError] = high.checked_add(one)
    wrapped: int8 = high.wrapping_add(one)
    saturated: int8 = high.saturating_add(one)
    overflowed: tuple[int8, bool] = high.overflowing_add(one)
",
    )
    .expect("fixed-width add APIs should lower with representation-preserving types");

    assert!(matches!(
        function_let_value(&module, "checked"),
        HirExpr::MethodCall { ty: Type::Result(ok, _), .. }
            if ok.as_ref() == &Type::FixedInt(FixedIntType::I8)
    ));
    assert_eq!(
        function_let_value(&module, "wrapped").ty(),
        &Type::FixedInt(FixedIntType::I8)
    );
    assert_eq!(
        function_let_value(&module, "saturated").ty(),
        &Type::FixedInt(FixedIntType::I8)
    );
    assert_eq!(
        function_let_value(&module, "overflowed").ty(),
        &Type::Tuple(vec![Type::FixedInt(FixedIntType::I8), Type::Bool])
    );
}

#[test]
pub(super) fn test_fixed_width_add_api_rejects_mixed_width_argument() {
    let source = "\
def main() -> None:
    left: int8 = 1
    right: int16 = 2
    wrapped: int8 = left.wrapping_add(right)
";
    let errors = lower_source(source).expect_err("fixed-width add API should reject mixed widths");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message == "int8.wrapping_add() argument must be 'int8', got 'int16'"
            && error.primary_range == Some(range_for_after(source, "wrapping_add(", "right"))
    }));
}

#[test]
pub(super) fn test_fixed_width_sub_apis_have_representation_preserving_types() {
    let module = lower_source(
        "\
def main() -> None:
    low: uint8 = 0
    one: uint8 = 1
    checked: Result[uint8, OverflowError] = low.checked_sub(one)
    wrapped: uint8 = low.wrapping_sub(one)
    saturated: uint8 = low.saturating_sub(one)
    overflowed: tuple[uint8, bool] = low.overflowing_sub(one)
",
    )
    .expect("fixed-width sub APIs should lower with representation-preserving types");

    assert!(matches!(
        function_let_value(&module, "checked"),
        HirExpr::MethodCall { ty: Type::Result(ok, _), .. }
            if ok.as_ref() == &Type::FixedInt(FixedIntType::U8)
    ));
    assert_eq!(
        function_let_value(&module, "wrapped").ty(),
        &Type::FixedInt(FixedIntType::U8)
    );
    assert_eq!(
        function_let_value(&module, "saturated").ty(),
        &Type::FixedInt(FixedIntType::U8)
    );
    assert_eq!(
        function_let_value(&module, "overflowed").ty(),
        &Type::Tuple(vec![Type::FixedInt(FixedIntType::U8), Type::Bool])
    );
}

#[test]
pub(super) fn test_fixed_width_sub_api_rejects_mixed_width_argument() {
    let source = "\
def main() -> None:
    left: uint8 = 1
    right: uint16 = 2
    wrapped: uint8 = left.wrapping_sub(right)
";
    let errors = lower_source(source).expect_err("fixed-width sub API should reject mixed widths");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message == "uint8.wrapping_sub() argument must be 'uint8', got 'uint16'"
            && error.primary_range == Some(range_for_after(source, "wrapping_sub(", "right"))
    }));
}

#[test]
pub(super) fn test_fixed_width_mul_apis_have_representation_preserving_types() {
    let module = lower_source(
        "\
def main() -> None:
    left: uint8 = 16
    right: uint8 = 16
    checked: Result[uint8, OverflowError] = left.checked_mul(right)
    wrapped: uint8 = left.wrapping_mul(right)
    saturated: uint8 = left.saturating_mul(right)
    overflowed: tuple[uint8, bool] = left.overflowing_mul(right)
",
    )
    .expect("fixed-width mul APIs should lower with representation-preserving types");

    assert!(matches!(
        function_let_value(&module, "checked"),
        HirExpr::MethodCall { ty: Type::Result(ok, _), .. }
            if ok.as_ref() == &Type::FixedInt(FixedIntType::U8)
    ));
    assert_eq!(
        function_let_value(&module, "wrapped").ty(),
        &Type::FixedInt(FixedIntType::U8)
    );
    assert_eq!(
        function_let_value(&module, "saturated").ty(),
        &Type::FixedInt(FixedIntType::U8)
    );
    assert_eq!(
        function_let_value(&module, "overflowed").ty(),
        &Type::Tuple(vec![Type::FixedInt(FixedIntType::U8), Type::Bool])
    );
}

#[test]
pub(super) fn test_fixed_width_mul_api_rejects_mixed_width_argument() {
    let source = "\
def main() -> None:
    left: uint8 = 2
    right: uint16 = 3
    wrapped: uint8 = left.wrapping_mul(right)
";
    let errors = lower_source(source).expect_err("fixed-width mul API should reject mixed widths");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message == "uint8.wrapping_mul() argument must be 'uint8', got 'uint16'"
            && error.primary_range == Some(range_for_after(source, "wrapping_mul(", "right"))
    }));
}

#[test]
pub(super) fn test_fixed_width_const_expression_out_of_range_has_int_code() {
    let source = "def main():\n    too_wide: uint8 = 2 ** 8\n";
    let errors =
        lower_source(source).expect_err("out-of-range fixed-width const expression should fail");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::INT_FIXED_WIDTH_OUT_OF_RANGE)
            && error.message
                == "integer value 256 does not fit target type uint8; valid range is 0..=255"
            && error.primary_range == Some(range_for(source, "2 ** 8"))
    }));
    assert!(
        errors
            .iter()
            .all(|error| error.code != Some(DiagnosticCode::TYPE_MISMATCH)),
        "range diagnostics should not be followed by generic type mismatches: {errors:?}"
    );
}

#[test]
pub(super) fn test_fixed_width_const_expression_budget_has_int_code() {
    let source = "def main():\n    too_large: uint8 = 10 ** 5000\n";
    let errors =
        lower_source(source).expect_err("over-budget fixed-width const expression should fail");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::INT_EVAL_BUDGET_EXCEEDED)
            && error.message
                == "integer literal exceeds compile-time evaluation budget: 5001 decimal digits (max 4096)"
            && error.primary_range == Some(range_for(source, "10 ** 5000"))
    }));
    assert!(
        errors
            .iter()
            .all(|error| error.code != Some(DiagnosticCode::INT_FIXED_WIDTH_OUT_OF_RANGE)),
        "over-budget const expressions should not also emit range diagnostics: {errors:?}"
    );
}

#[test]
pub(super) fn test_module_constant_export_uses_prior_const_name() {
    let module = lower_source(
        "BASE: int = 250\nLIMIT: int = BASE + 4\n\ndef main() -> uint8:\n    value: uint8 = LIMIT + 1\n    return value\n",
    )
    .expect("module constants should reuse earlier const-evaluable names");

    let main_fn = &module.functions[0];
    let HirStmt::Let { ty, value, .. } = &main_fn.body[0] else {
        panic!("expected uint8 let");
    };
    assert_eq!(ty, &Type::FixedInt(FixedIntType::U8));
    assert!(matches!(value, HirExpr::IntLiteral(255)));
}

#[test]
pub(super) fn test_module_constant_export_uses_unary_prior_const_name() {
    let module = lower_source(
        "BASE: int = 10\nNEGATIVE: int = -(BASE + 3)\n\ndef main() -> int8:\n    value: int8 = NEGATIVE\n    return value\n",
    )
    .expect("module constants should reuse earlier const-evaluable names through unary expressions");

    let main_fn = &module.functions[0];
    let HirStmt::Let { ty, value, .. } = &main_fn.body[0] else {
        panic!("expected int8 let");
    };
    assert_eq!(ty, &Type::FixedInt(FixedIntType::I8));
    assert!(matches!(value, HirExpr::IntLiteral(-13)));
}

#[test]
pub(super) fn test_module_constant_export_does_not_retype_fixed_width_name_as_int() {
    let module = lower_source(
        "BASE: uint8 = 250\nLIMIT: int = BASE + 4\n\ndef main():\n    print(\"ok\")\n",
    )
    .expect("mixed fixed-width to int module const reuse should stay out of export lowering");

    assert!(
        module
            .constants
            .iter()
            .any(|(name, ty, value)| name == "BASE"
                && ty == &Type::FixedInt(FixedIntType::U8)
                && matches!(value, HirExpr::IntLiteral(250))),
        "source fixed-width module constant should still be collected"
    );
    assert!(
        module.constants.iter().all(|(name, _, _)| name != "LIMIT"),
        "module lowering should not synthesize an int-typed name for a fixed-width constant"
    );
}

#[test]
pub(super) fn test_module_fixed_width_const_expression_budget_has_int_code_once() {
    let source = "LIMIT: uint8 = 10 ** 5000\n\ndef main():\n    print(\"ok\")\n";
    let errors = lower_source(source)
        .expect_err("module fixed-width over-budget const expression should fail");

    let budget_errors: Vec<_> = errors
        .iter()
        .filter(|error| error.code == Some(DiagnosticCode::INT_EVAL_BUDGET_EXCEEDED))
        .collect();
    assert_eq!(
        budget_errors.len(),
        1,
        "module fixed-width over-budget const expressions should emit one budget diagnostic: {errors:?}"
    );
    assert_eq!(
        budget_errors[0].message,
        "integer literal exceeds compile-time evaluation budget: 5001 decimal digits (max 4096)"
    );
    assert_eq!(
        budget_errors[0].primary_range,
        Some(range_for(source, "10 ** 5000"))
    );
    assert!(
        errors
            .iter()
            .all(|error| error.code != Some(DiagnosticCode::INT_FIXED_WIDTH_OUT_OF_RANGE)),
        "over-budget module const expressions should not also emit range diagnostics: {errors:?}"
    );
}

#[test]
pub(super) fn test_module_int_over_budget_const_expr_stays_hir_diagnostic() {
    let source = "LIMIT: int = 10 ** 5000\n\ndef main():\n    print(\"ok\")\n";
    let errors = lower_source(source)
        .expect_err("module int over-budget const expression should fail in HIR");

    let budget_errors: Vec<_> = errors
        .iter()
        .filter(|error| error.code == Some(DiagnosticCode::INT_EVAL_BUDGET_EXCEEDED))
        .collect();
    assert_eq!(
        budget_errors.len(),
        1,
        "module int over-budget const expressions should not reach codegen: {errors:?}"
    );
    assert_eq!(
        errors.len(),
        1,
        "module int over-budget const expressions should not emit follow-on diagnostics: {errors:?}"
    );
    assert_eq!(
        budget_errors[0].primary_range,
        Some(range_for(source, "10 ** 5000"))
    );
}

#[test]
pub(super) fn test_module_int_const_expr_above_i64_folds_to_large_literal_for_codegen() {
    let module = lower_source("LIMIT: int = 10 ** 20\n\ndef main():\n    print(str(LIMIT))\n")
        .expect("in-budget oversized module int constant should lower");

    assert!(
        module
            .constants
            .iter()
            .any(|(name, ty, value)| name == "LIMIT"
                && ty == &Type::Int
                && matches!(value, HirExpr::LargeIntLiteral(value) if value == "100000000000000000000")),
        "oversized exact int module constants should be folded to a canonical large literal for codegen: {:?}",
        module.constants
    );
}

#[test]
pub(super) fn test_fixed_width_over_budget_literal_diagnostic_is_not_duplicated() {
    let literal = "1".repeat(4097);
    let source = format!("def main():\n    too_large: uint8 = {literal}\n");
    let errors =
        lower_source(&source).expect_err("over-budget fixed-width literal should fail once");

    let budget_errors: Vec<_> = errors
        .iter()
        .filter(|error| error.code == Some(DiagnosticCode::INT_EVAL_BUDGET_EXCEEDED))
        .collect();
    assert_eq!(
        budget_errors.len(),
        1,
        "over-budget literal should not duplicate diagnostics: {errors:?}"
    );
    assert_eq!(
        budget_errors[0].primary_range,
        Some(range_for(&source, &literal))
    );
    assert!(
        errors
            .iter()
            .all(|error| error.code != Some(DiagnosticCode::TYPE_MISMATCH)),
        "already-diagnosed over-budget literals should not emit generic mismatches: {errors:?}"
    );
}

#[test]
pub(super) fn test_fixed_width_assignment_from_non_const_int_is_still_mismatch() {
    let source = "def main():\n    source: int = 1\n    target: uint8 = source\n";
    let errors = lower_source(source).expect_err("non-const int narrowing should fail");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message == "type mismatch: expected 'uint8', got 'int'"
            && error.primary_range
                == Some(range_for_after_anchor(source, "target: uint8 = ", "source"))
    }));
}

#[test]
pub(super) fn test_fixed_width_assignment_from_non_const_binop_is_still_mismatch() {
    let source = "def main():\n    source: int = 1\n    target: uint8 = source + 1\n";
    let errors = lower_source(source).expect_err("non-const int binop narrowing should fail");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message == "type mismatch: expected 'uint8', got 'int'"
            && error.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "target: uint8 = ",
                    "source + 1",
                ))
    }));
}

#[test]
pub(super) fn test_fixed_width_call_argument_literal_is_not_implicitly_narrowed() {
    let source = "def take(value: uint8) -> None:\n    pass\n\ndef main():\n    take(1)\n";
    let errors = lower_source(source).expect_err("call argument literal narrowing should fail");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error
                .message
                .contains("argument 1 ('value') of function 'take'")
            && error.primary_range == Some(range_for_after_anchor(source, "def main():", "1"))
    }));
}

#[test]
pub(super) fn test_promoted_fixed_width_result_is_not_implicitly_narrowed_in_return() {
    let source = "\
def add(left: uint8, right: uint8) -> uint8:
    return left + right
";
    let errors =
        lower_source(source).expect_err("promoted fixed-width return narrowing should fail");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message == "return type mismatch: expected 'uint8', got 'int'"
            && error.primary_range
                == Some(range_for_after_anchor(source, "return ", "left + right"))
    }));
}

#[test]
pub(super) fn test_promoted_fixed_width_result_is_not_implicitly_narrowed_in_list_literal() {
    let source = "\
def main() -> None:
    left: uint8 = 1
    right: uint8 = 2
    values: list[uint8] = [left + right]
";
    let errors = lower_source(source).expect_err("promoted fixed-width list narrowing should fail");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message == "type mismatch: expected 'list[uint8]', got 'list[int]'"
            && error.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "values: list[uint8] = ",
                    "[left + right]",
                ))
    }));
}

#[test]
pub(super) fn test_promoted_fixed_width_result_is_not_implicitly_narrowed_in_dict_literal() {
    let source = "\
def main() -> None:
    left: uint8 = 1
    right: uint8 = 2
    values: dict[str, uint8] = {\"sum\": left + right}
";
    let errors = lower_source(source).expect_err("promoted fixed-width dict narrowing should fail");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message == "type mismatch: expected 'dict[str, uint8]', got 'dict[str, int]'"
            && error.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "values: dict[str, uint8] = ",
                    "{\"sum\": left + right}",
                ))
    }));
}

#[test]
pub(super) fn test_promoted_fixed_width_result_is_not_implicitly_narrowed_in_generic_specialization(
) {
    let source = "\
class Box[T]:
    value: T

    def __init__(self, value: T):
        self.value = value

def main() -> None:
    left: uint8 = 1
    right: uint8 = 2
    box: Box[uint8] = Box(left + right)
";
    let errors =
        lower_source(source).expect_err("promoted fixed-width generic narrowing should fail");

    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message == "type mismatch: expected 'Box', got 'Box'"
                && error.primary_range
                    == Some(range_for_after_anchor(
                        source,
                        "box: Box[uint8] = ",
                        "Box(left + right)",
                    ))
        }),
        "{errors:?}"
    );
}

#[test]
pub(super) fn test_reassignment_type_mismatch_has_primary_range() {
    let source = "def main():\n    value = 1\n    value = \"not an int\"\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected reassignment type mismatch");
    assert!(errors.iter().any(|e| e
        .message
        .contains("cannot assign 'str' to variable 'value'")
        && e.code == Some(DiagnosticCode::TYPE_MISMATCH)
        && e.primary_range == Some(range_for(source, "\"not an int\""))));
}

#[test]
pub(super) fn test_return_type_mismatch_has_primary_range() {
    let source = "def main() -> int:\n    return \"no\"\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected return type mismatch");
    assert!(errors
        .iter()
        .any(|e| e.message.contains("return type mismatch")
            && e.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && e.primary_range == Some(range_for(source, "\"no\""))));
}

#[test]
pub(super) fn test_function_argument_type_mismatch_has_primary_range() {
    let source = "def takes_int(value: int) -> int:\n    return value\n\ndef main():\n    result: int = takes_int(\"bad\")\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected argument type mismatch");
    assert!(errors.iter().any(|e| e
        .message
        .contains("argument 1 ('value') of function 'takes_int'")
        && e.code == Some(DiagnosticCode::TYPE_MISMATCH)
        && e.primary_range == Some(range_for(source, "\"bad\""))));
}

#[test]
pub(super) fn test_typevar_constraint_mismatch_has_primary_range() {
    let source = "from typing import TypeVar\n\nT = TypeVar(\"T\", int, str)\n\ndef echo(x: T) -> T:\n    return x\n\ndef main():\n    bad: float = echo(1.5)\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected TypeVar constraint mismatch");
    assert!(errors.iter().any(|e| e.code
        == Some(DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED)
        && e.primary_range == Some(range_for(source, "1.5"))));
}

#[test]
pub(super) fn test_if_expression_branch_mismatch_has_primary_range() {
    let source = "def main():\n    x: str | int = \"yes\" if True else 42\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected if-expression branch mismatch");
    assert!(errors
        .iter()
        .any(|e| e.code == Some(DiagnosticCode::TYPE_IF_BRANCH_MISMATCH)
            && e.primary_range == Some(range_for(source, "42"))));
}

#[test]
pub(super) fn test_container_literal_type_conflict_has_primary_range() {
    let source = "def main():\n    values = [1, \"two\"]\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected container literal conflict");
    assert!(errors.iter().any(
        |e| e.code == Some(DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT)
            && e.primary_range == Some(range_for(source, "\"two\""))
    ));
}

#[test]
pub(super) fn test_undefined_variable() {
    let result = lower_source("def main():\n    print(x)\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("undefined variable")));
}

#[test]
pub(super) fn test_failed_assignment_rhs_still_seeds_followup_binding() {
    let result =
        lower_source("def main(xs: list[int]) -> int:\n    s = xs[0] + xs[0]\n    return s\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("unsupported operand type(s) for +")));
    assert!(
        !errors
            .iter()
            .any(|e| e.message == "undefined variable: 's'"),
        "failed initializer should not cascade to undefined-name errors: {errors:?}"
    );
    assert!(
        !errors.iter().any(|e| {
            e.message
                .contains("must return a value of type 'int' on all control-flow paths")
        }),
        "failed initializer should not trigger a synthetic missing-return diagnostic: {errors:?}"
    );
}

#[test]
pub(super) fn test_failed_annotated_assignment_rhs_still_seeds_followup_binding() {
    let result =
        lower_source("def main(xs: list[int]) -> int:\n    s: int = xs[0] + xs[0]\n    return s\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("unsupported operand type(s) for +")));
    assert!(
        !errors
            .iter()
            .any(|e| e.message == "undefined variable: 's'"),
        "failed annotated initializer should not cascade to undefined-name errors: {errors:?}"
    );
    assert!(
        !errors.iter().any(|e| {
            e.message
                .contains("must return a value of type 'int' on all control-flow paths")
        }),
        "failed annotated initializer should not trigger a synthetic missing-return diagnostic: {errors:?}"
    );
}
