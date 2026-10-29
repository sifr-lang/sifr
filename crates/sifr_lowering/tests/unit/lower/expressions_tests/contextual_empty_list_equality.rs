use super::*;

fn compare_operands<'a>(module: &'a crate::HirModule, name: &str) -> (&'a HirExpr, &'a HirExpr) {
    let HirExpr::Compare {
        left, comparators, ..
    } = function_let_value(module, name)
    else {
        panic!("expected comparison binding");
    };
    (left, &comparators[0])
}

fn assert_first_nested_element_type(expr: &HirExpr, expected: &Type) {
    let HirExpr::ListLiteral { elements, .. } = expr else {
        panic!("expected nested list literal");
    };
    assert_eq!(elements[0].ty(), expected);
}

#[test]
fn equality_specializes_empty_list_literals_from_concrete_opposite_operands() {
    let source = "def main():\n    values: list[int] = [1]\n    nested: list[list[int]] = [[1], []]\n    right_empty: bool = values == []\n    left_empty: bool = [] != values\n    right_nested: bool = nested == [[], [1]]\n    left_nested_leading: bool = [[], [1]] != nested\n    left_nested_trailing: bool = [[1], []] == nested\n";
    let module = lower_source(source).expect("contextual empty list literals should lower");

    let (right_empty_left, right_empty_right) = compare_operands(&module, "right_empty");
    assert_eq!(right_empty_left.ty(), &Type::List(Box::new(Type::Int)));
    assert_eq!(right_empty_right.ty(), &Type::List(Box::new(Type::Int)));

    let (left_empty_left, left_empty_right) = compare_operands(&module, "left_empty");
    assert_eq!(left_empty_left.ty(), &Type::List(Box::new(Type::Int)));
    assert_eq!(left_empty_right.ty(), &Type::List(Box::new(Type::Int)));

    let nested_ty = Type::List(Box::new(Type::List(Box::new(Type::Int))));
    let nested_element_ty = Type::List(Box::new(Type::Int));
    let (right_nested_left, right_nested_right) = compare_operands(&module, "right_nested");
    assert_eq!(right_nested_left.ty(), &nested_ty);
    assert_eq!(right_nested_right.ty(), &nested_ty);
    assert_first_nested_element_type(right_nested_right, &nested_element_ty);

    let (left_nested_left, left_nested_right) = compare_operands(&module, "left_nested_leading");
    assert_eq!(left_nested_left.ty(), &nested_ty);
    assert_eq!(left_nested_right.ty(), &nested_ty);
    assert_first_nested_element_type(left_nested_left, &nested_element_ty);

    let (left_nested_left, left_nested_right) = compare_operands(&module, "left_nested_trailing");
    assert_eq!(left_nested_left.ty(), &nested_ty);
    assert_eq!(left_nested_right.ty(), &nested_ty);
    let HirExpr::ListLiteral { elements, .. } = left_nested_left else {
        panic!("expected nested list literal");
    };
    assert_eq!(elements[1].ty(), &nested_element_ty);
}

#[test]
fn equality_preserves_mismatched_literal_diagnostics() {
    let source = "def main():\n    values: list[int] = [1]\n    floats: list[list[float]] = [[1.0]]\n    result: bool = values == [\"x\"]\n    widened: bool = floats == [[], [1]]\n";
    let errors = lower_source(source).expect_err("mismatched concrete lists should be rejected");

    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message == "cannot compare 'list[int]' and 'list[str]' with =="
                && error.primary_range == Some(range_for(source, "[\"x\"]"))
        }),
        "{errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message.contains("cannot compare")
                && error.primary_range == Some(range_for(source, "[[], [1]]"))
        }),
        "{errors:?}"
    );
}

#[test]
fn equality_does_not_retype_variable_operands() {
    let source = "def main():\n    concrete: list[int] = [1]\n    unresolved = []\n    result: bool = concrete == unresolved\n";
    let errors = lower_source(source).expect_err("named list[Any] operands should not be retyped");

    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message
                    == "cannot compare values without structural equality 'list[int]' and 'list[Any]' with =="
                && error.primary_range
                    == Some(range_for_after_anchor(source, "concrete ==", "unresolved"))
        }),
        "{errors:?}"
    );
}
