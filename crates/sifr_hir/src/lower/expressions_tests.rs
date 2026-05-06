use super::{
    classes::lower_expr_simple,
    expressions::{lower_named_expr, resolve_method_type},
    LowerCtx,
};
use crate::{lower_module, HirDiagnostic, HirExpr, HirModule, HirStmt};
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{
    AtomicNodeIndex, Expr, ExprNamed, ExprNoneLiteral, ExprNumberLiteral, Int, Number,
};
use sifr_python_parser::parse_module;
use sifr_type_system::{FixedIntType, FunctionType, Type};

fn lower_source(source: &str) -> Result<HirModule, Vec<HirDiagnostic>> {
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).map(|r| r.module)
}

fn range_for(source: &str, needle: &str) -> TextRange {
    let start = source.find(needle).expect("needle should exist") as u32;
    TextRange::new(
        TextSize::new(start),
        TextSize::new(start + needle.len() as u32),
    )
}

fn range_for_after(source: &str, after: &str, needle: &str) -> TextRange {
    let after_start = source.find(after).expect("anchor should exist");
    let relative_start = source[after_start..]
        .find(needle)
        .expect("needle should exist after anchor");
    let start = (after_start + relative_start) as u32;
    TextRange::new(
        TextSize::new(start),
        TextSize::new(start + needle.len() as u32),
    )
}

fn range_for_after_anchor(source: &str, after: &str, needle: &str) -> TextRange {
    let search_start = source.find(after).expect("anchor should exist") + after.len();
    let relative_start = source[search_start..]
        .find(needle)
        .expect("needle should exist after anchor");
    let start = (search_start + relative_start) as u32;
    TextRange::new(
        TextSize::new(start),
        TextSize::new(start + needle.len() as u32),
    )
}

fn function_let_value<'a>(module: &'a HirModule, name: &str) -> &'a HirExpr {
    module
        .functions
        .iter()
        .flat_map(|function| &function.body)
        .find_map(|stmt| match stmt {
            HirStmt::Let {
                name: local_name,
                value,
                ..
            } if local_name == name => Some(value),
            _ => None,
        })
        .expect("expected local binding")
}

#[test]
fn test_simple_function() {
    let module = lower_source("def add(a: int, b: int) -> int:\n    return a + b\n").unwrap();
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "add");
    assert_eq!(module.functions[0].return_type, Type::Int);
}

#[test]
fn test_large_integer_literals_lower_losslessly_from_source() {
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
fn test_negative_large_integer_literal_lowers_as_unary_large_literal() {
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
fn test_large_integer_default_arguments_lower_losslessly() {
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
fn test_large_integer_literal_over_budget_has_int_code() {
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
fn test_constructed_and_parsed_large_integer_literals_match_hir() {
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
fn test_type_mismatch_error() {
    let source = "def main():\n    x: int = \"hello\"\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("type mismatch")
        && e.code == Some(DiagnosticCode::TYPE_MISMATCH)
        && e.primary_range == Some(range_for(source, "\"hello\""))));
}

#[test]
fn test_fixed_width_literal_assignment_fits() {
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
    assert!(
        matches!(value, HirExpr::UnaryOp { op, operand, .. } if op == "-" && matches!(operand.as_ref(), HirExpr::IntLiteral(128)))
    );

    let HirStmt::Let { ty, value, .. } = &main_fn.body[2] else {
        panic!("expected third statement to be uint64 let");
    };
    assert_eq!(ty, &Type::FixedInt(FixedIntType::U64));
    assert!(matches!(value, HirExpr::LargeIntLiteral(value) if value == "18446744073709551615"));
}

#[test]
fn test_fixed_width_literal_assignment_out_of_range_has_int_code() {
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
fn test_fixed_width_module_constant_out_of_range_has_int_code() {
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
fn test_fixed_width_assignment_from_non_const_int_is_still_mismatch() {
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
fn test_fixed_width_call_argument_literal_is_not_implicitly_narrowed() {
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
fn test_reassignment_type_mismatch_has_primary_range() {
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
fn test_return_type_mismatch_has_primary_range() {
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
fn test_function_argument_type_mismatch_has_primary_range() {
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
fn test_typevar_constraint_mismatch_has_primary_range() {
    let source = "from typing import TypeVar\n\nT = TypeVar(\"T\", int, str)\n\ndef echo(x: T) -> T:\n    return x\n\ndef main():\n    bad: float = echo(1.5)\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected TypeVar constraint mismatch");
    assert!(errors.iter().any(|e| e.code
        == Some(DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED)
        && e.primary_range == Some(range_for(source, "1.5"))));
}

#[test]
fn test_if_expression_branch_mismatch_has_primary_range() {
    let source = "def main():\n    x: str | int = \"yes\" if True else 42\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected if-expression branch mismatch");
    assert!(errors
        .iter()
        .any(|e| e.code == Some(DiagnosticCode::TYPE_IF_BRANCH_MISMATCH)
            && e.primary_range == Some(range_for(source, "42"))));
}

#[test]
fn test_container_literal_type_conflict_has_primary_range() {
    let source = "def main():\n    values = [1, \"two\"]\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected container literal conflict");
    assert!(errors.iter().any(
        |e| e.code == Some(DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT)
            && e.primary_range == Some(range_for(source, "\"two\""))
    ));
}

#[test]
fn test_undefined_variable() {
    let result = lower_source("def main():\n    print(x)\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("undefined variable")));
}

#[test]
fn test_failed_assignment_rhs_still_seeds_followup_binding() {
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
fn test_failed_annotated_assignment_rhs_still_seeds_followup_binding() {
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

#[test]
fn test_poisoned_initializer_binding_suppresses_followup_operator_cascade() {
    let result =
        lower_source("def main(xs: list[int]) -> int:\n    s = xs[0] + xs[0]\n    return s + 1\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let unsupported_operator_count = errors
        .iter()
        .filter(|error| error.message.contains("unsupported operand type(s) for +"))
        .count();
    assert_eq!(
        unsupported_operator_count, 1,
        "poisoned initializer binding should not trigger a second operator cascade: {errors:?}"
    );
    assert!(
        !errors
            .iter()
            .any(|error| error.message == "undefined variable: 's'"),
        "poisoned initializer binding should suppress undefined-name cascades: {errors:?}"
    );
}

#[test]
fn test_poisoned_initializer_binding_suppresses_followup_unary_cascade() {
    let result =
        lower_source("def main(xs: list[int]) -> int:\n    s = xs[0] + xs[0]\n    return -s\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.message.contains("unsupported operand type(s)"))
            .count(),
        1,
        "poisoned initializer binding should not trigger unary operator cascades: {errors:?}"
    );
}

#[test]
fn test_use_after_move() {
    let source = "def consume(own s: str) -> str:\n    return s\ndef main():\n    s: str = \"hello\"\n    x: str = consume(s)\n    print(s)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("moved value")
            && e.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && e.primary_range == Some(range_for_after(source, "print(", "s"))
    }));
}

#[test]
fn test_double_mutable_borrow_has_ownership_code() {
    let source = "def swap(mut a: list[int], mut b: list[int]):\n    pass\n\ndef main():\n    items: list[int] = [1, 2, 3]\n    swap(items, items)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("cannot borrow 'items' as mutable more than once")
            && e.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
            && e.primary_range == Some(range_for_after_anchor(source, "swap(items, ", "items"))
    }));
}

#[test]
fn test_mutable_after_immutable_borrow_has_ownership_code() {
    let source = "def read_then_mutate(a: list[int], mut b: list[int]):\n    pass\n\ndef main():\n    items: list[int] = [1, 2, 3]\n    read_then_mutate(items, items)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains(
            "cannot borrow 'items' as mutable because it is already borrowed as immutable",
        ) && e.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
            && e.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "read_then_mutate(items, ",
                    "items",
                ))
    }));
}

#[test]
fn test_immutable_after_mutable_borrow_has_ownership_code() {
    let source = "def mutate_then_read(mut a: list[int], b: list[int]):\n    pass\n\ndef main():\n    items: list[int] = [1, 2, 3]\n    mutate_then_read(items, items)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains(
            "cannot borrow 'items' as immutable because it is already borrowed as mutable",
        ) && e.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
            && e.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "mutate_then_read(items, ",
                    "items",
                ))
    }));
}

#[test]
fn test_for_loop_move_has_ownership_code() {
    let result = lower_source(
        "def consume(own s: str) -> int:\n    return len(s)\n\ndef main():\n    s: str = \"hello\"\n    for i in range(3):\n        result: int = consume(s)\n        print(result)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("is moved inside loop body")
            && e.code == Some(DiagnosticCode::OWN_MOVED_ACROSS_LOOP)
    }));
}

#[test]
fn test_while_loop_move_has_ownership_code() {
    let result = lower_source(
        "def consume(own s: str) -> int:\n    return len(s)\n\ndef main():\n    s: str = \"hello\"\n    i: int = 0\n    while i < 3:\n        result: int = consume(s)\n        i = i + 1\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("is moved inside loop body")
            && e.code == Some(DiagnosticCode::OWN_MOVED_ACROSS_LOOP)
    }));
}

#[test]
fn test_borrow_by_default_no_move() {
    let result = lower_source(
        "def process(s: str) -> int:\n    return len(s)\ndef main():\n    s: str = \"hello\"\n    x: int = process(s)\n    print(s)\n",
    );
    assert!(
        result.is_ok(),
        "borrow-by-default should not cause use-after-move"
    );
}

#[test]
fn test_user_defined_sum_shadows_builtin() {
    let result = lower_source(
        "def sum(num1: int, num2: int) -> int:\n    return num1 + num2\ndef main():\n    assert sum(12, 5) == 17\n",
    );
    assert!(
        result.is_ok(),
        "user-defined sum should shadow the builtin lowering path"
    );
}

#[test]
fn test_builtin_set_constructor_accepts_list_iterable() {
    let result = lower_source("def main():\n    seen = set([1, 2, 2])\n    assert 2 in seen\n");
    assert!(
        result.is_ok(),
        "set(list[T]) should lower as a builtin constructor"
    );
}

#[test]
#[ignore = "depends on driver-loaded stdlib compat registry"]
fn test_bare_deque_call_resolves_without_import() {
    let result = lower_source(
        "from sifr.collections import deque\n\ndef main():\n    q = deque([1])\n    q.append(2)\n    assert q.popleft() == 1\n",
    );
    assert!(
        result.is_ok(),
        "deque(...) should resolve through the compat stdlib surface: {:?}",
        result.err()
    );
}

#[test]
fn test_generic_constructor_infers_typevar_from_optional_union_param() {
    let result = lower_source(
        "class Bucket[T]:\n    items: list[T]\n\n    def __init__(self, items: list[T] | None = None):\n        if items is None:\n            self.items = []\n        else:\n            self.items = items\n\n    def first(self) -> T | None:\n        if len(self.items) == 0:\n            return None\n        return self.items[0]\n\ndef main() -> int:\n    bucket = Bucket([1])\n    value = bucket.first()\n    if value is None:\n        return 0\n    return value + 1\n",
    );
    assert!(
        result.is_ok(),
        "constructor call should infer T from list[T] | None parameter when called with list[int]: {:?}",
        result.err()
    );
}

#[test]
fn test_defaultdict_list_call_resolves_without_import() {
    let result = lower_source(
        "def main():\n    groups = defaultdict(list)\n    groups[\"a\"].append(\"x\")\n    assert len(groups[\"a\"]) == 1\n",
    );
    assert!(
        result.is_ok(),
        "defaultdict(list) should resolve through the compat builtin surface"
    );
}

#[test]
fn test_defaultdict_keyword_constructor_unsupported_has_stdlib_code() {
    let source = "def main():\n    groups = defaultdict(default_factory=list)\n    _ = groups\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "defaultdict() does not support keyword arguments"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for(source, "default_factory"))
    }));
}

#[test]
fn test_defaultdict_unpacked_keyword_constructor_unsupported_has_stdlib_code() {
    let source =
        "def main():\n    groups = defaultdict(**{\"default_factory\": list})\n    _ = groups\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "defaultdict() does not support unpacked keyword arguments"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for(source, "**{\"default_factory\": list}"))
    }));
}

#[test]
fn test_builtin_sum_wrong_arity_has_call_code() {
    let source = "def main():\n    data: list[int] = [1, 2, 3]\n    print(sum(data, data))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "sum() takes exactly 1 argument(s), got 2"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(source, "sum(data, ", "data"))
    }));
}

#[test]
fn test_sorted_unexpected_keyword_has_call_code() {
    let source = "def main():\n    nums: list[int] = [3, 1, 2]\n    ordered: list[int] = sorted(nums, bogus=True)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "sorted() got an unexpected keyword argument 'bogus'"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range == Some(range_for_after_anchor(source, "sorted(nums, ", "bogus"))
    }));
}

#[test]
fn test_sorted_and_range_missing_required_argument_have_call_code() {
    let sorted_source = "def main():\n    values: list[int] = sorted()\n";
    let sorted_result = lower_source(sorted_source);
    assert!(sorted_result.is_err());
    let sorted_errors = sorted_result.unwrap_err();
    assert!(sorted_errors.iter().any(|error| {
        error.message == "sorted() missing required argument 'iterable'"
            && error.code == Some(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT)
            && error.primary_range == Some(range_for(sorted_source, "sorted"))
    }));

    let range_source = "def main():\n    values: list[int] = list(range())\n";
    let range_result = lower_source(range_source);
    assert!(range_result.is_err());
    let range_errors = range_result.unwrap_err();
    assert!(range_errors.iter().any(|error| {
        error.message == "range() missing required argument 'stop'"
            && error.code == Some(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT)
            && error.primary_range == Some(range_for(range_source, "range"))
    }));
}

#[test]
fn test_function_unexpected_keyword_has_call_code() {
    let source = "def greet(name: str) -> str:\n    return \"hello\"\n\ndef main():\n    print(greet(\"Alice\", punctuation=\"!\"))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "greet() got an unexpected keyword argument 'punctuation'"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "greet(\"Alice\", ",
                    "punctuation",
                ))
    }));
}

#[test]
fn test_keyword_after_positional_has_call_code() {
    let source = "def greet(name: str, greeting: str) -> str:\n    return greeting\n\ndef main():\n    print(greet(\"Alice\", name=\"Bob\"))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "greet() got multiple values for argument 'name'"
            && error.code == Some(DiagnosticCode::CALL_DUPLICATE_ARGUMENT)
            && error.primary_range
                == Some(range_for_after_anchor(source, "greet(\"Alice\", ", "name"))
    }));
}

#[test]
fn test_range_duplicate_stop_keyword_has_call_code() {
    let source = "def main():\n    print(list(range(10, stop=20)))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "range() got multiple values for argument 'stop'"
            && error.code == Some(DiagnosticCode::CALL_DUPLICATE_ARGUMENT)
            && error.primary_range == Some(range_for_after_anchor(source, "range(10, ", "stop"))
    }));
}

#[test]
fn test_map_callable_arity_mismatch_has_call_code() {
    let source = "def inc(x: int) -> int:\n    return x + 1\n\ndef main():\n    values: list[int] = map(inc, [1, 2], [3, 4])\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "map() callable expects 1 argument(s), got 2 iterable(s)"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "map(inc, [1, 2], ",
                    "[3, 4]",
                ))
    }));
}

#[test]
fn test_non_simple_call_target_has_call_code() {
    let source = "def make() -> int:\n    return 1\n\ndef main():\n    value: int = make()(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "only simple function calls are supported"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range
                == Some(range_for_after_anchor(source, "value: int = ", "make()"))
    }));
}

#[test]
fn test_open_missing_path_has_call_code() {
    let source = "def main():\n    _file = open()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "open() requires at least 1 argument: open(path) or open(path, mode)"
            && error.code == Some(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT)
            && error.primary_range == Some(range_for(source, "open"))
    }));
}

#[test]
fn test_callable_variable_call_errors_have_codes() {
    let arity_source = "def apply(f: Callable[[int], int]) -> int:\n    return f()\n";
    let arity_result = lower_source(arity_source);
    assert!(arity_result.is_err());
    let arity_errors = arity_result.unwrap_err();
    assert!(arity_errors.iter().any(|error| {
        error.message == "callable 'f' expects 1 argument(s), got 0"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range == Some(range_for_after_anchor(arity_source, "return ", "f"))
    }));

    let type_source = "def apply(f: Callable[[int], int]) -> int:\n    return f(\"bad\")\n";
    let type_result = lower_source(type_source);
    assert!(type_result.is_err());
    let type_errors = type_result.unwrap_err();
    assert!(type_errors.iter().any(|error| {
        error.message == "argument 1 of callable 'f': expected 'int', got 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(type_source, "f(", "\"bad\""))
    }));
}

#[test]
fn test_iter_keyword_has_call_code() {
    let source = "def main():\n    values: list[int] = [1, 2, 3]\n    _it = iter(source=values)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "iter() does not accept keyword arguments"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range == Some(range_for_after_anchor(source, "iter(", "source=values"))
    }));
}

#[test]
fn test_iter_wrong_arg_count_has_call_code() {
    let source = "def main():\n    values: list[int] = [1, 2, 3]\n    _it = iter(values, values)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "iter() takes exactly 1 argument, got 2"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range
                == Some(range_for_after_anchor(source, "iter(values, ", "values"))
    }));
}

#[test]
fn test_iter_non_iterable_has_type_code() {
    let source = "def main():\n    _it = iter(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "iter() argument must be iterable, got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(source, "iter(", "1"))
    }));
}

#[test]
fn test_next_non_iterator_has_type_code() {
    let source = "def main():\n    values: list[int] = [1, 2, 3]\n    next(values)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "next() argument must be an iterator, got 'list[int]'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(source, "next(", "values"))
    }));
}

#[test]
fn test_pow_wrong_arg_count_has_call_code() {
    let source = "def main():\n    value: int = pow(2)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "pow() takes exactly 2 arguments"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(source, "pow(", "2"))
    }));
}

#[test]
fn test_scalar_builtin_wrong_arg_counts_have_call_code() {
    let cases = [
        ("abs", "abs()", "abs() takes exactly 1 argument, got 0"),
        ("hash", "hash()", "hash() takes exactly 1 argument, got 0"),
        ("round", "round()", "round() takes 1 or 2 arguments, got 0"),
        ("repr", "repr()", "repr() takes exactly 1 argument, got 0"),
        ("int", "int()", "int() takes exactly 1 argument, got 0"),
        (
            "bigint",
            "bigint()",
            "bigint() takes exactly 1 argument, got 0",
        ),
        (
            "float",
            "float()",
            "float() takes exactly 1 argument, got 0",
        ),
        ("bool", "bool()", "bool() takes exactly 1 argument, got 0"),
    ];

    for (callable, call, message) in cases {
        let source = format!("def main():\n    _value = {call}\n");
        let result = lower_source(&source);
        assert!(result.is_err(), "{callable} should reject wrong arity");
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|error| {
                error.message == message
                    && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
                    && error.primary_range
                        == Some(range_for_after_anchor(&source, "_value = ", callable))
            }),
            "{callable} errors: {errors:?}"
        );
    }
}

#[test]
fn test_scalar_builtin_keywords_have_call_code() {
    let callables = [
        "abs", "hash", "round", "repr", "int", "bigint", "float", "bool",
    ];

    for callable in callables {
        let source = format!("def main():\n    _value = {callable}(value=1)\n");
        let result = lower_source(&source);
        assert!(result.is_err(), "{callable} should reject keywords");
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|error| {
                error.message == format!("{callable}() does not accept keyword arguments")
                    && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
                    && error.primary_range
                        == Some(range_for_after_anchor(
                            &source,
                            &format!("{callable}("),
                            "value=1",
                        ))
            }),
            "{callable} errors: {errors:?}"
        );
    }
}

#[test]
fn test_scalar_builtin_type_mismatches_have_type_code() {
    let cases = [
        (
            "abs",
            "abs(\"x\")",
            "abs() argument must be numeric, got 'str'",
        ),
        (
            "round",
            "round(\"x\")",
            "round() argument must be numeric, got 'str'",
        ),
        (
            "bigint",
            "bigint(\"x\")",
            "bigint() requires int, bigint, decimal, or bigdecimal argument, got 'str'",
        ),
    ];

    for (callable, call, message) in cases {
        let source = format!("def main():\n    _value = {call}\n");
        let result = lower_source(&source);
        assert!(
            result.is_err(),
            "{callable} should reject invalid argument type"
        );
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|error| {
                error.message == message
                    && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                    && error.primary_range
                        == Some(range_for_after_anchor(
                            &source,
                            &format!("{callable}("),
                            "\"x\"",
                        ))
            }),
            "{callable} errors: {errors:?}"
        );
    }
}

#[test]
fn test_hash_unhashable_argument_has_proto_code() {
    let result = lower_source(
        "class Measurement:\n    value: float\n\n    def __init__(self, value: float):\n        self.value = value\n\ndef main():\n    m: Measurement = Measurement(3.14)\n    print(hash(m))\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "hash() argument must be hashable, got 'Measurement'"
            && error.code == Some(DiagnosticCode::PROTO_HASHABLE_OR_COMPARABLE_REQUIRED)
    }));
}

#[test]
fn test_function_wrong_arg_count_has_call_code() {
    let source =
        "def takes_one(x: int) -> int:\n    return x\n\ndef main():\n    print(takes_one(1, 2))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "takes_one() takes at most 1 argument(s), got 2"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(source, "takes_one(1, ", "2"))
    }));
}

#[test]
fn test_missing_required_argument_has_call_code() {
    let source = "def display(name: str, *, verbose: bool) -> str:\n    if verbose:\n        return \"verbose\"\n    return \"quiet\"\n\ndef main():\n    print(display(\"Alice\"))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "display() missing required argument 'verbose'"
            && error.code == Some(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT)
            && error.primary_range == Some(range_for_after_anchor(source, "print(", "display"))
    }));
}

#[test]
fn test_defaultdict_accepts_counter_initial_mapping() {
    let result = lower_source(
        "class Counter[K: Hashable]:\n    counts: dict[K, int]\n\n    def __init__(self):\n        self.counts = {}\n\ndef main():\n    c = Counter()\n    d = defaultdict(int, c)\n    assert d is not None\n",
    );
    assert!(
        result.is_ok(),
        "defaultdict(int, Counter(...)) should lower via Counter.counts mapping bridge: {:?}",
        result.err()
    );
}

#[test]
fn test_defaultdict_subscript_read_is_non_optional_value_type() {
    let result = lower_source(
        "def main() -> int:\n    counts = defaultdict(int)\n    counts[1] += 1\n    value: int = counts[2]\n    return value\n",
    );
    assert!(
        result.is_ok(),
        "defaultdict index reads should resolve to the factory value type, not Optional"
    );
}

#[test]
fn test_defaultdict_membership_checks_lower() {
    let result = lower_source(
        "def main() -> bool:\n    groups = defaultdict(list)\n    groups[\"a\"].append(1)\n    return \"a\" in groups and \"b\" not in groups\n",
    );
    assert!(
        result.is_ok(),
        "defaultdict membership checks should lower through compat mapping surface: {:?}",
        result.err()
    );
}

#[test]
fn test_range_membership_checks_lower() {
    let result =
        lower_source("def main() -> bool:\n    return (2 in range(5)) and (9 not in range(5))\n");
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_imported_counter_iterable_constructor_remains_unsupported() {
    let result = lower_source(
        "from sifr.collections import Counter\n\ndef main():\n    c: Counter[str] = Counter([\"a\", \"b\", \"a\"])\n",
    );
    assert!(
        result.is_err(),
        "imported sifr.collections.Counter(list[T]) should remain unsupported"
    );
}

#[test]
fn test_constructor_assigned_fields_infer_class_instance_types() {
    let result = lower_source(
        "class Node:\n    def __init__(self):\n        self.marked = False\n\nclass Trie:\n    def __init__(self):\n        self.root = Node()\n\n    def is_marked(self) -> bool:\n        return self.root.marked\n\ndef main() -> bool:\n    trie = Trie()\n    return trie.is_marked()\n",
    );
    assert!(
        result.is_ok(),
        "constructor-assigned class instance fields should be registered and typed"
    );
}

#[test]
fn test_constructor_branch_assignments_register_all_fields() {
    let module = lower_source(
        "class Pair:\n    def __init__(self, flag: bool):\n        if flag:\n            self.left = 1\n        else:\n            self.right = 2\n",
    )
    .expect("constructor field registration should succeed");
    let pair = module
        .classes
        .iter()
        .find(|class| class.name == "Pair")
        .expect("Pair class should lower");
    assert!(pair.fields.iter().any(|(name, _)| name == "left"));
    assert!(pair.fields.iter().any(|(name, _)| name == "right"));
}

#[test]
fn test_attribute_subscript_augassign_lowers_for_class_fields() {
    let result = lower_source(
        "class Counter:\n    def __init__(self):\n        self.counts = {}\n\n    def bump(self, key: int) -> None:\n        if key not in self.counts:\n            self.counts[key] = 0\n        self.counts[key] += 1\n\ndef main() -> None:\n    c = Counter()\n    c.bump(1)\n",
    );
    assert!(
        result.is_err(),
        "fixture should still fail due optional indexing semantics"
    );
    let errors = result.unwrap_err();
    assert!(
        !errors.iter().any(|error| {
            error
                .message
                .contains("augmented subscript assignment target must be a simple name")
        }),
        "attribute subscript augassign should lower past target-shape validation: {errors:?}"
    );
    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
                && error.message.contains("unsupported operand type(s) for +")
        ),
        "lowering should reach operand typing for attribute subscript augassign: {errors:?}"
    );
}

#[test]
fn test_nested_subscript_augassign_lowers_for_name_targets() {
    let result =
        lower_source("def bump(mut grid: list[list[int]]) -> None:\n    grid[0][0] += 1\n");
    assert!(
        result.is_err(),
        "fixture should still fail due optional indexing semantics"
    );
    let errors = result.unwrap_err();
    assert!(
        !errors.iter().any(|error| {
            error
                .message
                .contains("augmented subscript assignment target must be a simple name")
        }),
        "nested subscript augassign should lower past target-shape validation: {errors:?}"
    );
    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
                && error.message.contains("unsupported operand type(s) for +")
        ),
        "lowering should reach operand typing for nested subscript augassign: {errors:?}"
    );
}

#[test]
fn test_matrix_augassign_has_unsupported_operator_code() {
    let source = "def bad(mut value: int) -> None:\n    value @= 1\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected matrix augassign unsupported operator error");

    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
                && error.message == "matrix multiplication operator (@) is not supported"
                && error.primary_range
                    == Some(range_for_after(source, ") -> None:\n    ", "value"))
        ),
        "matrix augassign diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
fn test_matrix_binop_has_unsupported_operator_code() {
    let source = "def main():\n    x: int = 1 @ 2\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected matrix binop unsupported operator error");

    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
                && error.message == "matrix multiplication operator (@) is not supported"
                && error.primary_range == Some(range_for(source, "1 @ 2"))
        ),
        "matrix binop diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
fn test_unsupported_expression_form_has_type_code() {
    let source = "def main():\n    x = (yield 1)\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected unsupported expression form error");

    assert!(
        errors.iter().any(|error| error.code
            == Some(DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM)
            && error.message == "unsupported expression form: unsupported expression type"
            && error.primary_range == Some(range_for(source, "yield 1"))),
        "unsupported expression form diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
fn test_in_operator_non_collection_has_unsupported_operator_code() {
    let source = "def main() -> bool:\n    return 1 in 2\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected unsupported in operator error");

    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
                && error.message == "unsupported operator in for int"
                && error.primary_range == Some(range_for_after(source, " in ", "2"))
        ),
        "in operator diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
fn test_dict_unpacking_has_type_code() {
    let source = "def main():\n    other: dict[str, int] = {}\n    merged = {\"a\": 1, **other}\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected dict unpacking error");
    assert!(errors.iter().any(|error| {
        error.message == "dict unpacking (**) not supported"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(source, "**", "other"))
    }));
}

#[test]
fn test_tuple_slice_errors_have_type_codes() {
    let out_of_range_source =
        "def main():\n    pair: tuple[int, str] = (1, \"x\")\n    _bad = pair[0:3]\n";
    let out_of_range_result = lower_source(out_of_range_source);
    let out_of_range_errors = out_of_range_result.expect_err("expected tuple slice range error");
    assert!(out_of_range_errors.iter().any(|error| {
        error.message == "tuple slice indices out of range"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range
                == Some(range_for_after_anchor(out_of_range_source, "pair[", "0:3"))
    }));

    let dynamic_source =
        "def main():\n    pair: tuple[int, str] = (1, \"x\")\n    start: int = 0\n    _bad = pair[start:2]\n";
    let dynamic_result = lower_source(dynamic_source);
    let dynamic_errors = dynamic_result.expect_err("expected tuple dynamic slice error");
    assert!(dynamic_errors.iter().any(|error| {
        error.message == "tuple slicing requires compile-time constant indices"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range
                == Some(range_for_after_anchor(dynamic_source, "pair[", "start:2"))
    }));
}

#[test]
fn test_unsupported_slice_receiver_has_type_code() {
    let source = "def main():\n    value: int = 1\n    _bad = value[0:1]\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected unsupported slice receiver error");
    assert!(errors.iter().any(|error| {
        error.message == "cannot slice type 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(source, "value[", "0:1"))
    }));
}

#[test]
fn test_augassign_complex_targets_have_type_codes() {
    let cases = [
        (
            "attribute receiver",
            "def make_box() -> int:\n    return 1\n\ndef bad() -> None:\n    make_box().field += 1\n",
            "augmented attribute assignment target must be a simple name",
            "make_box()",
        ),
        (
            "subscript receiver",
            "def make_items() -> list[int]:\n    return [1]\n\ndef bad() -> None:\n    make_items()[0] += 1\n",
            "augmented subscript assignment target must be a simple name",
            "make_items()",
        ),
        (
            "attribute subscript receiver",
            "def make_box() -> int:\n    return 1\n\ndef bad() -> None:\n    make_box().counts[0] += 1\n",
            "augmented subscript assignment target must be a simple name",
            "make_box()",
        ),
        (
            "nested subscript receiver",
            "def make_grid() -> list[list[int]]:\n    return [[1]]\n\ndef bad() -> None:\n    make_grid()[0][0] += 1\n",
            "augmented subscript assignment target must be a simple name",
            "make_grid()",
        ),
        (
            "nested subscript expression receiver",
            "def bad(mut xs: list[list[int]], mut ys: list[list[int]]) -> None:\n    (xs + ys)[0][0] += 1\n",
            "augmented subscript assignment target must be a simple name",
            "xs + ys",
        ),
    ];

    for (label, source, message, range_needle) in cases {
        let result = lower_source(source);
        let errors = result.expect_err("expected complex augassign target error");
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                    && error.message == message
                    && error.primary_range
                        == Some(range_for_after(source, ") -> None:\n    ", range_needle))),
            "{label} diagnostic should be structured and ranged: {errors:?}"
        );
    }
}

#[test]
fn test_bytes_subscript_assignment_has_ownership_code() {
    let source = "def main() -> None:\n    payload: bytes = b\"abc\"\n    payload[0] = 65\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected bytes subscript assignment error");

    assert!(
        errors.iter().any(|error| error.code
            == Some(DiagnosticCode::OWN_IMMUTABLE_BYTES_ASSIGNMENT)
            && error.message == "bytes is immutable; subscript assignment is not supported"
            && error.primary_range == Some(range_for(source, "payload[0]"))),
        "bytes subscript assignment should preserve ownership code: {errors:?}"
    );
}

#[test]
fn test_bytes_augmented_subscript_assignment_has_ownership_code() {
    let source = "def main() -> None:\n    payload: bytes = b\"abc\"\n    payload[0] += 1\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected bytes augmented subscript assignment error");

    assert!(
        errors.iter().any(|error| error.code
            == Some(DiagnosticCode::OWN_IMMUTABLE_BYTES_AUGMENTED_ASSIGNMENT)
            && error.message
                == "bytes is immutable; augmented subscript assignment is not supported"
            && error.primary_range == Some(range_for(source, "payload[0]"))),
        "bytes augmented subscript assignment should preserve ownership code: {errors:?}"
    );
}

#[test]
fn test_bytes_codec_type_errors_have_structured_codes() {
    let encode_source = "def main() -> None:\n    _bad: bytes = \"abc\".encode(1)\n";
    let encode_result = lower_source(encode_source);
    let encode_errors = encode_result.expect_err("expected str.encode codec type error");
    assert!(
        encode_errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message == "str.encode() encoding must be 'str', got 'int'"
                && error.primary_range == Some(range_for(encode_source, "1"))),
        "str.encode codec type diagnostic should be structured and ranged: {encode_errors:?}"
    );

    let decode_source = "def main() -> None:\n    _bad: str = b\"abc\".decode(1)\n";
    let decode_result = lower_source(decode_source);
    let decode_errors = decode_result.expect_err("expected bytes.decode codec type error");
    assert!(
        decode_errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message == "bytes.decode() encoding must be 'str', got 'int'"
                && error.primary_range == Some(range_for(decode_source, "1"))),
        "bytes.decode codec type diagnostic should be structured and ranged: {decode_errors:?}"
    );
}

#[test]
fn test_decimal_method_surface_errors_have_structured_codes() {
    let arity_source =
        "def main() -> None:\n    d: decimal = Decimal(\"1.25\")\n    _bad: decimal = d.sqrt(1)\n";
    let arity_result = lower_source(arity_source);
    let arity_errors = arity_result.expect_err("expected decimal.sqrt arity error");
    assert!(
        arity_errors.iter().any(|error| error.code
            == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.message == "decimal.sqrt() takes no arguments"
            && error.primary_range == Some(range_for_after_anchor(arity_source, "sqrt(", "1"))),
        "decimal.sqrt arity diagnostic should be structured and ranged: {arity_errors:?}"
    );

    let method_source = "def main() -> None:\n    d: decimal = Decimal(\"1.25\")\n    _bad: decimal = d.magnitude()\n";
    let method_result = lower_source(method_source);
    let method_errors = method_result.expect_err("expected decimal unknown method error");
    assert!(
        method_errors.iter().any(|error| error.code
            == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.message == "type 'decimal' has no method 'magnitude'"
            && error.primary_range == Some(range_for(method_source, "magnitude"))),
        "decimal unknown method diagnostic should be structured and ranged: {method_errors:?}"
    );
}

#[test]
fn test_list_subscript_augassign_type_error_keeps_code() {
    let source = "def bad(mut xs: list[int]) -> None:\n    xs[0] += \"x\"\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected list subscript augassign type error");

    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
                && error.message.contains("unsupported operand type(s) for +")
                && error.primary_range == Some(range_for(source, "\"x\""))
        ),
        "list subscript augassign should preserve the operator helper code: {errors:?}"
    );
}

#[test]
fn test_dict_subscript_augassign_type_error_keeps_code() {
    let source =
        "def bad(mut data: dict[str, int]) -> None:\n    data[\"x\"] = 1\n    data[\"x\"] += \"x\"\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected dict subscript augassign type error");

    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
                && error.message.contains("unsupported operand type(s) for +")
                && error.primary_range == Some(range_for_after(source, "+= ", "\"x\""))
        ),
        "dict subscript augassign should preserve the operator helper code: {errors:?}"
    );
}

#[test]
fn test_list_subscript_assignment_index_error_has_type_code() {
    let source = "def bad(mut xs: list[int]) -> None:\n    xs[\"0\"] = 1\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected list subscript assignment index error");

    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message == "list subscript assignment index must be 'int', got 'str'"
                && error.primary_range == Some(range_for(source, "xs[\"0\"]"))),
        "list subscript assignment index diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
fn test_list_subscript_assignment_value_error_has_type_code() {
    let source = "def bad(mut xs: list[int]) -> None:\n    xs[0] = \"x\"\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected list subscript assignment value error");

    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message
                    == "list subscript assignment value type 'str' is not compatible with list element type 'int'"
                && error.primary_range == Some(range_for(source, "xs[0]"))),
        "list subscript assignment value diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
fn test_unsupported_subscript_assignment_has_type_code() {
    let source = "def bad(mut value: int) -> None:\n    value[0] = 1\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected unsupported subscript assignment error");

    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message == "subscript assignment is not supported for type 'int'"
                && error.primary_range == Some(range_for(source, "value[0]"))),
        "unsupported subscript assignment diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
fn test_unsupported_subscript_augassign_has_type_code() {
    let source = "def bad(mut value: int) -> None:\n    value[0] += 1\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected unsupported subscript augassign error");

    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message
                    == "augmented subscript assignment is not supported for type 'int'"
                && error.primary_range == Some(range_for(source, "value[0]"))),
        "unsupported subscript augassign diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
fn test_tuple_index_out_of_range_has_type_code() {
    let source = "def main():\n    pair: tuple[int, str] = (1, \"x\")\n    value: int = pair[2]\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected tuple index error");

    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message == "tuple index out of range"
                && error.primary_range == Some(range_for(source, "2"))),
        "tuple index diagnostic should preserve type code and literal index range: {errors:?}"
    );
}

#[test]
fn test_invalid_subscript_receiver_has_type_code() {
    let source = "def main():\n    value: int = 1\n    bad: int = value[0]\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected invalid subscript receiver error");

    assert!(
        errors.iter().any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message == "cannot index type 'int' with 'int'"
            && error.primary_range == Some(range_for_after_anchor(source, "bad: int = ", "value[0]"))),
        "invalid subscript receiver diagnostic should preserve type code and subscript range: {errors:?}"
    );
}

#[test]
fn test_nested_attribute_assignment_target_lowers_for_self_fields() {
    let result = lower_source(
        "class ListNode:\n    next: ListNode | None\n\n    def __init__(self):\n        self.next = None\n\nclass Wrapper:\n    head: ListNode\n\n    def __init__(self):\n        self.head = ListNode()\n        self.head.next = ListNode()\n",
    );
    assert!(
        result.is_ok(),
        "nested attribute assignment on class fields should lower: {:?}",
        result.err()
    );
}

#[test]
fn test_nested_attribute_assignment_lowers_for_optional_field_base() {
    let result = lower_source(
        "class ListNode:\n    next: ListNode | None\n    prev: ListNode | None\n\n    def __init__(self):\n        self.next = None\n        self.prev = None\n\ndef relink(mut node: ListNode) -> None:\n    if node.prev is not None:\n        node.prev.next = node.next\n",
    );
    assert!(
        result.is_ok(),
        "nested attribute assignment through optional field bases should lower under explicit narrowing: {:?}",
        result.err()
    );
}

#[test]
fn test_empty_list_specializes_on_append_and_satisfies_return_type() {
    let result = lower_source(
        "def collect() -> list[int]:\n    res = []\n    res.append(1)\n    return res\n",
    );
    assert!(
        result.is_ok(),
        "empty list should specialize to list[int] after append"
    );
}

#[test]
fn test_empty_list_specializes_on_insert_and_extend() {
    let result = lower_source(
        "def collect() -> list[int]:\n    res = []\n    res.insert(0, 1)\n    res.extend([2, 3])\n    return res\n",
    );
    assert!(
        result.is_ok(),
        "empty list should specialize from insert/extend element types"
    );
}

#[test]
fn test_empty_list_specialization_rejects_mixed_append_types() {
    let result = lower_source(
        "def collect() -> list[int]:\n    res = []\n    res.append(1)\n    res.append(\"x\")\n    return res\n",
    );
    assert!(
        result.is_err(),
        "after first append specialization, mixed element types must fail"
    );
}

#[test]
fn test_empty_list_specialization_survives_loop_append() {
    let result = lower_source(
        "def collect(values: list[int]) -> list[int]:\n    res = []\n    i = 0\n    while i < len(values):\n        res.append(values[i])\n        i += 1\n    return res\n",
    );
    assert!(
        result.is_ok(),
        "loop-body append specialization should persist so return boundary sees list[int]"
    );
}

#[test]
fn test_generic_class_receiver_refines_from_method_arguments() {
    let result = lower_source(
        "class Bucket[T]:\n    items: list[T]\n\n    def __init__(self):\n        self.items = []\n\n    def push(self, value: T) -> None:\n        self.items.append(value)\n\n    def first(self) -> T | None:\n        if len(self.items) == 0:\n            return None\n        return self.items[0]\n\ndef main() -> int:\n    bucket = Bucket()\n    bucket.push(1)\n    value = bucket.first()\n    if value is None:\n        return 0\n    return value + 1\n",
    );
    assert!(
        result.is_ok(),
        "receiver generic type vars should refine from method arguments"
    );
}

#[test]
fn test_generic_class_receiver_refinement_rejects_mixed_argument_types() {
    let result = lower_source(
        "class Bucket[T]:\n    items: list[T]\n\n    def __init__(self):\n        self.items = []\n\n    def push(self, value: T) -> None:\n        self.items.append(value)\n\ndef main() -> None:\n    bucket = Bucket()\n    bucket.push(1)\n    bucket.push(\"x\")\n",
    );
    assert!(
        result.is_err(),
        "once method-driven specialization binds T, incompatible argument types must fail"
    );
}

#[test]
fn test_empty_list_specialization_optional_append_in_loop_rejects_return_annotation() {
    let result = lower_source(
        "def collect(matrix: list[list[int]]) -> list[int]:\n    res = []\n    i = 0\n    while i < len(matrix):\n        res.append(matrix[i][0])\n        i += 1\n    return res\n",
    );
    assert!(
        result.is_err(),
        "optional element append should specialize to list[int|None] and fail list[int] return annotation"
    );
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("return type mismatch")));
}

#[test]
fn test_copy_type_no_move() {
    let module =
        lower_source("def main():\n    x: int = 42\n    print(x)\n    print(x)\n").unwrap();
    assert_eq!(module.functions.len(), 1);
}

#[test]
fn test_while_loop() {
    let module =
        lower_source("def main():\n    i: int = 0\n    while i < 10:\n        i = i + 1\n")
            .unwrap();
    assert_eq!(module.functions.len(), 1);
    assert!(module.functions[0].body.len() >= 2);
    assert!(matches!(module.functions[0].body[1], HirStmt::While { .. }));
}

#[test]
fn test_if_else_branch_bindings_are_visible_after_if() {
    let result = lower_source(
        "def main(flag: bool) -> int:\n    if flag:\n        value = 1\n    else:\n        value = 2\n    return value\n",
    );
    assert!(
        result.is_ok(),
        "exhaustive if/else branch-local bindings should be visible after the conditional: {:?}",
        result.err()
    );
}

#[test]
fn test_if_condition_rejects_numeric_truthiness() {
    let source = "def main():\n    if 1:\n        pass\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("if condition must be bool or collection/string truthiness")
            && e.code == Some(DiagnosticCode::FLOW_INVALID_CONDITION_TYPE)
            && e.primary_range == Some(range_for(source, "1"))
    }));
}

#[test]
fn test_while_condition_rejects_numeric_truthiness() {
    let source = "def main():\n    while 1:\n        return\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("while condition must be bool or collection/string truthiness")
            && e.code == Some(DiagnosticCode::FLOW_INVALID_CONDITION_TYPE)
            && e.primary_range == Some(range_for(source, "1"))
    }));
}

#[test]
fn test_elif_condition_rejects_numeric_truthiness_with_primary_range() {
    let source = "def main(flag: bool):\n    if flag:\n        pass\n    elif 1:\n        pass\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("elif condition must be bool or collection/string truthiness")
            && e.code == Some(DiagnosticCode::FLOW_INVALID_CONDITION_TYPE)
            && e.primary_range == Some(range_for(source, "1"))
    }));
}

#[test]
fn test_class_truthiness_allowed_in_if_while_and_boolop() {
    let result = lower_source(
        "class Node:\n    val: int\n    def __init__(self, val: int):\n        self.val = val\n\ndef probe(a: Node, b: Node) -> bool:\n    seen: bool = False\n    if a:\n        seen = True\n    while b:\n        break\n    return a and b and seen\n",
    );
    assert!(
        result.is_ok(),
        "class instances should be valid truthiness operands in control-flow and boolops"
    );
}

#[test]
fn test_non_none_return_annotation_requires_exhaustive_returns() {
    let source = "def f(flag: bool) -> int:\n    if flag:\n        return 1\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("must return a value of type 'int' on all control-flow paths")
            && e.code == Some(DiagnosticCode::FLOW_MISSING_RETURN_VALUE)
            && e.primary_range == Some(range_for_after_anchor(source, "def ", "f"))
    }));
}

#[test]
fn test_invalid_return_expression_does_not_emit_missing_return_cascade() {
    let result = lower_source("def main(xs: list[int]) -> int:\n    return xs[0] + xs[0]\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("unsupported operand type(s) for +")));
    assert!(
        !errors.iter().any(|e| {
            e.message
                .contains("must return a value of type 'int' on all control-flow paths")
        }),
        "invalid return expressions should not trigger a return-completeness cascade: {errors:?}"
    );
}

#[test]
fn test_duplicate_module_function_definition_reports_error() {
    let source = "def same() -> bool:\n    return True\n\ndef same() -> bool:\n    return False\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("duplicate function definition in module: 'same'")
        && e.code == Some(DiagnosticCode::NAME_DUPLICATE_DEFINITION)
        && e.primary_range == Some(range_for_after(source, "\n\ndef ", "same"))));
}

#[test]
fn test_guarded_list_pop_narrows_to_element_type() {
    let result = lower_source(
        "def main():\n    values: list[int] = [1, 2]\n    while values:\n        item: int = values.pop()\n",
    );
    assert!(
        result.is_ok(),
        "list.pop() under non-empty guard should narrow to element type"
    );
}

#[test]
fn test_guarded_zero_index_list_pop_narrows_to_element_type() {
    let result = lower_source(
        "def main():\n    values: list[int] = [1, 2]\n    while values:\n        item: int = values.pop(0)\n",
    );
    assert!(
        result.is_ok(),
        "list.pop(0) under non-empty guard should narrow to element type"
    );
}

#[test]
fn test_guarded_list_pop_on_field_access_narrows_to_element_type() {
    let result = lower_source(
        "class Q:\n    data: list[int]\n\n    def __init__(self):\n        self.data = [1, 2]\n\n    def pop_one(self) -> int:\n        while self.data:\n            item: int = self.data.pop()\n            return item\n        return 0\n",
    );
    assert!(
        result.is_ok(),
        "list.pop() under non-empty field guard should narrow to element type"
    );
}

#[test]
fn test_guarded_list_pop_preserves_optional_element_none() {
    let result = lower_source(
        "def main():\n    values: list[int | None] = []\n    values.append(1)\n    values.append(None)\n    while values:\n        item: int | None = values.pop()\n",
    );
    assert!(
        result.is_ok(),
        "list.pop() under non-empty guard should keep element-level optionality"
    );
}

#[test]
fn test_guarded_list_pop_optional_element_rejects_non_optional_annotation() {
    let result = lower_source(
        "def main():\n    values: list[int | None] = []\n    values.append(1)\n    values.append(None)\n    while values:\n        item: int = values.pop()\n",
    );
    assert!(
        result.is_err(),
        "non-empty guard must not erase element-level None from list[int|None].pop()"
    );
}

#[test]
fn test_unguarded_list_pop_stays_optional() {
    let result =
        lower_source("def main():\n    values: list[int] = [1, 2]\n    item: int = values.pop()\n");
    assert!(
        result.is_err(),
        "unguarded list.pop() should remain optional"
    );
}

#[test]
fn test_unguarded_zero_index_list_pop_stays_optional() {
    let result = lower_source(
        "def main():\n    values: list[int] = [1, 2]\n    item: int = values.pop(0)\n",
    );
    assert!(
        result.is_err(),
        "unguarded list.pop(0) should remain optional"
    );
}

#[test]
fn test_guarded_indexed_list_pop_stays_optional() {
    let result = lower_source(
        "def main():\n    values: list[int] = [1, 2]\n    while values:\n        item: int = values.pop(5)\n",
    );
    assert!(
        result.is_err(),
        "indexed list.pop(i) should remain optional under non-empty guard"
    );
}

#[test]
fn test_guarded_dict_pop_stays_optional() {
    let result = lower_source(
        "def main():\n    values: dict[str, int] = {\"x\": 1}\n    if values:\n        item: int = values.pop(\"missing\")\n",
    );
    assert!(
        result.is_err(),
        "dict.pop(key) should remain optional under dict truthiness guard"
    );
}

#[test]
fn test_boolop_and_short_circuit_narrows_guarded_index_operand() {
    let result = lower_source(
        "def main():\n    values: list[int] = [1, 2, 3]\n    i: int = 1\n    ok: bool = i < len(values) and values[i] > 0\n    assert ok == True\n",
    );
    assert!(
        result.is_ok(),
        "`and` short-circuit should apply sequence guard facts to the RHS operand"
    );
}

#[test]
fn test_boolop_or_short_circuit_narrows_rhs_after_not_empty_guard() {
    let result = lower_source(
        "def probe(stack: list[int]) -> bool:\n    return not stack or stack[0] > 0\n\ndef main():\n    assert probe([]) == True\n    assert probe([1, 2]) == True\n",
    );
    assert!(
        result.is_ok(),
        "`or` short-circuit should apply false-branch guard facts to the RHS operand"
    );
}

#[test]
fn test_boolop_and_without_sequence_guard_keeps_optional_index_error() {
    let result = lower_source(
        "def read_without_len_guard(values: list[int], i: int) -> int:\n    if True and i >= 0:\n        value: int = values[i]\n        return value\n    return 0\n",
    );
    assert!(
        result.is_err(),
        "`and` without an explicit sequence guard should not narrow index access"
    );
}

#[test]
fn test_tuple_literal_index_uses_exact_position_type() {
    let result = lower_source(
        "def main() -> int:\n    pair: tuple[str, int] = (\"x\", 7)\n    value: int = pair[1]\n    return value\n",
    );
    assert!(
        result.is_ok(),
        "tuple[1] should resolve to the second element type"
    );
}

#[test]
fn test_tuple_nonliteral_index_uses_union_of_element_types() {
    let result = lower_source(
        "def bad(i: int) -> int:\n    pair: tuple[int, str] = (1, \"x\")\n    value: int = pair[i]\n    return value\n",
    );
    assert!(
        result.is_err(),
        "non-literal tuple index should be typed as a union of element types"
    );
}

#[test]
fn test_if_expr_optional_branch_does_not_implicitly_unwrap() {
    let result = lower_source(
        "def pick(x: int | None) -> int:\n    value: int = x if x is not None else 0\n    return value\n",
    );
    assert!(
        result.is_err(),
        "ternary optional branch should not implicitly unwrap Option values"
    );
}

#[test]
fn test_if_expr_true_branch_sequence_guard_narrows_index() {
    let result = lower_source(
        "def pick(values: list[int], i: int) -> int:\n    value: int = values[i] if i < len(values) else 0\n    return value\n",
    );
    assert!(
        result.is_ok(),
        "ternary true branch should honor index guard and produce definite element type"
    );
}

#[test]
fn test_if_expr_true_branch_sequence_guard_narrows_index_with_offset() {
    let result = lower_source(
        "def pick(values: list[int], i: int) -> int:\n    value: int = values[i + 1] if i + 1 < len(values) else 0\n    return value\n",
    );
    assert!(
        result.is_ok(),
        "ternary true branch should honor offset index guard and produce definite element type"
    );
}

#[test]
fn test_while_not_none_narrows_optional_receiver_for_attribute_access() {
    let result = lower_source(
        "class Node:\n    val: int\n    next: Node | None\n\n    def __init__(self, val: int, next: Node | None):\n        self.val = val\n        self.next = next\n\ndef total(own head: Node | None) -> int:\n    cur: Node | None = head\n    acc: int = 0\n    while cur is not None:\n        acc = acc + cur.val\n        cur = cur.next\n    return acc\n",
    );
    assert!(
        result.is_ok(),
        "`while x is not None` should narrow optional receivers inside the loop body"
    );
}

#[test]
fn test_inferred_local_can_widen_to_optional_on_reassignment() {
    let result = lower_source(
        "def pick(head: int | None) -> int:\n    cur = None\n    cur = head\n    if cur is None:\n        return 0\n    return cur\n",
    );
    assert!(
        result.is_ok(),
        "inferred locals should widen to Optional under reassignment from/to None"
    );
}

#[test]
fn test_optional_reassignment_invalidates_non_none_narrowing() {
    let result = lower_source(
        "def bad(mut x: int | None) -> int:\n    if x is not None:\n        x = None\n        return x\n    return 0\n",
    );
    assert!(
        result.is_err(),
        "rebinding an Optional must invalidate prior non-None narrowing"
    );
}

#[test]
fn test_sequence_reassignment_invalidates_index_guard() {
    let result = lower_source(
        "def bad(mut values: list[int], i: int) -> int:\n    if i < len(values):\n        values = []\n        value: int = values[i]\n        return value\n    return 0\n",
    );
    assert!(
        result.is_err(),
        "rebinding a guarded sequence must invalidate prior index facts"
    );
}

#[test]
fn test_index_reassignment_invalidates_index_guard() {
    let result = lower_source(
        "def bad(values: list[int], mut i: int) -> int:\n    if i < len(values):\n        i = len(values)\n        value: int = values[i]\n        return value\n    return 0\n",
    );
    assert!(
        result.is_err(),
        "rebinding a guarded index variable must invalidate prior index facts"
    );
}

#[test]
fn test_shrinking_collection_method_invalidates_index_guard() {
    let result = lower_source(
        "def bad(mut values: list[int], i: int) -> int:\n    if i < len(values):\n        values.clear()\n        value: int = values[i]\n        return value\n    return 0\n",
    );
    assert!(
        result.is_err(),
        "a shrinking collection mutation must invalidate prior index facts"
    );
}

#[test]
fn test_shrinking_field_collection_method_invalidates_index_guard() {
    let result = lower_source(
        "class Box:\n    values: list[int]\n\n    def __init__(self, values: list[int]):\n        self.values = values\n\n    def bad(mut self, i: int) -> int:\n        if i < len(self.values):\n            self.values.clear()\n            value: int = self.values[i]\n            return value\n        return 0\n",
    );
    assert!(
        result.is_err(),
        "a shrinking collection mutation on a field must invalidate field index facts"
    );
}

#[test]
fn test_annotated_local_does_not_widen_on_reassignment() {
    let result =
        lower_source("def bad() -> int:\n    value: int = 1\n    value = None\n    return value\n");
    assert!(
        result.is_err(),
        "explicitly annotated locals should keep their declared type on reassignment"
    );
}

#[test]
fn test_for_range() {
    let module = lower_source("def main():\n    for i in range(10):\n        print(i)\n").unwrap();
    assert_eq!(module.functions.len(), 1);
    assert!(matches!(module.functions[0].body[0], HirStmt::For { .. }));
}

#[test]
fn test_for_range_start_end() {
    let module =
        lower_source("def main():\n    for i in range(1, 5):\n        print(i)\n").unwrap();
    assert_eq!(module.functions.len(), 1);
    assert!(matches!(module.functions[0].body[0], HirStmt::For { .. }));
}

#[test]
fn test_tuple_unpack_len_alias_enables_range_index_guard() {
    let result = lower_source(
        "def sum_all(values: list[int]) -> int:\n    start, n = (0, len(values))\n    total: int = 0\n    for i in range(start, n):\n        total = total + values[i]\n    return total\n",
    );
    assert!(
        result.is_ok(),
        "tuple-unpacked len aliases should feed range-based index guards"
    );
}

#[test]
fn test_tuple_unpack_non_len_alias_does_not_enable_range_index_guard() {
    let result = lower_source(
        "def sum_all(values: list[int], n: int) -> int:\n    start, limit = (0, n)\n    total: int = 0\n    for i in range(start, limit):\n        total = total + values[i]\n    return total\n",
    );
    assert!(
        result.is_err(),
        "range-based index guards must not activate for tuple-unpacked non-len aliases"
    );
}

#[test]
fn test_for_loop_lowers_through_iter_protocol_call() {
    let module = lower_source(
        "def main():\n    values: list[int] = [1, 2]\n    for x in values:\n        print(x)\n",
    )
    .unwrap();
    let for_stmt = module.functions[0]
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::For { .. }))
        .expect("expected for loop");
    let HirStmt::For { iter, .. } = for_stmt else {
        unreachable!("matched for loop above")
    };
    assert!(matches!(
        iter,
        HirExpr::IteratorCall { op, args, ty }
            if op == &crate::hir_nodes::HirIteratorOp::Iter
                && args.len() == 1
                && matches!(ty, Type::Iterator(_))
    ));
}

#[test]
fn test_iterator_builtins_lower_to_canonical_iterator_call_nodes() {
    fn call_uses_legacy_iterator_builtin(expr: &HirExpr) -> bool {
        let legacy = [
            "iter",
            "next",
            "reversed",
            "map",
            "filter",
            "zip",
            "enumerate",
        ];
        match expr {
            HirExpr::Call { func, args, .. } => {
                legacy.contains(&func.as_str()) || args.iter().any(call_uses_legacy_iterator_builtin)
            }
            HirExpr::IteratorCall { args, .. }
            | HirExpr::ListLiteral { elements: args, .. }
            | HirExpr::SetLiteral { elements: args, .. }
            | HirExpr::TupleLiteral { elements: args, .. }
            | HirExpr::BoolOp { values: args, .. } => {
                args.iter().any(call_uses_legacy_iterator_builtin)
            }
            HirExpr::BinOp { left, right, .. } => {
                call_uses_legacy_iterator_builtin(left) || call_uses_legacy_iterator_builtin(right)
            }
            HirExpr::UnaryOp { operand, .. }
            | HirExpr::QuestionMark { expr: operand, .. }
            | HirExpr::OkWrap { value: operand, .. }
            | HirExpr::ErrWrap { value: operand, .. }
            | HirExpr::WalrusExpr { value: operand, .. }
            | HirExpr::FieldAccess {
                object: operand, ..
            } => call_uses_legacy_iterator_builtin(operand),
            HirExpr::Compare {
                left, comparators, ..
            } => {
                call_uses_legacy_iterator_builtin(left)
                    || comparators
                        .iter()
                        .any(call_uses_legacy_iterator_builtin)
            }
            HirExpr::IfExpr {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                call_uses_legacy_iterator_builtin(condition)
                    || call_uses_legacy_iterator_builtin(then_expr)
                    || call_uses_legacy_iterator_builtin(else_expr)
            }
            HirExpr::RangeLiteral {
                start, end, step, ..
            } => {
                call_uses_legacy_iterator_builtin(start)
                    || call_uses_legacy_iterator_builtin(end)
                    || step
                        .as_ref()
                        .is_some_and(|expr| call_uses_legacy_iterator_builtin(expr))
            }
            HirExpr::DictLiteral { keys, values, .. } => {
                keys.iter().any(call_uses_legacy_iterator_builtin)
                    || values.iter().any(call_uses_legacy_iterator_builtin)
            }
            HirExpr::Index { object, index, .. } => {
                call_uses_legacy_iterator_builtin(object)
                    || call_uses_legacy_iterator_builtin(index)
            }
            HirExpr::MethodCall { object, args, .. } => {
                call_uses_legacy_iterator_builtin(object)
                    || args.iter().any(call_uses_legacy_iterator_builtin)
            }
            HirExpr::ConstructorCall { args, .. } | HirExpr::SuperCall { args, .. } => {
                args.iter().any(call_uses_legacy_iterator_builtin)
            }
            HirExpr::Slice {
                object,
                start,
                stop,
                step,
                ..
            } => {
                call_uses_legacy_iterator_builtin(object)
                    || start
                        .as_ref()
                        .is_some_and(|expr| call_uses_legacy_iterator_builtin(expr))
                    || stop
                        .as_ref()
                        .is_some_and(|expr| call_uses_legacy_iterator_builtin(expr))
                    || step
                        .as_ref()
                        .is_some_and(|expr| call_uses_legacy_iterator_builtin(expr))
            }
            HirExpr::GeneratorExpr {
                expr, iter, filter, ..
            } => {
                call_uses_legacy_iterator_builtin(expr)
                    || call_uses_legacy_iterator_builtin(iter)
                    || filter
                        .as_ref()
                        .is_some_and(|expr| call_uses_legacy_iterator_builtin(expr))
            }
            HirExpr::ListComp {
                expr, generators, ..
            }
            | HirExpr::SetComp {
                expr, generators, ..
            } => {
                call_uses_legacy_iterator_builtin(expr)
                    || generators.iter().any(|(_, iter, filter)| {
                        call_uses_legacy_iterator_builtin(iter)
                            || filter
                                .as_ref()
                                .is_some_and(call_uses_legacy_iterator_builtin)
                    })
            }
            HirExpr::DictComp {
                key_expr,
                val_expr,
                generators,
                ..
            } => {
                call_uses_legacy_iterator_builtin(key_expr)
                    || call_uses_legacy_iterator_builtin(val_expr)
                    || generators.iter().any(|(_, iter, filter)| {
                        call_uses_legacy_iterator_builtin(iter)
                            || filter
                                .as_ref()
                                .is_some_and(call_uses_legacy_iterator_builtin)
                    })
            }
            HirExpr::FString { parts, .. } => parts.iter().any(|part| {
                matches!(part, crate::hir_nodes::HirFStringPart::Expr(expr) if call_uses_legacy_iterator_builtin(expr))
            }),
            HirExpr::EnumVariant { .. }
            | HirExpr::Name { .. }
            | HirExpr::IntLiteral(_)
            | HirExpr::LargeIntLiteral(_)
            | HirExpr::FloatLiteral(_)
            | HirExpr::StringLiteral(_)
            | HirExpr::BoolLiteral(_)
            | HirExpr::NoneLiteral
            | HirExpr::ContainsOp { .. }
            | HirExpr::Lambda { .. } => false,
        }
    }

    let module = lower_source(
        "def add(x: int, y: int) -> int:\n    return x + y\n\ndef pred(x: int) -> bool:\n    return x % 2 == 0\n\ndef main():\n    nums: list[int] = [1, 2, 3]\n    it: Iterator[int] = iter(nums)\n    first: int | None = next(it)\n    rev: Iterator[int] = reversed(nums)\n    indexed: Iterator[tuple[int, int]] = enumerate(nums)\n    zipped: Iterator[tuple[int, int]] = zip(nums, nums)\n    mapped: Iterator[int] = map(add, nums, nums)\n    filtered: Iterator[int] = filter(pred, nums)\n    list_comp: list[int] = [x for x in nums]\n    set_comp: set[int] = {x for x in nums}\n    dict_comp: dict[int, int] = {x: x for x in nums}\n    gen_expr: Iterator[int] = (x for x in nums)\n",
    )
    .unwrap();

    let main_fn = module
        .functions
        .iter()
        .find(|func| func.name == "main")
        .expect("main function should exist");

    for stmt in &main_fn.body {
        if let HirStmt::Let { value, .. } = stmt {
            assert!(
                !call_uses_legacy_iterator_builtin(value),
                "legacy iterator builtin call node found in canonical iterator lowering: {value:?}"
            );
        }
    }

    let mut saw_list_comp = false;
    let mut saw_set_comp = false;
    let mut saw_dict_comp = false;
    let mut saw_gen_expr = false;
    for stmt in &main_fn.body {
        let HirStmt::Let { name, value, .. } = stmt else {
            continue;
        };
        match (name.as_str(), value) {
            ("list_comp", HirExpr::ListComp { generators, .. }) => {
                saw_list_comp = true;
                assert!(generators.iter().all(|(_, iter, _)| {
                    matches!(
                        iter,
                        HirExpr::IteratorCall { op, args, .. }
                            if op == &crate::hir_nodes::HirIteratorOp::Iter && args.len() == 1
                    )
                }));
            }
            ("set_comp", HirExpr::SetComp { generators, .. }) => {
                saw_set_comp = true;
                assert!(generators.iter().all(|(_, iter, _)| {
                    matches!(
                        iter,
                        HirExpr::IteratorCall { op, args, .. }
                            if op == &crate::hir_nodes::HirIteratorOp::Iter && args.len() == 1
                    )
                }));
            }
            ("dict_comp", HirExpr::DictComp { generators, .. }) => {
                saw_dict_comp = true;
                assert!(generators.iter().all(|(_, iter, _)| {
                    matches!(
                        iter,
                        HirExpr::IteratorCall { op, args, .. }
                            if op == &crate::hir_nodes::HirIteratorOp::Iter && args.len() == 1
                    )
                }));
            }
            ("gen_expr", HirExpr::GeneratorExpr { iter, .. }) => {
                saw_gen_expr = true;
                assert!(matches!(
                    iter.as_ref(),
                    HirExpr::IteratorCall { op, args, .. }
                        if op == &crate::hir_nodes::HirIteratorOp::Iter && args.len() == 1
                ));
            }
            _ => {}
        }
    }
    assert!(
        saw_list_comp,
        "list comprehension binding should be present"
    );
    assert!(saw_set_comp, "set comprehension binding should be present");
    assert!(
        saw_dict_comp,
        "dict comprehension binding should be present"
    );
    assert!(
        saw_gen_expr,
        "generator expression binding should be present"
    );
}

#[test]
fn test_iterable_annotation_accepts_list_argument() {
    let result = lower_source(
        "def consume(xs: Iterable[int]) -> int:\n    total: int = 0\n    for x in xs:\n        total = total + x\n    return total\n\ndef main():\n    values: list[int] = [1, 2, 3]\n    out: int = consume(values)\n    print(out)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_iterator_annotation_rejects_plain_list_argument() {
    let result = lower_source(
        "def consume_one(it: Iterator[int]) -> int:\n    return 1\n\ndef main():\n    values: list[int] = [1, 2, 3]\n    consume_one(values)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("expected 'Iterator[int]', got 'list[int]'")));
}

#[test]
fn test_iter_and_next_builtin_protocol_calls_lower() {
    let result = lower_source(
        "def main():\n    values: list[int] = [1, 2, 3]\n    it: Iterator[int] = iter(values)\n    first: int | None = next(it)\n    second: int | None = next(it)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_iter_accepts_homogeneous_tuple_argument() {
    let result = lower_source(
        "def main():\n    values: tuple[int, int, int] = (1, 2, 3)\n    it: Iterator[int] = iter(values)\n    first: int | None = next(it)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_iter_rejects_heterogeneous_tuple_argument() {
    let result = lower_source(
        "def main():\n    values: tuple[int, str] = (1, \"x\")\n    _it = iter(values)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("iter() tuple argument must have one statically provable element type")
    }));
}

#[test]
fn test_for_accepts_homogeneous_tuple_iterable() {
    let result = lower_source(
        "def main():\n    values: tuple[int, int, int] = (1, 2, 3)\n    total: int = 0\n    for value in values:\n        total = total + value\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_for_rejects_heterogeneous_tuple_iterable() {
    let result =
        lower_source("def main():\n    values: tuple[int, str] = (1, \"x\")\n    for value in values:\n        print(value)\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("for-loop tuple iteration requires one statically provable element type")
    }));
}

#[test]
fn test_next_rejects_plain_iterable_argument() {
    let result = lower_source("def main():\n    values: list[int] = [1, 2, 3]\n    next(values)\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("next() argument must be an iterator")));
}

#[test]
fn test_user_defined_iterable_class_participates_in_builtin_iteration_surface() {
    let result = lower_source(
        "class Boxed:\n    items: list[int]\n\n    def __init__(self, items: list[int]):\n        self.items = items\n\n    def __iter__(self) -> Iterator[int]:\n        return iter(self.items)\n\n    def __reversed__(self) -> Iterator[int]:\n        return reversed(self.items)\n\n\ndef main():\n    boxed: Boxed = Boxed([1, 2, 3])\n    vals: list[int] = list(boxed)\n    rev_vals: list[int] = list(reversed(boxed))\n    total: int = 0\n    for value in boxed:\n        total = total + value\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_next_accepts_user_defined_iterator_class() {
    let result = lower_source(
        "class CounterIter:\n    value: int\n\n    def __init__(self, start: int):\n        self.value = start\n\n    def __iter__(self) -> Iterator[int]:\n        return iter([self.value])\n\n    def __next__(self) -> int | None:\n        if self.value <= 0:\n            return None\n        out: int = self.value\n        self.value = self.value - 1\n        return out\n\n\ndef main():\n    it: CounterIter = CounterIter(2)\n    first: int | None = next(it)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_user_defined_iterable_protocol_rejects_invalid_iter_signature() {
    let result = lower_source("class BadIter:\n    def __iter__(self) -> int:\n        return 1\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("class 'BadIter.__iter__' must return 'Iterator[T]' or 'Iterable[T]'")
    }));
}

#[test]
fn test_user_defined_iterable_protocol_rejects_invalid_next_signature() {
    let result = lower_source(
        "class BadNext:\n    def __iter__(self) -> Iterator[int]:\n        return iter([1])\n\n    def __next__(self) -> int:\n        return 1\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("class 'BadNext.__next__' must return 'T | None'")
    }));
}

#[test]
fn test_for_rejects_mutation_of_collection_with_live_iterator() {
    let result = lower_source(
        "def main():\n    values: list[int] = [1, 2, 3]\n    for value in values:\n        values.append(value)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("cannot mutate 'values' while iterating over it in a for loop")
    }));
}

#[test]
fn test_generator_function_infers_iterator_return_type() {
    let module = lower_source(
        "def count_up(n: int):\n    i: int = 0\n    while i < n:\n        yield i\n        i = i + 1\n",
    )
    .unwrap();
    assert_eq!(
        module.functions[0].return_type,
        Type::Iterator(Box::new(Type::Int))
    );
}

#[test]
fn test_generator_function_rejects_non_iterator_annotation() {
    let source =
        "def count_up(n: int) -> list[int]:\n    i: int = 0\n    while i < n:\n        yield i\n        i = i + 1\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("must declare return type 'Iterator[T]'")
            && e.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && e.primary_range == Some(range_for_after_anchor(source, "-> ", "list[int]"))
    }));
}

#[test]
fn test_generator_expression_is_typed_as_iterator() {
    let module = lower_source(
        "def main():\n    nums: list[int] = [1, 2, 3]\n    g: Iterator[int] = (x * x for x in nums)\n    _first: int | None = next(g)\n",
    )
    .unwrap();
    let main_fn = module
        .functions
        .iter()
        .find(|func| func.name == "main")
        .expect("main function should exist");
    let Some(HirStmt::Let { ty, .. }) = main_fn
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::Let { name, .. } if name == "g"))
    else {
        panic!("expected let binding for generator expression");
    };
    assert!(matches!(ty, Type::Iterator(_)));
}

#[test]
fn test_generator_accepts_nested_yield_shapes() {
    let module = lower_source(
        "def nested(n: int):\n    i: int = 0\n    while i < n:\n        while i < n:\n            yield i\n            i = i + 1\n",
    )
    .unwrap();
    assert_eq!(
        module.functions[0].return_type,
        Type::Iterator(Box::new(Type::Int))
    );
}

#[test]
fn test_generator_accepts_trailing_statements_after_loop() {
    let module = lower_source(
        "def trailing(n: int):\n    i: int = 0\n    while i < n:\n        yield i\n        i = i + 1\n    i = i + 1\n",
    )
    .unwrap();
    assert_eq!(
        module.functions[0].return_type,
        Type::Iterator(Box::new(Type::Int))
    );
}

#[test]
fn test_reversed_enumerate_zip_are_typed_as_iterators() {
    let module = lower_source(
        "def main():\n    nums: list[int] = [1, 2, 3]\n    labels: list[str] = [\"a\", \"b\", \"c\"]\n    rev: Iterator[int] = reversed(nums)\n    indexed: Iterator[tuple[int, int]] = enumerate(nums, start=1)\n    paired: Iterator[tuple[int, str]] = zip(nums, labels)\n    _rev_list: list[int] = list(rev)\n    _indexed_list: list[tuple[int, int]] = list(indexed)\n    _paired_list: list[tuple[int, str]] = list(paired)\n",
    )
    .unwrap();
    let main_fn = module
        .functions
        .iter()
        .find(|func| func.name == "main")
        .expect("main function should exist");
    let Some(HirStmt::Let { ty, .. }) = main_fn
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::Let { name, .. } if name == "rev"))
    else {
        panic!("expected let binding for rev");
    };
    assert!(matches!(ty, Type::Iterator(_)));
    let Some(HirStmt::Let { ty, .. }) = main_fn
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::Let { name, .. } if name == "indexed"))
    else {
        panic!("expected let binding for indexed");
    };
    assert!(matches!(ty, Type::Iterator(_)));
    let Some(HirStmt::Let { ty, .. }) = main_fn
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::Let { name, .. } if name == "paired"))
    else {
        panic!("expected let binding for paired");
    };
    assert!(matches!(ty, Type::Iterator(_)));
}

#[test]
fn test_zip_keyword_diagnostics_are_stable() {
    let strict_result = lower_source(
        "def main():\n    nums: list[int] = [1, 2]\n    _paired = zip(nums, nums, strict=True)\n",
    );
    assert!(strict_result.is_err());
    let strict_errors = strict_result.unwrap_err();
    assert!(strict_errors.iter().any(|error| {
        error
            .message
            .contains("zip() keyword argument 'strict' is not supported")
    }));

    let unexpected_source =
        "def main():\n    nums: list[int] = [1, 2]\n    _paired = zip(nums, nums, bogus=True)\n";
    let unexpected_result = lower_source(unexpected_source);
    assert!(unexpected_result.is_err());
    let unexpected_errors = unexpected_result.unwrap_err();
    assert!(unexpected_errors.iter().any(|error| {
        error.message == "zip() got an unexpected keyword argument 'bogus'"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range
                == Some(range_for_after_anchor(
                    unexpected_source,
                    "zip(nums, nums, ",
                    "bogus",
                ))
    }));
}

#[test]
fn test_zip_non_iterable_argument_has_type_code() {
    let source = "def main():\n    nums: list[int] = [1, 2]\n    _paired = zip(nums, 1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "zip() argument 2 must be an iterable with a statically-known element type, got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(source, "zip(nums, ", "1"))
    }));
}

#[test]
fn test_any_all_wrong_arity_have_call_codes() {
    let any_source = "def main():\n    _value = any()\n";
    let any_result = lower_source(any_source);
    assert!(any_result.is_err());
    let any_errors = any_result.unwrap_err();
    assert!(any_errors.iter().any(|error| {
        error.message == "any() takes exactly 1 argument"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for(any_source, "any"))
    }));

    let all_source =
        "def main():\n    flags: list[bool] = [True]\n    _value = all(flags, flags)\n";
    let all_result = lower_source(all_source);
    assert!(all_result.is_err());
    let all_errors = all_result.unwrap_err();
    assert!(all_errors.iter().any(|error| {
        error.message == "all() takes exactly 1 argument"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range
                == Some(range_for_after_anchor(all_source, "all(flags, ", "flags"))
    }));
}

#[test]
fn test_range_and_enumerate_unexpected_keywords_have_call_code() {
    let range_source = "def main():\n    print(list(range(stop=3, bogus=1)))\n";
    let range_result = lower_source(range_source);
    assert!(range_result.is_err());
    let range_errors = range_result.unwrap_err();
    assert!(range_errors.iter().any(|error| {
        error.message == "range() got an unexpected keyword argument 'bogus'"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range
                == Some(range_for_after_anchor(range_source, "stop=3, ", "bogus"))
    }));

    let enumerate_source =
        "def main():\n    nums: list[int] = [1, 2]\n    _items = enumerate(nums, bogus=1)\n";
    let enumerate_result = lower_source(enumerate_source);
    assert!(enumerate_result.is_err());
    let enumerate_errors = enumerate_result.unwrap_err();
    assert!(enumerate_errors.iter().any(|error| {
        error.message == "enumerate() got an unexpected keyword argument 'bogus'"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range
                == Some(range_for_after_anchor(
                    enumerate_source,
                    "enumerate(nums, ",
                    "bogus",
                ))
    }));
}

#[test]
fn test_enumerate_duplicate_start_keyword_has_call_code() {
    let source = "\
def main():
    nums: list[int] = [1, 2]
    _items = enumerate(nums, 10, start=1)
";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "enumerate() got multiple values for argument 'start'"
            && error.code == Some(DiagnosticCode::CALL_DUPLICATE_ARGUMENT)
            && error.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "enumerate(nums, 10, ",
                    "start",
                ))
    }));
}

#[test]
fn test_reversed_rejects_non_reversible_iterator_argument() {
    let source =
        "def main():\n    nums: list[int] = [1, 2, 3]\n    it: Iterator[int] = iter(nums)\n    _rev = reversed(it)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "reversed() argument must be reversible, got 'Iterator[int]'"
            && error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
            && error.primary_range == Some(range_for_after_anchor(source, "reversed(", "it"))
    }));
}

#[test]
fn test_reversed_and_enumerate_argument_errors_have_codes() {
    let reversed_source = "def main():\n    _rev = reversed(1)\n";
    let reversed_result = lower_source(reversed_source);
    assert!(reversed_result.is_err());
    let reversed_errors = reversed_result.unwrap_err();
    assert!(reversed_errors.iter().any(|error| {
        error.message
            == "reversed() argument must be an iterable with a statically-known element type, got 'int'"
            && error.code == Some(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE)
            && error.primary_range == Some(range_for_after_anchor(reversed_source, "reversed(", "1"))
    }));

    let enumerate_source = "def main():\n    _items = enumerate(1)\n";
    let enumerate_result = lower_source(enumerate_source);
    assert!(enumerate_result.is_err());
    let enumerate_errors = enumerate_result.unwrap_err();
    assert!(enumerate_errors.iter().any(|error| {
        error.message
            == "enumerate() argument must be an iterable with a statically-known element type, got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(enumerate_source, "enumerate(", "1"))
    }));
}

#[test]
fn test_enumerate_start_type_errors_have_codes() {
    let positional_source =
        "def main():\n    nums: list[int] = [1, 2]\n    _items = enumerate(nums, \"bad\")\n";
    let positional_result = lower_source(positional_source);
    assert!(positional_result.is_err());
    let positional_errors = positional_result.unwrap_err();
    assert!(positional_errors.iter().any(|error| {
        error.message == "enumerate() start argument must be 'int', got 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range
                == Some(range_for_after_anchor(
                    positional_source,
                    "enumerate(nums, ",
                    "\"bad\"",
                ))
    }));

    let keyword_source =
        "def main():\n    nums: list[int] = [1, 2]\n    _items = enumerate(nums, start=\"bad\")\n";
    let keyword_result = lower_source(keyword_source);
    assert!(keyword_result.is_err());
    let keyword_errors = keyword_result.unwrap_err();
    assert!(keyword_errors.iter().any(|error| {
        error.message == "enumerate() keyword argument 'start' must be 'int', got 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range
                == Some(range_for_after_anchor(
                    keyword_source,
                    "enumerate(nums, start=",
                    "\"bad\"",
                ))
    }));
}

#[test]
fn test_enumerate_arity_and_unpacked_keyword_errors_have_codes() {
    let arity_source =
        "def main():\n    nums: list[int] = [1, 2]\n    _items = enumerate(nums, 1, 2)\n";
    let arity_result = lower_source(arity_source);
    assert!(arity_result.is_err());
    let arity_errors = arity_result.unwrap_err();
    assert!(arity_errors.iter().any(|error| {
        error.message == "enumerate() takes 1 or 2 arguments"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range
                == Some(range_for_after_anchor(
                    arity_source,
                    "enumerate(nums, 1, ",
                    "2",
                ))
    }));

    let unpacked_source =
        "def main():\n    nums: list[int] = [1, 2]\n    kwargs: dict[str, int] = {\"start\": 1}\n    _items = enumerate(nums, **kwargs)\n";
    let unpacked_result = lower_source(unpacked_source);
    assert!(unpacked_result.is_err());
    let unpacked_errors = unpacked_result.unwrap_err();
    assert!(unpacked_errors.iter().any(|error| {
        error.message == "enumerate() does not support unpacked keyword arguments"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range
                == Some(range_for_after_anchor(
                    unpacked_source,
                    "enumerate(nums, ",
                    "**kwargs",
                ))
    }));
}

#[test]
fn test_reversible_annotation_accepts_list_and_rejects_set() {
    let ok = lower_source(
        "def consume(xs: Reversible[int]) -> int:\n    rev: Iterator[int] = reversed(xs)\n    first: int | None = next(rev)\n    if first is None:\n        return 0\n    return first\n\ndef main():\n    nums: list[int] = [1, 2, 3]\n    consume(nums)\n",
    );
    assert!(ok.is_ok(), "{ok:?}");

    let err = lower_source(
        "def consume(xs: Reversible[int]) -> int:\n    rev: Iterator[int] = reversed(xs)\n    first: int | None = next(rev)\n    if first is None:\n        return 0\n    return first\n\ndef main():\n    nums: set[int] = {1, 2, 3}\n    consume(nums)\n",
    );
    assert!(err.is_err());
    let errors = err.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("expected 'Reversible[int]', got 'set[int]'")));
}

#[test]
fn test_comprehensions_accept_iterator_inputs() {
    let result = lower_source(
        "def main():\n    nums: list[int] = [1, 2, 3]\n    it_list: Iterator[int] = iter(nums)\n    list_comp: list[int] = [x for x in it_list]\n    it_set: Iterator[int] = iter(nums)\n    set_comp: set[int] = {x for x in it_set}\n    it_dict: Iterator[tuple[int, int]] = enumerate(nums)\n    dict_comp: dict[int, int] = {i: x for i, x in it_dict}\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_list_comprehension_invalid_target_has_flow_code() {
    let source = "def main():\n    values: list[int] = [1]\n    out: list[int] = [x for values[0] in values]\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "comprehension target must be a simple name or tuple"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ASSIGNMENT_TARGET)
            && error.primary_range == Some(range_for_after(source, "for ", "values[0]"))
    }));
}

#[test]
fn test_list_comprehension_non_iterable_has_flow_code() {
    let source = "def main():\n    value: int = 1\n    out: list[int] = [x for x in value]\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "cannot iterate over type 'int'"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ITERATION)
            && error.primary_range == Some(range_for_after(source, "in ", "value"))
    }));
}

#[test]
fn test_set_comprehension_invalid_target_has_flow_code() {
    let source = "def main():\n    values: list[int] = [1]\n    out: set[int] = {x for values[0] in values}\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "set comprehension target must be a simple name"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ASSIGNMENT_TARGET)
            && error.primary_range == Some(range_for_after(source, "for ", "values[0]"))
    }));
}

#[test]
fn test_set_comprehension_non_iterable_has_flow_code() {
    let source = "def main():\n    value: int = 1\n    out: set[int] = {x for x in value}\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "cannot iterate over type 'int'"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ITERATION)
            && error.primary_range == Some(range_for_after(source, "in ", "value"))
    }));
}

#[test]
fn test_dict_comprehension_invalid_tuple_target_has_flow_code() {
    let source = "def main():\n    values: list[int] = [0]\n    pairs: list[tuple[int, int]] = [(1, 2)]\n    out: dict[int, int] = {left: right for (left, values[0]) in pairs}\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "dict comprehension tuple target must contain only simple names"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ASSIGNMENT_TARGET)
            && error.primary_range == Some(range_for(source, "(left, values[0])"))
    }));
}

#[test]
fn test_dict_comprehension_non_iterable_has_flow_code() {
    let source =
        "def main():\n    value: int = 1\n    out: dict[int, int] = {x: x for x in value}\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "cannot iterate over type 'int'"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ITERATION)
            && error.primary_range == Some(range_for_after(source, "in ", "value"))
    }));
}

#[test]
fn test_generator_expression_multi_generator_has_type_code() {
    let source = "def main():\n    xs: list[int] = [1]\n    ys: list[int] = [2]\n    out: Iterator[int] = (x for x in xs for y in ys)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "only single-generator generator expressions are supported"
            && error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM)
            && error.primary_range == Some(range_for(source, "(x for x in xs for y in ys)"))
    }));
}

#[test]
fn test_generator_expression_invalid_target_has_flow_code() {
    let source = "def main():\n    values: list[int] = [1]\n    out: Iterator[int] = (x for values[0] in values)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "generator target must be a simple name"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ASSIGNMENT_TARGET)
            && error.primary_range == Some(range_for_after(source, "for ", "values[0]"))
    }));
}

#[test]
fn test_generator_expression_non_iterable_has_flow_code() {
    let source = "def main():\n    value: int = 1\n    out: Iterator[int] = (x for x in value)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "cannot iterate over type 'int'"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ITERATION)
            && error.primary_range == Some(range_for_after(source, "in ", "value"))
    }));
}

#[test]
fn test_walrus_invalid_target_has_flow_code() {
    let target_range = TextRange::new(TextSize::new(10), TextSize::new(14));
    let value_range = TextRange::new(TextSize::new(18), TextSize::new(22));
    let named = ExprNamed {
        node_index: AtomicNodeIndex::NONE,
        range: TextRange::new(TextSize::new(10), TextSize::new(22)),
        target: Box::new(Expr::NoneLiteral(ExprNoneLiteral {
            node_index: AtomicNodeIndex::NONE,
            range: target_range,
        })),
        value: Box::new(Expr::NoneLiteral(ExprNoneLiteral {
            node_index: AtomicNodeIndex::NONE,
            range: value_range,
        })),
    };
    let mut ctx = LowerCtx::new();

    let result = lower_named_expr(&named, &mut ctx);

    assert!(result.is_none());
    assert!(ctx.errors.iter().any(|error| {
        error.message == "walrus operator target must be a simple name"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ASSIGNMENT_TARGET)
            && error.primary_range == Some(target_range)
    }));
}

#[test]
fn test_map_is_typed_as_iterator() {
    let module = lower_source(
        "def add(x: int, y: int) -> int:\n    return x + y\n\ndef main():\n    left: list[int] = [1, 2]\n    right: list[int] = [3, 4]\n    mapped: Iterator[int] = map(add, left, right)\n    _vals: list[int] = list(mapped)\n",
    )
    .unwrap();
    let main_fn = module
        .functions
        .iter()
        .find(|func| func.name == "main")
        .expect("main function should exist");
    let Some(HirStmt::Let { ty, .. }) = main_fn
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::Let { name, .. } if name == "mapped"))
    else {
        panic!("expected let binding for mapped");
    };
    assert!(matches!(ty, Type::Iterator(_)));
}

#[test]
fn test_map_rejects_plain_list_annotation_without_materialization() {
    let result = lower_source(
        "def add(x: int, y: int) -> int:\n    return x + y\n\ndef main():\n    values: list[int] = map(add, [1, 2], [3, 4])\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("expected 'list[int]', got 'Iterator[int]'")));
}

#[test]
fn test_map_rejects_keywords_with_stable_diagnostic() {
    let source = "def add(x: int) -> int:\n    return x + 1\n\ndef main():\n    nums: list[int] = [1, 2]\n    _mapped = map(function=add, iterable=nums)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "map() does not accept keyword arguments"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range == Some(range_for_after_anchor(source, "map(", "function=add"))
    }));
}

#[test]
fn test_map_argument_errors_have_codes() {
    let missing_source =
        "def inc(x: int) -> int:\n    return x + 1\n\ndef main():\n    _mapped = map(inc)\n";
    let missing_result = lower_source(missing_source);
    assert!(missing_result.is_err());
    let missing_errors = missing_result.unwrap_err();
    assert!(missing_errors.iter().any(|error| {
        error.message == "map() takes a callable followed by at least one iterable"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(missing_source, "map(", "inc"))
    }));

    let iterable_source =
        "def inc(x: int) -> int:\n    return x + 1\n\ndef main():\n    _mapped = map(inc, 1)\n";
    let iterable_result = lower_source(iterable_source);
    assert!(iterable_result.is_err());
    let iterable_errors = iterable_result.unwrap_err();
    assert!(iterable_errors.iter().any(|error| {
        error.message
            == "map() iterable arguments must have statically-known element types, got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range
                == Some(range_for_after_anchor(iterable_source, "map(inc, ", "1"))
    }));

    let callable_source = "def main():\n    nums: list[int] = [1, 2]\n    _mapped = map(1, nums)\n";
    let callable_result = lower_source(callable_source);
    assert!(callable_result.is_err());
    let callable_errors = callable_result.unwrap_err();
    assert!(callable_errors.iter().any(|error| {
        error.message == "map() first argument must be callable"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range == Some(range_for_after_anchor(callable_source, "map(", "1"))
    }));
}

#[test]
fn test_filter_is_typed_as_iterator() {
    let module = lower_source(
        "def pred(x: int) -> bool:\n    return x % 2 == 0\n\ndef main():\n    nums: list[int] = [1, 2, 3, 4]\n    filtered: Iterator[int] = filter(pred, nums)\n    _vals: list[int] = list(filtered)\n",
    )
    .unwrap();
    let main_fn = module
        .functions
        .iter()
        .find(|func| func.name == "main")
        .expect("main function should exist");
    let Some(HirStmt::Let { ty, .. }) = main_fn
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::Let { name, .. } if name == "filtered"))
    else {
        panic!("expected let binding for filtered");
    };
    assert!(matches!(ty, Type::Iterator(_)));
}

#[test]
fn test_filter_rejects_plain_list_annotation_without_materialization() {
    let result = lower_source(
        "def pred(x: int) -> bool:\n    return x % 2 == 0\n\ndef main():\n    values: list[int] = filter(pred, [1, 2, 3, 4])\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("expected 'list[int]', got 'Iterator[int]'")));
}

#[test]
fn test_filter_rejects_keywords_with_stable_diagnostic() {
    let source = "def pred(x: int) -> bool:\n    return x > 0\n\ndef main():\n    nums: list[int] = [1, 2]\n    _filtered = filter(function=pred, iterable=nums)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "filter() does not accept keyword arguments"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range
                == Some(range_for_after_anchor(source, "filter(", "function=pred"))
    }));
}

#[test]
fn test_filter_argument_errors_have_codes() {
    let arity_source = "def pred(x: int) -> bool:\n    return x > 0\n\ndef main():\n    _filtered = filter(pred)\n";
    let arity_result = lower_source(arity_source);
    assert!(arity_result.is_err());
    let arity_errors = arity_result.unwrap_err();
    assert!(arity_errors.iter().any(|error| {
        error.message == "filter() takes exactly 2 arguments (function, iterable)"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(arity_source, "filter(", "pred"))
    }));

    let iterable_source =
        "def pred(x: int) -> bool:\n    return x > 0\n\ndef main():\n    _filtered = filter(pred, 1)\n";
    let iterable_result = lower_source(iterable_source);
    assert!(iterable_result.is_err());
    let iterable_errors = iterable_result.unwrap_err();
    assert!(iterable_errors.iter().any(|error| {
        error.message
            == "filter() argument must be an iterable with a statically-known element type, got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range
                == Some(range_for_after_anchor(iterable_source, "filter(pred, ", "1"))
    }));

    let callable_source =
        "def main():\n    nums: list[int] = [1, 2]\n    _filtered = filter(1, nums)\n";
    let callable_result = lower_source(callable_source);
    assert!(callable_result.is_err());
    let callable_errors = callable_result.unwrap_err();
    assert!(callable_errors.iter().any(|error| {
        error.message == "filter() first argument must be callable"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range == Some(range_for_after_anchor(callable_source, "filter(", "1"))
    }));

    let return_source =
        "def ident(x: int) -> int:\n    return x\n\ndef main():\n    nums: list[int] = [1, 2]\n    _filtered = filter(ident, nums)\n";
    let return_result = lower_source(return_source);
    assert!(return_result.is_err());
    let return_errors = return_result.unwrap_err();
    assert!(return_errors.iter().any(|error| {
        error.message == "filter() callable must return 'bool', got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range
                == Some(range_for_after_anchor(return_source, "filter(", "ident"))
    }));
}

#[test]
fn test_sum_min_max_accept_iterator_inputs() {
    let result = lower_source(
        "def main():\n    nums: list[int] = [3, 1, 2]\n    total: int = sum(iter(nums))\n    lo: int | None = min(iter(nums))\n    hi: int | None = max(iter(nums))\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_min_max_accept_variadic_scalar_inputs() {
    let result = lower_source(
        "def main() -> int:\n    lo: int = min(3, 1, 2)\n    hi: int = max(1, 5, 2, 4)\n    return lo + hi\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_min_max_missing_args_have_call_code() {
    let cases = [
        ("min", "min() takes at least 1 argument"),
        ("max", "max() takes at least 1 argument"),
    ];

    for (callable, message) in cases {
        let source = format!("def main():\n    _value = {callable}()\n");
        let result = lower_source(&source);
        assert!(
            result.is_err(),
            "{callable} should reject missing arguments"
        );
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|error| {
                error.message == message
                    && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
                    && error.primary_range
                        == Some(range_for_after_anchor(&source, "_value = ", callable))
            }),
            "{callable} errors: {errors:?}"
        );
    }
}

#[test]
fn test_min_max_keywords_have_call_code() {
    for callable in ["min", "max"] {
        let source = format!(
            "def main():\n    values: list[int] = [1, 2]\n    _value = {callable}(values=values)\n"
        );
        let result = lower_source(&source);
        assert!(result.is_err(), "{callable} should reject keywords");
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|error| {
                error.message == format!("{callable}() does not accept keyword arguments")
                    && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
                    && error.primary_range
                        == Some(range_for_after_anchor(
                            &source,
                            &format!("{callable}("),
                            "values=values",
                        ))
            }),
            "{callable} errors: {errors:?}"
        );
    }
}

#[test]
fn test_min_max_single_non_iterable_has_type_code() {
    let cases = [
        (
            "min",
            "min() argument must be an iterable with a statically-known element type, got 'int'",
        ),
        (
            "max",
            "max() argument must be an iterable with a statically-known element type, got 'int'",
        ),
    ];

    for (callable, message) in cases {
        let source = format!("def main():\n    _value = {callable}(1)\n");
        let result = lower_source(&source);
        assert!(
            result.is_err(),
            "{callable} should reject single non-iterable argument"
        );
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|error| {
                error.message == message
                    && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                    && error.primary_range
                        == Some(range_for_after_anchor(
                            &source,
                            &format!("{callable}("),
                            "1",
                        ))
            }),
            "{callable} errors: {errors:?}"
        );
    }
}

#[test]
fn test_max_two_arg_rejects_optional_operand() {
    let source =
        "def pick(d: dict[str, int], k: str) -> int:\n    best = 0\n    best = max(best, d[k])\n    return best\n";
    let result = lower_source(source);
    assert!(result.is_err(), "max(i64, i64|None) should be rejected");
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error
                    .message
                    .contains("max() with 2 arguments does not accept optional operands")
                && error.primary_range == Some(range_for(source, "d[k]"))),
        "max optional operand diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
fn test_min_max_incompatible_operands_have_type_codes() {
    let source = "def main() -> None:\n    lo = min(1, \"x\")\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected min incompatible operand error");

    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message
                    == "min() arguments must be comparable and type-compatible; got 'int' and 'str'"
                && error.primary_range == Some(range_for(source, "\"x\""))),
        "min incompatible operand diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
fn test_sorted_accepts_iterable_keyword_and_key_none() {
    let result = lower_source(
        "def main():\n    nums: list[int] = [3, 1, 2]\n    ordered: list[int] = sorted(iterable=nums, key=None, reverse=True)\n    assert ordered == [3, 2, 1]\n",
    );
    assert!(result.is_ok());
}

#[test]
fn test_sum_keyword_and_type_errors_have_codes() {
    let keyword_source =
        "def main():\n    nums: list[int] = [1, 2]\n    _total = sum(values=nums)\n";
    let keyword_result = lower_source(keyword_source);
    assert!(keyword_result.is_err());
    let keyword_errors = keyword_result.unwrap_err();
    assert!(keyword_errors.iter().any(|error| {
        error.message == "sum() does not accept keyword arguments"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range
                == Some(range_for_after_anchor(
                    keyword_source,
                    "sum(",
                    "values=nums",
                ))
    }));

    let type_source = "def main():\n    _total = sum(1)\n";
    let type_result = lower_source(type_source);
    assert!(type_result.is_err());
    let type_errors = type_result.unwrap_err();
    assert!(type_errors.iter().any(|error| {
        error.message
            == "sum() argument must be an iterable with a statically-known element type, got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(type_source, "sum(", "1"))
    }));
}

#[test]
fn test_sorted_positional_and_duplicate_errors_have_codes() {
    let too_many_source =
        "def main():\n    nums: list[int] = [3, 1, 2]\n    ordered = sorted(nums, nums)\n";
    let too_many_result = lower_source(too_many_source);
    assert!(too_many_result.is_err());
    let too_many_errors = too_many_result.unwrap_err();
    assert!(too_many_errors.iter().any(|error| {
        error.message == "sorted() takes at most 1 positional argument"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range
                == Some(range_for_after_anchor(
                    too_many_source,
                    "sorted(nums, ",
                    "nums",
                ))
    }));
}

#[test]
fn test_sorted_rejects_duplicate_iterable_argument() {
    let source =
        "def main():\n    nums: list[int] = [3, 1, 2]\n    ordered: list[int] = sorted(nums, iterable=nums)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "sorted() got multiple values for argument 'iterable'"
            && error.code == Some(DiagnosticCode::CALL_DUPLICATE_ARGUMENT)
            && error.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "sorted(nums, ",
                    "iterable=nums",
                ))
    }));
}

#[test]
fn test_sorted_type_and_key_errors_have_codes() {
    let iterable_source = "def main():\n    ordered = sorted(1)\n";
    let iterable_result = lower_source(iterable_source);
    assert!(iterable_result.is_err());
    let iterable_errors = iterable_result.unwrap_err();
    assert!(iterable_errors.iter().any(|error| {
        error.message
            == "sorted() argument must be an iterable with a statically-known element type, got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(iterable_source, "sorted(", "1"))
    }));

    let key_source =
        "def main():\n    nums: list[int] = [3, 1, 2]\n    ordered = sorted(nums, key=1)\n";
    let key_result = lower_source(key_source);
    assert!(key_result.is_err());
    let key_errors = key_result.unwrap_err();
    assert!(key_errors.iter().any(|error| {
        error.message == "sorted() keyword argument 'key' must be callable"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range == Some(range_for_after_anchor(key_source, "key=", "1"))
    }));

    let reverse_source =
        "def main():\n    nums: list[int] = [3, 1, 2]\n    ordered = sorted(nums, reverse=1)\n";
    let reverse_result = lower_source(reverse_source);
    assert!(reverse_result.is_err());
    let reverse_errors = reverse_result.unwrap_err();
    assert!(reverse_errors.iter().any(|error| {
        error.message == "sorted() keyword argument 'reverse' must be 'bool', got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(reverse_source, "reverse=", "1"))
    }));
}

#[test]
fn test_list_sort_accepts_reverse_keyword() {
    let result =
        lower_source("def main():\n    nums: list[int] = [3, 1, 2]\n    nums.sort(reverse=True)\n");
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_list_sort_rejects_non_bool_reverse_keyword() {
    let source = "def main():\n    nums: list[int] = [3, 1, 2]\n    nums.sort(reverse=1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("list.sort() argument 'reverse' must be 'bool'")
            && e.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && e.primary_range == Some(range_for_after_anchor(source, "reverse=", "1"))
    }));
}

#[test]
fn test_tuple_constructor_rejects_dynamic_list_shape() {
    let source = "def main():\n    nums: list[int] = [1, 2, 3]\n    t = tuple(nums)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("tuple() currently requires a tuple, list literal, or string literal")
            && e.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && e.primary_range == Some(range_for_after_anchor(source, "tuple(", "nums"))
    }));
}

#[test]
fn test_list_pop_index_and_tuple_index_optional_forms_lower() {
    let result = lower_source(
        "def main():\n    xs: list[int] = [1, 2, 3, 2]\n    popped: int | None = xs.pop(0)\n    idx: int | None = xs.index(2, start=0, stop=3)\n    pair: tuple[int, int, int] = (4, 5, 4)\n    tidx: int | None = pair.index(4, start=1)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_index_stop_only_keyword_forms_lower() {
    let result = lower_source(
        "def main():\n    xs: list[int] = [1, 2, 3, 2]\n    list_idx: int | None = xs.index(2, stop=3)\n    pair: tuple[int, int, int] = (4, 5, 4)\n    tuple_idx: int | None = pair.index(4, stop=2)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_index_optional_keyword_duplicate_forms_are_rejected() {
    let list_result =
        lower_source("def main():\n    xs: list[int] = [1, 2, 3]\n    xs.index(2, 0, start=1)\n");
    assert!(list_result.is_err());
    let list_errors = list_result.unwrap_err();
    assert!(list_errors.iter().any(|e| e
        .message
        .contains("index() got multiple values for argument 'start'")));

    let tuple_result = lower_source(
        "def main():\n    pair: tuple[int, int, int] = (1, 2, 3)\n    pair.index(2, 0, 2, stop=3)\n",
    );
    assert!(tuple_result.is_err());
    let tuple_errors = tuple_result.unwrap_err();
    assert!(tuple_errors.iter().any(|e| e
        .message
        .contains("index() got multiple values for argument 'stop'")));
}

#[test]
fn test_dict_update_kwargs_and_pop_default_lower() {
    let result = lower_source(
        "def main():\n    data: dict[str, int] = {\"x\": 1}\n    data.update(a=2)\n    other: dict[str, int] = {\"b\": 3}\n    data.update(other, c=4)\n    fallback: int = data.pop(\"missing\", default=9)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_string_split_and_replace_keyword_forms_lower() {
    let result = lower_source(
        "def main():\n    parts: list[str] = \"a,b,c\".split(sep=\",\", maxsplit=1)\n    replaced: str = \"aaaa\".replace(\"a\", \"b\", count=2)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_unexpected_method_keyword_is_rejected() {
    let source = "def main():\n    xs: list[int] = [1]\n    xs.append(value=2)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "append() got an unexpected keyword argument 'value'"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range == Some(range_for(source, "value"))
    }));
}

#[test]
fn test_unpacked_method_keyword_has_call_code() {
    let source = "def main():\n    xs: list[int] = [1]\n    xs.append(**{\"value\": 2})\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "append() does not support unpacked keyword arguments"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range == Some(range_for(source, "**{\"value\": 2}"))
    }));
}

#[test]
fn test_list_extend_non_iterable_has_protocol_code() {
    let source = "def main():\n    xs: list[int] = []\n    xs.extend(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "list.extend() argument must be an iterable with a statically-known element type, got 'int'"
            && error.code == Some(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE)
            && error.primary_range == Some(range_for(source, "1"))
    }));
}

#[test]
fn test_list_method_wrong_positional_count_has_call_code() {
    let source = "def main():\n    xs: list[int] = []\n    xs.append(1, 2)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "list.append() takes exactly 1 argument, got 2"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(source, "xs.append(1, ", "2"))
    }));
}

#[test]
fn test_list_method_type_mismatch_has_type_code() {
    let source = "def main():\n    xs: list[int] = [1]\n    xs.pop(\"0\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "list.pop() index must be 'int', got 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for(source, "\"0\""))
    }));
}

#[test]
fn test_list_missing_method_has_stdlib_code() {
    let source = "def main():\n    xs: list[int] = []\n    xs.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "list has no method 'missing'"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for_after(source, "xs.", "missing"))
    }));
}

#[test]
fn test_dict_update_keyword_value_mismatch_has_type_code() {
    let source = "def main():\n    data: dict[str, int] = {}\n    data.update(bad=\"x\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "dict.update() value type 'str' is not compatible with dict value type 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for(source, "bad=\"x\""))
    }));
}

#[test]
fn test_dict_method_wrong_positional_count_has_call_code() {
    let source = "def main():\n    data: dict[str, int] = {}\n    data.clear(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "dict.clear() takes no arguments"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for(source, "1"))
    }));
}

#[test]
fn test_dict_method_type_mismatch_has_type_code() {
    let source = "def main():\n    data: dict[str, int] = {\"x\": 1}\n    data.get(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "dict.get() key type 'int' is not compatible with dict key type 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after(source, "data.get(", "1"))
    }));
}

#[test]
fn test_dict_get_default_keyword_type_mismatch_has_type_code_and_range() {
    let source =
        "def main():\n    data: dict[int, int] = {0: 1}\n    value = data.get(0, default=\"bad\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "dict.get() default type 'str' is not compatible with dict value type 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(source, "default=", "\"bad\""))
    }));
}

#[test]
fn test_dict_pop_default_keyword_type_mismatch_has_type_code_and_range() {
    let source =
        "def main():\n    data: dict[int, int] = {0: 1}\n    value = data.pop(0, default=\"bad\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "dict.pop() default type 'str' is not compatible with dict value type 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(source, "default=", "\"bad\""))
    }));
}

#[test]
fn test_dict_setdefault_keyword_type_mismatch_has_type_code_and_range() {
    let source = "def main():\n    data: dict[int, int] = {0: 1}\n    value = data.setdefault(0, default=\"bad\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "dict.setdefault() default type 'str' is not compatible with dict value type 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(source, "default=", "\"bad\""))
    }));
}

#[test]
fn test_dict_missing_method_has_stdlib_code() {
    let source = "def main():\n    data: dict[str, int] = {}\n    data.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "dict has no method 'missing'"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for_after(source, "data.", "missing"))
    }));
}

#[test]
fn test_set_method_wrong_positional_count_has_call_code() {
    let source = "def main():\n    values: set[int] = {1}\n    values.add(1, 2)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "set.add() takes exactly 1 argument, got 2"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(source, "values.add(1, ", "2"))
    }));
}

#[test]
fn test_set_missing_method_has_stdlib_code() {
    let source = "def main():\n    values: set[int] = {1}\n    values.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "set has no method 'missing'"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for_after(source, "values.", "missing"))
    }));
}

#[test]
fn test_str_method_wrong_positional_count_has_call_code() {
    let source = "def main():\n    text: str = \"abc\"\n    text.find(\"a\", 1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "str.find() takes exactly 1 argument, got 2"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(source, "text.find(\"a\", ", "1"))
    }));
}

#[test]
fn test_str_method_type_mismatch_has_type_code() {
    let source = "def main():\n    text: str = \"a,b\"\n    text.split(\",\", \"bad\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "str.split() maxsplit must be 'int', got 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for(source, "\"bad\""))
    }));
}

#[test]
fn test_str_replace_keyword_count_type_mismatch_has_type_code() {
    let source =
        "def main():\n    text: str = \"aaaa\"\n    text.replace(\"a\", \"b\", count=\"bad\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "str.replace() count must be 'int', got 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for(source, "\"bad\""))
    }));
}

#[test]
fn test_str_missing_method_has_stdlib_code() {
    let source = "def main():\n    text: str = \"abc\"\n    text.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "str has no method 'missing'"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for_after(source, "text.", "missing"))
    }));
}

#[test]
fn test_tuple_method_wrong_positional_count_has_call_code() {
    let source = "def main():\n    pair: tuple[int, int] = (1, 2)\n    pair.count(1, 2)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "tuple.count() takes exactly 1 argument, got 2"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(source, "pair.count(1, ", "2"))
    }));
}

#[test]
fn test_tuple_method_type_mismatch_has_type_code() {
    let source =
        "def main():\n    pair: tuple[int, int, int] = (1, 2, 3)\n    pair.index(1, \"bad\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "tuple.index() bounds must be 'int', got 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for(source, "\"bad\""))
    }));
}

#[test]
fn test_tuple_missing_method_has_stdlib_code() {
    let source = "def main():\n    pair: tuple[int, int] = (1, 2)\n    pair.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "tuple has no method 'missing'"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for_after(source, "pair.", "missing"))
    }));
}

#[test]
fn test_class_method_argument_type_has_type_code() {
    let source = "class Box:\n    def take(self, value: int) -> None:\n        pass\n\ndef main():\n    box: Box = Box()\n    box.take(\"bad\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "argument 1 ('value') of Box.take(): expected 'int', got 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for(source, "\"bad\""))
    }));
}

#[test]
fn test_callable_field_wrong_arity_has_call_code() {
    let source = "class Runner:\n    callback: Callable[[int], int]\n\n    def __init__(self, callback: Callable[[int], int]):\n        self.callback = callback\n\ndef double(x: int) -> int:\n    return x * 2\n\ndef main():\n    runner: Runner = Runner(double)\n    runner.callback()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "Runner.callback() (callable field) takes 1 argument(s), got 0"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range == Some(range_for_after(source, "runner.", "callback"))
    }));
}

#[test]
fn test_class_field_not_callable_has_call_code() {
    let source = "class Box:\n    value: int\n\n    def __init__(self, value: int):\n        self.value = value\n\ndef main():\n    box: Box = Box(1)\n    box.value()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "field 'value' of class 'Box' is not callable (type: 'int')"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range == Some(range_for_after(source, "box.", "value"))
    }));
}

#[test]
fn test_class_missing_method_has_class_code() {
    let source = "class Box:\n    value: int\n\n    def __init__(self, value: int):\n        self.value = value\n\ndef main():\n    box: Box = Box(1)\n    box.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "class 'Box' has no method 'missing'"
            && error.code == Some(DiagnosticCode::CLASS_MISSING_MEMBER)
            && error.primary_range == Some(range_for_after(source, "box.", "missing"))
    }));
}

#[test]
fn test_protocol_method_wrong_arity_has_call_code() {
    let protocol_ty = Type::Protocol {
        name: "Runner".to_string(),
        methods: vec![(
            "run".to_string(),
            FunctionType::new(vec![("value".to_string(), Type::Int)], Type::Str),
        )],
    };
    let mut ctx = LowerCtx::new();
    let method_range = TextRange::new(TextSize::new(10), TextSize::new(13));

    let result = resolve_method_type(&protocol_ty, "run", &[], &[], method_range, &mut ctx);

    assert!(result.is_none());
    assert!(ctx.errors.iter().any(|error| {
        error.message == "Runner.run() takes 1 argument(s), got 0"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(method_range)
    }));
}

#[test]
fn test_protocol_missing_method_has_protocol_code() {
    let protocol_ty = Type::Protocol {
        name: "Runner".to_string(),
        methods: vec![(
            "run".to_string(),
            FunctionType::new(vec![("value".to_string(), Type::Int)], Type::Str),
        )],
    };
    let mut ctx = LowerCtx::new();
    let method_range = TextRange::new(TextSize::new(20), TextSize::new(27));

    let result = resolve_method_type(&protocol_ty, "missing", &[], &[], method_range, &mut ctx);

    assert!(result.is_none());
    assert!(ctx.errors.iter().any(|error| {
        error.message == "protocol 'Runner' has no method 'missing'"
            && error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
            && error.primary_range == Some(method_range)
    }));
}

#[test]
fn test_newtype_value_wrong_arity_has_call_code() {
    let source = "class Port(int):\n    pass\n\ndef main():\n    port: Port = Port(8080)\n    port.value(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "Port.value() takes no arguments"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after(source, "port.value(", "1"))
    }));
}

#[test]
fn test_enum_value_wrong_arity_has_call_code() {
    let source = "from enum import Enum\n\nclass Status(Enum):\n    OK = 200\n\ndef main():\n    status: Status = Status.OK\n    status.value(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "Status.value() takes no arguments"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after(source, "status.value(", "1"))
    }));
}

#[test]
fn test_enum_missing_method_has_class_code() {
    let source = "from enum import Enum\n\nclass Status(Enum):\n    OK = 200\n\ndef main():\n    status: Status = Status.OK\n    status.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "enum 'Status' has no method 'missing'"
            && error.code == Some(DiagnosticCode::CLASS_MISSING_MEMBER)
            && error.primary_range == Some(range_for_after(source, "status.", "missing"))
    }));
}

#[test]
fn test_bigint_clone_wrong_arity_has_call_code() {
    let source = "def main():\n    value: bigint = bigint(1)\n    value.clone(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "bigint.clone() takes no arguments"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after(source, "value.clone(", "1"))
    }));
}

#[test]
fn test_bigint_missing_method_has_stdlib_code() {
    let source = "def main():\n    value: bigint = bigint(1)\n    value.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "type 'bigint' has no method 'missing'"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for_after(source, "value.", "missing"))
    }));
}

#[test]
fn test_generic_type_missing_method_has_stdlib_code() {
    let source = "def use_value[T](value: T):\n    value.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "type 'T' has no method 'missing'"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for_after(source, "value.", "missing"))
    }));
}

#[test]
fn test_duplicate_optional_method_keyword_is_rejected() {
    let source = "def main():\n    data: dict[str, int] = {\"x\": 1}\n    value: int = data.get(\"x\", 1, default=2)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("get() got multiple values for argument 'default'")
            && e.code == Some(DiagnosticCode::CALL_DUPLICATE_ARGUMENT)
            && e.primary_range == Some(range_for(source, "default"))
    }));
}

#[test]
fn test_user_defined_method_defaults_and_keywords_lower() {
    let result = lower_source(
        "class CounterBox:\n    value: int\n\n    def __init__(self, value: int):\n        self.value = value\n\n    def bump(self, amount: int = 1) -> int:\n        return self.value + amount\n\ndef main():\n    box: CounterBox = CounterBox(4)\n    a: int = box.bump()\n    b: int = box.bump(amount=3)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_break_outside_loop() {
    let source = "def main():\n    break\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("'break' outside of loop")
            && e.code == Some(DiagnosticCode::FLOW_BREAK_OUTSIDE_LOOP)
            && e.primary_range == Some(range_for(source, "break"))));
}

#[test]
fn test_continue_outside_loop() {
    let source = "def main():\n    continue\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("'continue' outside of loop")
            && e.code == Some(DiagnosticCode::FLOW_CONTINUE_OUTSIDE_LOOP)
            && e.primary_range == Some(range_for(source, "continue"))));
}

#[test]
fn test_break_inside_loop() {
    let module = lower_source("def main():\n    while True:\n        break\n").unwrap();
    assert_eq!(module.functions.len(), 1);
}

#[test]
fn test_nested_loops() {
    let module = lower_source(
        "def main():\n    for i in range(3):\n        for j in range(2):\n            print(i)\n",
    )
    .unwrap();
    assert_eq!(module.functions.len(), 1);
}

#[test]
fn test_fstring_basic() {
    let module = lower_source(
        "def main():\n    name: str = \"Alice\"\n    msg: str = f\"Hello, {name}!\"\n    print(msg)\n",
    )
    .unwrap();
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].body.len(), 3);
}

#[test]
fn test_fstring_with_expression() {
    let module = lower_source(
        "def main():\n    a: int = 2\n    b: int = 3\n    print(f\"{a} + {b} = {a + b}\")\n",
    )
    .unwrap();
    assert_eq!(module.functions.len(), 1);
}

#[test]
fn test_tuple_unpack() {
    let module = lower_source(
        "def main():\n    pair: tuple[int, str] = (1, \"hello\")\n    x, y = pair\n    print(x)\n",
    )
    .unwrap();
    assert_eq!(module.functions.len(), 1);
    assert!(module.functions[0].body.len() >= 3);
    assert!(matches!(
        module.functions[0].body[1],
        HirStmt::TupleUnpack { .. }
    ));
}

#[test]
fn test_tuple_unpack_wrong_count() {
    let source = "def main():\n    pair: tuple[int, str] = (1, \"hello\")\n    x, y, z = pair\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("expected 3 values, got 2")
            && e.code == Some(DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH)
            && e.primary_range == Some(range_for(source, "x, y, z"))));
}

#[test]
fn test_tuple_unpack_non_tuple() {
    let source = "def main():\n    x: int = 42\n    a, b = x\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("cannot unpack non-tuple")
            && e.code == Some(DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH)
            && e.primary_range == Some(range_for_after(source, "a, b = ", "x"))));
}

#[test]
fn test_tuple_unpack_invalid_target_has_unpack_code() {
    let source = "def main():\n    values: list[int] = [0]\n    values[0], y = (1, 2)\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected tuple unpack target error");
    assert!(errors.iter().any(|e| {
        e.message == "tuple unpacking target must be a simple name or attribute"
            && e.code == Some(DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH)
            && e.primary_range == Some(range_for(source, "values[0]"))
    }));
}

#[test]
fn test_tuple_unpack_reassignment_type_mismatch_has_primary_range() {
    let source =
        "def main():\n    left = 1\n    left, label = (\"not an int\", \"name\")\n    print(label)\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected tuple unpack reassignment type mismatch");
    assert!(errors.iter().any(
        |e| e.message.contains("cannot assign 'str' to variable 'left'")
            && e.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && e.primary_range == Some(range_for_after_anchor(source, "left = 1\n    ", "left"))
    ));
}

#[test]
fn test_star_unpack_multiple_starred_targets_have_unpack_code() {
    let source = "def main():\n    first, *rest, *tail = [1, 2, 3]\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected multiple starred target error");
    assert!(errors.iter().any(|e| {
        e.message == "multiple starred expressions in assignment"
            && e.code == Some(DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH)
            && e.primary_range == Some(range_for(source, "*tail"))
    }));
}

#[test]
fn test_star_unpack_invalid_starred_target_has_unpack_code() {
    let source = "def main():\n    values: list[int] = [0]\n    first, *values[0] = [1, 2]\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected invalid starred target error");
    assert!(errors.iter().any(|e| {
        e.message == "starred target must be a simple name"
            && e.code == Some(DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH)
            && e.primary_range == Some(range_for_after_anchor(source, "*", "values[0]"))
    }));
}

#[test]
fn test_star_unpack_invalid_trailing_target_has_unpack_code() {
    let source = "def main():\n    values: list[int] = [0]\n    first, *rest, values[0] = [1, 2]\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected invalid star unpack trailing target error");
    assert!(errors.iter().any(|e| {
        e.message == "star unpacking target must be a simple name"
            && e.code == Some(DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH)
            && e.primary_range == Some(range_for_after_anchor(source, "*rest, ", "values[0]"))
    }));
}

#[test]
fn test_star_unpack_requires_list_has_primary_range() {
    let source = "def main():\n    first, *rest = (1, 2, 3)\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected star unpack list-shape error");
    assert!(errors.iter().any(
        |e| e.code == Some(DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH)
            && e.primary_range == Some(range_for(source, "(1, 2, 3)"))
    ));
}

#[test]
fn test_tuple_unpack_allows_attribute_targets() {
    let module = lower_source(
        "class Pair:\n    x: int\n    y: int\n    def __init__(self):\n        self.x = 1\n        self.y = 2\n    def swap(self):\n        self.x, self.y = self.y, self.x\n",
    )
    .unwrap();
    let pair_class = module
        .classes
        .iter()
        .find(|class| class.name == "Pair")
        .expect("Pair class");
    let swap_method = pair_class
        .methods
        .iter()
        .find(|method| method.name == "swap")
        .expect("swap method");
    let HirStmt::TupleUnpack { targets, .. } = &swap_method.body[0] else {
        panic!("expected tuple unpack statement");
    };
    assert!(matches!(
        targets.as_slice(),
        [
            crate::hir_nodes::HirTupleTarget {
                binding: crate::hir_nodes::HirTupleTargetBinding::Field { object: left_obj, field: left_field },
                ..
            },
            crate::hir_nodes::HirTupleTarget {
                binding: crate::hir_nodes::HirTupleTargetBinding::Field { object: right_obj, field: right_field },
                ..
            }
        ] if left_obj == "self"
            && left_field == "x"
            && right_obj == "self"
            && right_field == "y"
    ));
}

#[test]
fn test_for_tuple_target_requires_tuple_elements() {
    let source =
        "def main():\n    nums: list[int] = [1, 2, 3]\n    for a, b in nums:\n        print(a)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("for loop tuple target expects iterable elements of tuple type")
            && e.code == Some(DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH)
            && e.primary_range == Some(range_for(source, "a, b"))
    }));
}

#[test]
fn test_for_tuple_target_arity_mismatch_has_primary_range() {
    let source = "def main():\n    pairs: list[tuple[int, int, int]] = [(1, 2, 3)]\n    for a, b in pairs:\n        print(a)\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected for tuple target arity mismatch");
    assert!(errors
        .iter()
        .any(|e| e.message.contains("expects 2 element(s)")
            && e.code == Some(DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH)
            && e.primary_range == Some(range_for(source, "a, b"))));
}

#[test]
fn test_generic_class_subscript_requires_declared_type_params() {
    let source =
        "T = TypeVar(\"T\")\nclass LegacyBox:\n    value: T\ndef f(x: LegacyBox[int]) -> int:\n    return 1\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("does not declare type parameters")
            && e.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)
            && e.primary_range == Some(range_for_after_anchor(source, "def f(x: ", "LegacyBox"))
    }));
}

#[test]
fn test_generic_class_subscript_arity_mismatch_errors() {
    let source =
        "class Pair[T]:\n    left: T\n    right: T\ndef f(x: Pair[int, str]) -> int:\n    return 1\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("expects 1 type argument(s), got 2")
            && e.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)
            && e.primary_range == Some(range_for(source, "int, str"))
    }));
}

#[test]
fn test_invalid_dict_type_annotation_has_primary_range() {
    let source = "def consume(value: dict[int]) -> int:\n    return 0\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message == "dict type annotation requires [K, V] syntax"
            && e.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)
            && e.primary_range == Some(range_for(source, "int"))
    }));
}

#[test]
fn test_callable_param_list_annotation_has_primary_range() {
    let source = "def consume(callback: Callable[int, str]) -> int:\n    return 0\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message == "Callable parameter types must be a list: Callable[[int, str], bool]"
            && e.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)
            && e.primary_range == Some(range_for_after_anchor(source, "Callable[", "int"))
    }));
}

#[test]
fn test_missing_function_parameter_annotation_has_primary_range() {
    let source = "def identity(value) -> int:\n    return value\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "parameter 'value' in function 'identity' is missing a type annotation"
            && error.code == Some(DiagnosticCode::TYPE_MISSING_ANNOTATION)
            && error.primary_range == Some(range_for(source, "value"))
    }));
}

#[test]
fn test_missing_class_method_parameter_annotation_has_primary_range() {
    let source = "class Tool:\n    def scale(self, value) -> int:\n        return value\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "parameter 'value' in Tool.scale is missing a type annotation"
            && error.code == Some(DiagnosticCode::TYPE_MISSING_ANNOTATION)
            && error.primary_range == Some(range_for(source, "value"))
    }));
}

#[test]
fn test_unsupported_function_default_argument_has_primary_range() {
    let source =
        "def seed() -> int:\n    return 7\n\ndef pick(x: int = seed()) -> int:\n    return x\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "function 'pick': unsupported default argument expression for parameter 'x'"
            && error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_DEFAULT_ARGUMENT)
            && error.primary_range == Some(range_for_after_anchor(source, "= ", "seed()"))
    }));
}

#[test]
fn test_unsupported_method_default_argument_has_primary_range() {
    let source = "def seed() -> int:\n    return 7\n\nclass Tool:\n    def scale(self, value: int = seed()) -> int:\n        return value\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "class 'Tool.scale': unsupported default argument expression for parameter 'value'"
            && error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_DEFAULT_ARGUMENT)
            && error.primary_range == Some(range_for_after_anchor(source, "= ", "seed()"))
    }));
}

#[test]
fn test_unknown_type_annotation_has_primary_range() {
    let source = "def consume(value: MissingType) -> int:\n    return 0\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "unknown type: 'MissingType'"
            && error.code == Some(DiagnosticCode::NAME_UNKNOWN_TYPE)
            && error.primary_range == Some(range_for(source, "MissingType"))
    }));
}

#[test]
fn test_unknown_generic_type_annotation_has_primary_range() {
    let source = "def main():\n    x: UnknownType[int] = 42\n    print(x)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "unknown type: 'UnknownType'"
            && error.code == Some(DiagnosticCode::NAME_UNKNOWN_TYPE)
            && error.primary_range == Some(range_for(source, "UnknownType"))
    }));
}

#[test]
fn test_typevar_constraints_violation_has_type_code() {
    let result = lower_source(
        "from typing import TypeVar\n\nT = TypeVar(\"T\", int, str)\n\ndef echo(x: T) -> T:\n    return x\n\ndef main():\n    bad: float = echo(1.5)\n    print(bad)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            == "type 'float' does not satisfy constraints (int, str) required by type parameter 'T'"
            && e.code == Some(DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED)
    }));
}

#[test]
fn test_typevar_invalid_bound_shape_has_primary_range() {
    let source = "from typing import TypeVar\n\nT = TypeVar(\"T\", bound=1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "TypeVar bound must be a simple type name"
            && error.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)
            && error.primary_range == Some(range_for_after_anchor(source, "bound=", "1"))
    }));
}

#[test]
fn test_typevar_bound_constraints_conflict_has_primary_range() {
    let source = "from typing import TypeVar\n\nT = TypeVar(\"T\", int, bound=str)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "TypeVar cannot declare both 'bound' and 'constraints'"
            && error.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)
            && error.primary_range == Some(range_for_after_anchor(source, "int, ", "bound"))
    }));
}

#[test]
fn test_pep695_typevar_constraint_shape_has_primary_range() {
    let source = "def echo[T: (int, 1)](x: T) -> T:\n    return x\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "TypeVar constraints must be simple type names"
            && error.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)
            && error.primary_range == Some(range_for_after_anchor(source, "(int, ", "1"))
    }));
}

#[test]
fn test_auto_init_inheritance_missing_super_has_class_code() {
    let source = "class Animal:\n    name: str\n\n    def __init__(self, name: str):\n        self.name = name\n\nclass Dog(Animal):\n    breed: str\n\ndef main():\n    d: Dog = Dog(\"Rex\", \"Labrador\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            == "class 'Dog' has fields but no __init__; parent fields will not be initialized. Define an explicit __init__ with super().__init__(...)"
            && e.code == Some(DiagnosticCode::CLASS_MISSING_INITIALIZER)
            && e.primary_range == Some(range_for_after(source, "class ", "Dog"))
    }));
}

#[test]
fn test_auto_init_required_after_default_has_class_code() {
    let source =
        "class BadConfig:\n    debug: bool = False\n    name: str\n\ndef main():\n    c: BadConfig = BadConfig(True, \"test\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            == "class 'BadConfig': required field 'name' declared after field with default value"
            && e.code == Some(DiagnosticCode::CLASS_REQUIRED_FIELD_AFTER_DEFAULT)
            && e.primary_range == Some(range_for_after(source, "    ", "name"))
    }));
}

#[test]
fn test_enum_duplicate_value_has_class_code() {
    let source = "from enum import Enum\n\nclass Status(Enum):\n    OK = 200\n    SUCCESS = 200\n    NOT_FOUND = 404\n\ndef main():\n    s: Status = Status.OK\n    print(s)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message == "enum 'Status' has duplicate value 200: variants 'OK' and 'SUCCESS'"
            && e.code == Some(DiagnosticCode::CLASS_DUPLICATE_OR_INVALID_VALUE)
            && e.primary_range == Some(range_for_after(source, "    ", "SUCCESS"))
    }));
}

#[test]
fn test_missing_field_has_class_code() {
    let source = "class Point:\n    x: float\n    y: float\n\n    def __init__(self, x: float, y: float):\n        self.x = x\n        self.y = y\n\ndef main():\n    p: Point = Point(1.0, 2.0)\n    print(p.z)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message == "type 'Point' has no field 'z'"
            && e.code == Some(DiagnosticCode::CLASS_MISSING_MEMBER)
            && e.primary_range == Some(range_for_after(source, "print(p.", "z"))
    }));
}

#[test]
fn test_enum_missing_attribute_has_class_code() {
    let source = "from enum import Enum\n\nclass Status(Enum):\n    OK = 200\n\ndef main():\n    s: Status = Status.OK\n    print(s.missing)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message == "enum 'Status' has no attribute 'missing'"
            && e.code == Some(DiagnosticCode::CLASS_MISSING_MEMBER)
            && e.primary_range == Some(range_for_after(source, "print(s.", "missing"))
    }));
}

#[test]
fn test_unsupported_attribute_expression_has_type_code() {
    let source = "def main():\n    value: int = 1\n    print(value.real)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            == "unsupported expression form: attribute access '.real' is not supported as an expression; use as a method call"
            && e.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM)
            && e.primary_range == Some(range_for_after(source, "print(", "value.real"))
    }));
}

#[test]
fn test_super_outside_parent_has_class_code() {
    let source = "def main():\n    super().missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message == "super() used outside of a class with a parent"
            && e.code == Some(DiagnosticCode::CLASS_INVALID_BASE)
            && e.primary_range == Some(range_for(source, "super()"))
    }));
}

#[test]
fn test_missing_class_static_method_has_class_code() {
    let source = "class Box:\n    value: int\n\n    def __init__(self, value: int):\n        self.value = value\n\ndef main():\n    Box.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message == "type 'Box' has no class/static method 'missing'"
            && e.code == Some(DiagnosticCode::CLASS_MISSING_MEMBER)
            && e.primary_range == Some(range_for_after(source, "Box.", "missing"))
    }));
}

#[test]
fn test_unknown_parent_class_has_class_code() {
    let source =
        "class Child(MissingParent):\n    value: int\n\ndef main():\n    c: Child = Child(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message == "invalid base class for 'Child': parent class 'MissingParent' not defined"
            && e.code == Some(DiagnosticCode::CLASS_INVALID_BASE)
            && e.primary_range == Some(range_for_after(source, "class Child(", "MissingParent"))
    }));
}

#[test]
fn test_unsupported_class_field_default_has_class_code() {
    let source = "class BadDefault:\n    value: int = 1 + 2\n\ndef main():\n    b: BadDefault = BadDefault(3)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message == "unsupported class declaration in 'BadDefault': unsupported default expression for field 'value'"
            && e.code == Some(DiagnosticCode::CLASS_UNSUPPORTED_DECLARATION)
            && e.primary_range == Some(range_for_after(source, "= ", "1 + 2"))
    }));
}

#[test]
fn test_match_tuple_pattern_requires_tuple_subject() {
    let result = lower_source(
        "def main():\n    x: int = 1\n    match x:\n        case (a, b):\n            print(a)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("tuple pattern requires subject of tuple type")));
}

#[test]
fn test_match_tuple_pattern_arity_mismatch_errors() {
    let result = lower_source(
        "def main():\n    x: tuple[int, int] = (1, 2)\n    match x:\n        case (a, b, c):\n            print(a)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("tuple pattern expects 3 element(s), subject has 2")));
}

#[test]
fn test_protocol_bound_forwarding_accepts_conforming_typevar() {
    let result = lower_source(
        "class Runner(Protocol):\n    def run(self) -> int:\n        pass\n\nclass Job:\n    def run(self) -> int:\n        return 1\n\ndef use_runner[T: Runner](x: T) -> T:\n    return x\n\ndef relay_runner[U: Runner](x: U) -> U:\n    return use_runner(x)\n\ndef main():\n    j: Job = relay_runner(Job())\n    print(j.run())\n",
    );
    assert!(result.is_ok());
}

#[test]
fn test_protocol_bound_forwarding_rejects_unknown_bound() {
    let result = lower_source(
        "def take_missing[T: MissingBound](x: T) -> T:\n    return x\n\ndef relay_missing[U: MissingBound](x: U) -> U:\n    return take_missing(x)\n\ndef main():\n    print(1)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("does not implement protocol 'MissingBound'")));
}

#[test]
fn test_protocol_bound_forwarding_rejects_non_conforming_typevar() {
    let result = lower_source(
        "class Readable(Protocol):\n    def read(self) -> str:\n        pass\n\nclass Closable(Protocol):\n    def close(self) -> None:\n        pass\n\ndef take_readable[T: Readable](x: T) -> T:\n    return x\n\ndef relay_bad[U: Closable](x: U) -> U:\n    return take_readable(x)\n\ndef main():\n    print(1)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("does not implement protocol 'Readable'")));
}

#[test]
fn test_comparable_bound_accepts_homogeneous_tuples() {
    let result = lower_source(
        "def choose[T: Comparable](x: T, y: T) -> T:\n    return x if x > y else y\n\ndef main():\n    left: tuple[int, int] = (1, 2)\n    right: tuple[int, int] = (2, 1)\n    out: tuple[int, int] = choose(left, right)\n    print(out)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_recursive_tree_attributes_narrow_after_truthiness_or_guard() {
    let result = lower_source(
        "class TreeNode:\n    val: int\n    left: TreeNode | None\n    right: TreeNode | None\n\n    def __init__(self, val: int, left: TreeNode | None, right: TreeNode | None):\n        self.val = val\n        self.left = left\n        self.right = right\n\ndef mirrored_sum(p: TreeNode | None, q: TreeNode | None) -> int:\n    if not p and not q:\n        return 0\n    if not p or not q:\n        return 0\n    left: TreeNode | None = p.left\n    right: TreeNode | None = q.right\n    return p.val + q.val + mirrored_sum(left, q.left) + mirrored_sum(p.right, right)\n",
    );
    assert!(
        result.is_ok(),
        "recursive tree attributes should lower after `if not p or not q` early-return narrowing"
    );
}

#[test]
fn test_empty_dict_literal_specializes_from_first_subscript_write_and_get_default() {
    let result = lower_source(
        "def main():\n    counts = {}\n    key: str = \"x\"\n    counts[key] = 1 + counts.get(key, 0)\n    value: int = counts.get(key, 0)\n    assert value == 1\n",
    );
    assert!(
        result.is_ok(),
        "empty dict literal should specialize to dict[str, int] from first write/get-default flow"
    );
}

#[test]
fn test_empty_dict_literal_conflicting_write_reports_deterministic_error() {
    let result =
        lower_source("def main():\n    data = {}\n    data[1] = 10\n    data[\"x\"] = 20\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.message.contains("empty literal type conflict")));
}

#[test]
fn test_empty_dict_specialization_with_split_zip_word_pattern_shape() {
    let result = lower_source(
        "def wordPattern(pattern: str, s: str) -> bool:\n    words = s.split(\" \")\n    if len(pattern) != len(words):\n        return False\n    charToWord = {}\n    wordToChar = {}\n    for c, w in zip(pattern, words):\n        if c in charToWord and charToWord[c] != w:\n            return False\n        if w in wordToChar and wordToChar[w] != c:\n            return False\n        charToWord[c] = w\n        wordToChar[w] = c\n    return True\n",
    );
    assert!(
        result.is_ok(),
        "word-pattern split/zip flow should specialize empty dicts to dict[str, str]: {:?}",
        result.err()
    );
}
