use super::*;

#[test]
fn recursive_list_total_order_accepts_nested_orderable_elements() {
    let result = lower_source(
        "def nested_key(value: int) -> list[int]:\n    return [value % 10, value]\n\ndef main():\n    pairs: list[list[int]] = [[3, 4], [1, 2]]\n    pairs.sort()\n    cubes: list[list[list[str]]] = [[[\"b\"]], [[\"a\"]]]\n    ordered: list[list[list[str]]] = sorted(cubes)\n    keyed: list[int] = sorted([21, 12, 11], key=nested_key)\n",
    );

    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn recursive_list_total_order_rejects_nested_non_total_order_elements() {
    let sources = [
        "def main():\n    values: list[list[float]] = [[2.0], [1.0]]\n    values.sort()\n",
        "def main():\n    values: list[list[set[int]]] = [[{2}], [{1}]]\n    values.sort()\n",
        "def main():\n    values: list[list[dict[str, int]]] = [[{\"b\": 2}], [{\"a\": 1}]]\n    values.sort()\n",
        "class Item:\n    value: int\n\ndef main():\n    values: list[list[Item]] = [[Item(2)], [Item(1)]]\n    values.sort()\n",
    ];

    for source in sources {
        let errors =
            lower_source(source).expect_err("nested non-total-order elements should be rejected");
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                    && error.message
                        == "list.sort() requires elements with generated Rust total Ord support"
                    && error.primary_range == Some(range_for(source, "sort"))
            }),
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn sorted_rejects_nested_non_total_order_elements_with_iterable_range() {
    let source = "def main():\n    values: list[list[float]] = [[2.0], [1.0]]\n    ordered: list[list[float]] = sorted(values)\n";
    let errors =
        lower_source(source).expect_err("sorted nested partial-order elements should be rejected");

    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message
                    == "sorted() requires an element or key type with generated Rust total Ord support, unavailable for 'list[float]'"
                && error.primary_range
                    == Some(range_for_after_anchor(source, "sorted(", "values"))
        }),
        "{errors:?}"
    );
}
