use super::*;
#[test]
pub(super) fn test_empty_list_specializes_on_append_and_satisfies_return_type() {
    let result = lower_source(
        "def collect() -> list[int]:\n    res = []\n    res.append(1)\n    return res\n",
    );
    assert!(
        result.is_ok(),
        "empty list should specialize to list[int] after append"
    );
}

#[test]
pub(super) fn test_empty_list_specializes_on_insert_and_extend() {
    let result = lower_source(
        "def collect() -> list[int]:\n    res = []\n    res.insert(0, 1)\n    res.extend([2, 3])\n    return res\n",
    );
    assert!(
        result.is_ok(),
        "empty list should specialize from insert/extend element types"
    );
}

#[test]
pub(super) fn test_empty_list_specialization_rejects_mixed_append_types() {
    let result = lower_source(
        "def collect() -> list[int]:\n    res = []\n    res.append(1)\n    res.append(\"x\")\n    return res\n",
    );
    assert!(
        result.is_err(),
        "after first append specialization, mixed element types must fail"
    );
}

#[test]
pub(super) fn test_empty_list_specialization_survives_loop_append() {
    let result = lower_source(
        "def collect(values: list[int]) -> list[int]:\n    res = []\n    i = 0\n    while i < len(values):\n        res.append(values[i])\n        i += 1\n    return res\n",
    );
    assert!(
        result.is_ok(),
        "loop-body append specialization should persist so return boundary sees list[int]"
    );
}

#[test]
pub(super) fn test_generic_class_receiver_refines_from_method_arguments() {
    let result = lower_source(
        "class Bucket[T]:\n    items: list[T]\n\n    def __init__(self):\n        self.items = []\n\n    def push(mut self, value: T) -> None:\n        self.items.append(value)\n\n    def first(self) -> T | None:\n        if len(self.items) == 0:\n            return None\n        return self.items[0]\n\ndef main() -> int:\n    bucket = Bucket()\n    bucket.push(1)\n    value = bucket.first()\n    if value is None:\n        return 0\n    return value + 1\n",
    );
    assert!(
        result.is_ok(),
        "receiver generic type vars should refine from method arguments"
    );
}

#[test]
pub(super) fn test_generic_class_receiver_refinement_rejects_mixed_argument_types() {
    let result = lower_source(
        "class Bucket[T]:\n    items: list[T]\n\n    def __init__(self):\n        self.items = []\n\n    def push(mut self, value: T) -> None:\n        self.items.append(value)\n\ndef main() -> None:\n    bucket = Bucket()\n    bucket.push(1)\n    bucket.push(\"x\")\n",
    );
    assert!(
        result.is_err(),
        "once method-driven specialization binds T, incompatible argument types must fail"
    );
}

#[test]
pub(super) fn test_empty_list_specialization_optional_append_in_loop_rejects_return_annotation() {
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
pub(super) fn test_copy_type_no_move() {
    let module =
        lower_source("def main():\n    x: int = 42\n    print(x)\n    print(x)\n").unwrap();
    assert_eq!(module.functions.len(), 1);
}

#[test]
pub(super) fn test_while_loop() {
    let module =
        lower_source("def main():\n    i: int = 0\n    while i < 10:\n        i = i + 1\n")
            .unwrap();
    assert_eq!(module.functions.len(), 1);
    assert!(module.functions[0].body.len() >= 2);
    assert!(matches!(module.functions[0].body[1], HirStmt::While { .. }));
}

#[test]
pub(super) fn test_if_else_branch_bindings_are_visible_after_if() {
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
pub(super) fn test_if_condition_rejects_numeric_truthiness() {
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
pub(super) fn test_while_condition_rejects_numeric_truthiness() {
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
pub(super) fn test_elif_condition_rejects_numeric_truthiness_with_primary_range() {
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
pub(super) fn test_class_truthiness_allowed_in_if_while_and_boolop() {
    let result = lower_source(
        "class Node:\n    val: int\n    def __init__(self, val: int):\n        self.val = val\n\ndef probe(a: Node, b: Node) -> bool:\n    seen: bool = False\n    if a:\n        seen = True\n    while b:\n        break\n    return a and b and seen\n",
    );
    assert!(
        result.is_ok(),
        "class instances should be valid truthiness operands in control-flow and boolops"
    );
}

#[test]
pub(super) fn test_non_none_return_annotation_requires_exhaustive_returns() {
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
pub(super) fn test_invalid_return_expression_does_not_emit_missing_return_cascade() {
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
pub(super) fn test_duplicate_module_function_definition_reports_error() {
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
pub(super) fn test_guarded_list_pop_narrows_to_element_type() {
    let result = lower_source(
        "def main():\n    values: list[int] = [1, 2]\n    while values:\n        item: int = values.pop()\n",
    );
    assert!(
        result.is_ok(),
        "list.pop() under non-empty guard should narrow to element type"
    );
}

#[test]
pub(super) fn test_guarded_zero_index_list_pop_narrows_to_element_type() {
    let result = lower_source(
        "def main():\n    values: list[int] = [1, 2]\n    while values:\n        item: int = values.pop(0)\n",
    );
    assert!(
        result.is_ok(),
        "list.pop(0) under non-empty guard should narrow to element type"
    );
}

#[test]
pub(super) fn test_guarded_list_pop_on_field_access_narrows_to_element_type() {
    let result = lower_source(
        "class Q:\n    data: list[int]\n\n    def __init__(self):\n        self.data = [1, 2]\n\n    def pop_one(mut self) -> int:\n        while self.data:\n            item: int = self.data.pop()\n            return item\n        return 0\n",
    );
    assert!(
        result.is_ok(),
        "list.pop() under non-empty field guard should narrow to element type"
    );
}

#[test]
pub(super) fn test_guarded_list_pop_preserves_optional_element_none() {
    let result = lower_source(
        "def main():\n    values: list[int | None] = []\n    values.append(1)\n    values.append(None)\n    while values:\n        item: int | None = values.pop()\n",
    );
    assert!(
        result.is_ok(),
        "list.pop() under non-empty guard should keep element-level optionality"
    );
}

#[test]
pub(super) fn test_guarded_list_pop_optional_element_rejects_non_optional_annotation() {
    let result = lower_source(
        "def main():\n    values: list[int | None] = []\n    values.append(1)\n    values.append(None)\n    while values:\n        item: int = values.pop()\n",
    );
    assert!(
        result.is_err(),
        "non-empty guard must not erase element-level None from list[int|None].pop()"
    );
}

#[test]
pub(super) fn test_unguarded_list_pop_stays_optional() {
    let result =
        lower_source("def main():\n    values: list[int] = [1, 2]\n    item: int = values.pop()\n");
    assert!(
        result.is_err(),
        "unguarded list.pop() should remain optional"
    );
}

#[test]
pub(super) fn test_unguarded_zero_index_list_pop_stays_optional() {
    let result = lower_source(
        "def main():\n    values: list[int] = [1, 2]\n    item: int = values.pop(0)\n",
    );
    assert!(
        result.is_err(),
        "unguarded list.pop(0) should remain optional"
    );
}

#[test]
pub(super) fn test_guarded_indexed_list_pop_stays_optional() {
    let result = lower_source(
        "def main():\n    values: list[int] = [1, 2]\n    while values:\n        item: int = values.pop(5)\n",
    );
    assert!(
        result.is_err(),
        "indexed list.pop(i) should remain optional under non-empty guard"
    );
}

#[test]
pub(super) fn test_guarded_dict_pop_stays_optional() {
    let result = lower_source(
        "def main():\n    values: dict[str, int] = {\"x\": 1}\n    if values:\n        item: int = values.pop(\"missing\")\n",
    );
    assert!(
        result.is_err(),
        "dict.pop(key) should remain optional under dict truthiness guard"
    );
}

#[test]
pub(super) fn test_boolop_and_short_circuit_narrows_guarded_index_operand() {
    let result = lower_source(
        "def main():\n    values: list[int] = [1, 2, 3]\n    i: int = 1\n    ok: bool = i < len(values) and values[i] > 0\n    assert ok == True\n",
    );
    assert!(
        result.is_ok(),
        "`and` short-circuit should apply sequence guard facts to the RHS operand"
    );
}

#[test]
pub(super) fn test_boolop_or_short_circuit_narrows_rhs_after_not_empty_guard() {
    let result = lower_source(
        "def probe(stack: list[int]) -> bool:\n    return not stack or stack[0] > 0\n\ndef main():\n    assert probe([]) == True\n    assert probe([1, 2]) == True\n",
    );
    assert!(
        result.is_ok(),
        "`or` short-circuit should apply false-branch guard facts to the RHS operand"
    );
}

#[test]
pub(super) fn test_boolop_and_without_sequence_guard_keeps_optional_index_error() {
    let result = lower_source(
        "def read_without_len_guard(values: list[int], i: int) -> int:\n    if True and i >= 0:\n        value: int = values[i]\n        return value\n    return 0\n",
    );
    assert!(
        result.is_err(),
        "`and` without an explicit sequence guard should not narrow index access"
    );
}

#[test]
pub(super) fn test_tuple_literal_index_uses_exact_position_type() {
    let result = lower_source(
        "def main() -> int:\n    pair: tuple[str, int] = (\"x\", 7)\n    value: int = pair[1]\n    return value\n",
    );
    assert!(
        result.is_ok(),
        "tuple[1] should resolve to the second element type"
    );
}

#[test]
pub(super) fn test_tuple_nonliteral_index_uses_union_of_element_types() {
    let result = lower_source(
        "def bad(i: int) -> int:\n    pair: tuple[int, str] = (1, \"x\")\n    value: int = pair[i]\n    return value\n",
    );
    assert!(
        result.is_err(),
        "non-literal tuple index should be typed as a union of element types"
    );
}

#[test]
pub(super) fn test_if_expr_optional_branch_does_not_implicitly_unwrap() {
    let result = lower_source(
        "def pick(x: int | None) -> int:\n    value: int = x if x is not None else 0\n    return value\n",
    );
    assert!(
        result.is_err(),
        "ternary optional branch should not implicitly unwrap Option values"
    );
}

#[test]
pub(super) fn test_if_expr_true_branch_sequence_guard_narrows_index() {
    let result = lower_source(
        "def pick(values: list[int], i: int) -> int:\n    value: int = values[i] if i < len(values) else 0\n    return value\n",
    );
    assert!(
        result.is_ok(),
        "ternary true branch should honor index guard and produce definite element type"
    );
}

#[test]
pub(super) fn test_if_expr_true_branch_sequence_guard_narrows_index_with_offset() {
    let result = lower_source(
        "def pick(values: list[int], i: int) -> int:\n    value: int = values[i + 1] if i + 1 < len(values) else 0\n    return value\n",
    );
    assert!(
        result.is_ok(),
        "ternary true branch should honor offset index guard and produce definite element type"
    );
}

#[test]
pub(super) fn test_while_not_none_narrows_optional_receiver_for_attribute_access() {
    let result = lower_source(
        "class Node:\n    val: int\n    next: Node | None\n\n    def __init__(self, val: int, next: Node | None):\n        self.val = val\n        self.next = next\n\ndef total(own head: Node | None) -> int:\n    cur: Node | None = head\n    acc: int = 0\n    while cur is not None:\n        acc = acc + cur.val\n        cur = cur.next\n    return acc\n",
    );
    assert!(
        result.is_ok(),
        "`while x is not None` should narrow optional receivers inside the loop body"
    );
}

#[test]
pub(super) fn test_inferred_local_can_widen_to_optional_on_reassignment() {
    let result = lower_source(
        "def pick(head: int | None) -> int:\n    cur = None\n    cur = head\n    if cur is None:\n        return 0\n    return cur\n",
    );
    assert!(
        result.is_ok(),
        "inferred locals should widen to Optional under reassignment from/to None"
    );
}

#[test]
pub(super) fn test_optional_reassignment_invalidates_non_none_narrowing() {
    let result = lower_source(
        "def bad(mut x: int | None) -> int:\n    if x is not None:\n        x = None\n        return x\n    return 0\n",
    );
    assert!(
        result.is_err(),
        "rebinding an Optional must invalidate prior non-None narrowing"
    );
}

#[test]
pub(super) fn test_sequence_reassignment_invalidates_index_guard() {
    let result = lower_source(
        "def bad(mut values: list[int], i: int) -> int:\n    if i < len(values):\n        values = []\n        value: int = values[i]\n        return value\n    return 0\n",
    );
    assert!(
        result.is_err(),
        "rebinding a guarded sequence must invalidate prior index facts"
    );
}

#[test]
pub(super) fn test_index_reassignment_invalidates_index_guard() {
    let result = lower_source(
        "def bad(values: list[int], mut i: int) -> int:\n    if i < len(values):\n        i = len(values)\n        value: int = values[i]\n        return value\n    return 0\n",
    );
    assert!(
        result.is_err(),
        "rebinding a guarded index variable must invalidate prior index facts"
    );
}

#[test]
pub(super) fn test_shrinking_collection_method_invalidates_index_guard() {
    let result = lower_source(
        "def bad(mut values: list[int], i: int) -> int:\n    if i < len(values):\n        values.clear()\n        value: int = values[i]\n        return value\n    return 0\n",
    );
    assert!(
        result.is_err(),
        "a shrinking collection mutation must invalidate prior index facts"
    );
}

#[test]
pub(super) fn test_shrinking_field_collection_method_invalidates_index_guard() {
    let result = lower_source(
        "class Box:\n    values: list[int]\n\n    def __init__(self, values: list[int]):\n        self.values = values\n\n    def bad(mut self, i: int) -> int:\n        if i < len(self.values):\n            self.values.clear()\n            value: int = self.values[i]\n            return value\n        return 0\n",
    );
    assert!(
        result.is_err(),
        "a shrinking collection mutation on a field must invalidate field index facts"
    );
}

#[test]
pub(super) fn test_annotated_local_does_not_widen_on_reassignment() {
    let result =
        lower_source("def bad() -> int:\n    value: int = 1\n    value = None\n    return value\n");
    assert!(
        result.is_err(),
        "explicitly annotated locals should keep their declared type on reassignment"
    );
}

#[test]
pub(super) fn test_for_range() {
    let module = lower_source("def main():\n    for i in range(10):\n        print(i)\n").unwrap();
    assert_eq!(module.functions.len(), 1);
    assert!(matches!(module.functions[0].body[0], HirStmt::For { .. }));
}

#[test]
pub(super) fn test_for_range_start_end() {
    let module =
        lower_source("def main():\n    for i in range(1, 5):\n        print(i)\n").unwrap();
    assert_eq!(module.functions.len(), 1);
    assert!(matches!(module.functions[0].body[0], HirStmt::For { .. }));
}

#[test]
pub(super) fn test_tuple_unpack_len_alias_enables_range_index_guard() {
    let result = lower_source(
        "def sum_all(values: list[int]) -> int:\n    start, n = (0, len(values))\n    total: int = 0\n    for i in range(start, n):\n        total = total + values[i]\n    return total\n",
    );
    assert!(
        result.is_ok(),
        "tuple-unpacked len aliases should feed range-based index guards"
    );
}

#[test]
pub(super) fn test_tuple_unpack_non_len_alias_does_not_enable_range_index_guard() {
    let result = lower_source(
        "def sum_all(values: list[int], n: int) -> int:\n    start, limit = (0, n)\n    total: int = 0\n    for i in range(start, limit):\n        total = total + values[i]\n    return total\n",
    );
    assert!(
        result.is_err(),
        "range-based index guards must not activate for tuple-unpacked non-len aliases"
    );
}

#[test]
pub(super) fn test_for_loop_lowers_through_iter_protocol_call() {
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
        HirExpr::IteratorCall { op, args, ty, .. }
            if op == &crate::hir_nodes::HirIteratorOp::Iter
                && args.len() == 1
                && matches!(ty, Type::Iterator(_))
    ));
}

#[test]
pub(super) fn test_iterator_builtins_lower_to_canonical_iterator_call_nodes() {
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
            HirExpr::Call { func, args, .. } | HirExpr::PythonCall { func, args, .. } => {
                legacy.contains(&func.as_str()) || args.iter().any(call_uses_legacy_iterator_builtin)
            }
            HirExpr::IteratorCall { args, .. }
            | HirExpr::IntrinsicCall { args, .. }
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
            | HirExpr::Await { value: operand, .. }
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
pub(super) fn test_iterable_annotation_accepts_list_argument() {
    let result = lower_source(
        "def consume(xs: Iterable[int]) -> int:\n    total: int = 0\n    for x in xs:\n        total = total + x\n    return total\n\ndef main():\n    values: list[int] = [1, 2, 3]\n    out: int = consume(values)\n    print(out)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_iterator_annotation_rejects_plain_list_argument() {
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
pub(super) fn test_iter_and_next_builtin_protocol_calls_lower() {
    let result = lower_source(
        "def main():\n    values: list[int] = [1, 2, 3]\n    it: Iterator[int] = iter(values)\n    first: int | None = next(it)\n    second: int | None = next(it)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_iter_accepts_homogeneous_tuple_argument() {
    let result = lower_source(
        "def main():\n    values: tuple[int, int, int] = (1, 2, 3)\n    it: Iterator[int] = iter(values)\n    first: int | None = next(it)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}
