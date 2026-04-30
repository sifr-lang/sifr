use crate::{lower_module, HirExpr, HirModule, HirStmt, LoweringError};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;
use sifr_type_system::Type;

fn lower_source(source: &str) -> Result<HirModule, Vec<LoweringError>> {
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).map(|r| r.module)
}

#[test]
fn test_simple_function() {
    let module = lower_source("def add(a: int, b: int) -> int:\n    return a + b\n").unwrap();
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "add");
    assert_eq!(module.functions[0].return_type, Type::Int);
}

#[test]
fn test_type_mismatch_error() {
    let result = lower_source("def main():\n    x: int = \"hello\"\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("type mismatch")));
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
fn test_use_after_move() {
    let result = lower_source(
        "def consume(own s: str) -> str:\n    return s\ndef main():\n    s: str = \"hello\"\n    x: str = consume(s)\n    print(s)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("moved value") && e.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
    }));
}

#[test]
fn test_double_mutable_borrow_has_ownership_code() {
    let result = lower_source(
        "def swap(mut a: list[int], mut b: list[int]):\n    pass\n\ndef main():\n    items: list[int] = [1, 2, 3]\n    swap(items, items)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("cannot borrow 'items' as mutable more than once")
            && e.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
    }));
}

#[test]
fn test_mutable_after_immutable_borrow_has_ownership_code() {
    let result = lower_source(
        "def read_then_mutate(a: list[int], mut b: list[int]):\n    pass\n\ndef main():\n    items: list[int] = [1, 2, 3]\n    read_then_mutate(items, items)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains(
            "cannot borrow 'items' as mutable because it is already borrowed as immutable",
        ) && e.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
    }));
}

#[test]
fn test_immutable_after_mutable_borrow_has_ownership_code() {
    let result = lower_source(
        "def mutate_then_read(mut a: list[int], b: list[int]):\n    pass\n\ndef main():\n    items: list[int] = [1, 2, 3]\n    mutate_then_read(items, items)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains(
            "cannot borrow 'items' as immutable because it is already borrowed as mutable",
        ) && e.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
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
    let result = lower_source(
        "def main():\n    groups = defaultdict(default_factory=list)\n    _ = groups\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "defaultdict() does not support keyword arguments"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
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
        errors
            .iter()
            .any(|error| error.message.contains("unsupported operand type(s) for +")),
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
        errors
            .iter()
            .any(|error| error.message.contains("unsupported operand type(s) for +")),
        "lowering should reach operand typing for nested subscript augassign: {errors:?}"
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
    let result = lower_source("def main():\n    if 1:\n        pass\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("if condition must be bool or collection/string truthiness")));
}

#[test]
fn test_while_condition_rejects_numeric_truthiness() {
    let result = lower_source("def main():\n    while 1:\n        return\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("while condition must be bool or collection/string truthiness")));
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
    let result = lower_source("def f(flag: bool) -> int:\n    if flag:\n        return 1\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("must return a value of type 'int' on all control-flow paths")));
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
    let result = lower_source(
        "def same() -> bool:\n    return True\n\ndef same() -> bool:\n    return False\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("duplicate function definition in module: 'same'")));
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
    let result = lower_source(
        "def count_up(n: int) -> list[int]:\n    i: int = 0\n    while i < n:\n        yield i\n        i = i + 1\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| { e.message.contains("must declare return type 'Iterator[T]'") }));
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

    let unexpected_result = lower_source(
        "def main():\n    nums: list[int] = [1, 2]\n    _paired = zip(nums, nums, bogus=True)\n",
    );
    assert!(unexpected_result.is_err());
    let unexpected_errors = unexpected_result.unwrap_err();
    assert!(unexpected_errors.iter().any(|error| {
        error
            .message
            .contains("zip() got an unexpected keyword argument 'bogus'")
    }));
}

#[test]
fn test_reversed_rejects_non_reversible_iterator_argument() {
    let result = lower_source(
        "def main():\n    nums: list[int] = [1, 2, 3]\n    it: Iterator[int] = iter(nums)\n    _rev = reversed(it)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("reversed() argument must be reversible")));
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
    let result = lower_source(
        "def add(x: int) -> int:\n    return x + 1\n\ndef main():\n    nums: list[int] = [1, 2]\n    _mapped = map(function=add, iterable=nums)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("map() does not accept keyword arguments")
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
    let result = lower_source(
        "def pred(x: int) -> bool:\n    return x > 0\n\ndef main():\n    nums: list[int] = [1, 2]\n    _filtered = filter(function=pred, iterable=nums)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("filter() does not accept keyword arguments")
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
fn test_max_two_arg_rejects_optional_operand() {
    let result = lower_source(
        "def pick(d: dict[str, int], k: str) -> int:\n    best = 0\n    best = max(best, d[k])\n    return best\n",
    );
    assert!(result.is_err(), "max(i64, i64|None) should be rejected");
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("max() with 2 arguments does not accept optional operands")));
}

#[test]
fn test_sorted_accepts_iterable_keyword_and_key_none() {
    let result = lower_source(
        "def main():\n    nums: list[int] = [3, 1, 2]\n    ordered: list[int] = sorted(iterable=nums, key=None, reverse=True)\n    assert ordered == [3, 2, 1]\n",
    );
    assert!(result.is_ok());
}

#[test]
fn test_sorted_rejects_duplicate_iterable_argument() {
    let result = lower_source(
        "def main():\n    nums: list[int] = [3, 1, 2]\n    ordered: list[int] = sorted(nums, iterable=nums)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("sorted() got multiple values for argument 'iterable'")));
}

#[test]
fn test_list_sort_accepts_reverse_keyword() {
    let result =
        lower_source("def main():\n    nums: list[int] = [3, 1, 2]\n    nums.sort(reverse=True)\n");
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_list_sort_rejects_non_bool_reverse_keyword() {
    let result =
        lower_source("def main():\n    nums: list[int] = [3, 1, 2]\n    nums.sort(reverse=1)\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("list.sort() argument 'reverse' must be 'bool'")));
}

#[test]
fn test_tuple_constructor_rejects_dynamic_list_shape() {
    let result =
        lower_source("def main():\n    nums: list[int] = [1, 2, 3]\n    t = tuple(nums)\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("tuple() currently requires a tuple, list literal, or string literal")
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
    let result = lower_source("def main():\n    xs: list[int] = [1]\n    xs.append(value=2)\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("append() got an unexpected keyword argument 'value'")));
}

#[test]
fn test_duplicate_optional_method_keyword_is_rejected() {
    let result = lower_source(
        "def main():\n    data: dict[str, int] = {\"x\": 1}\n    value: int = data.get(\"x\", 1, default=2)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e
        .message
        .contains("get() got multiple values for argument 'default'")));
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
    let result = lower_source("def main():\n    break\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("'break' outside of loop")
            && e.code == Some(DiagnosticCode::FLOW_BREAK_OUTSIDE_LOOP)));
}

#[test]
fn test_continue_outside_loop() {
    let result = lower_source("def main():\n    continue\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("'continue' outside of loop")
            && e.code == Some(DiagnosticCode::FLOW_CONTINUE_OUTSIDE_LOOP)));
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
    let result = lower_source(
        "def main():\n    pair: tuple[int, str] = (1, \"hello\")\n    x, y, z = pair\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("expected 3 values, got 2")));
}

#[test]
fn test_tuple_unpack_non_tuple() {
    let result = lower_source("def main():\n    x: int = 42\n    a, b = x\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("cannot unpack non-tuple")));
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
    let result = lower_source(
        "def main():\n    nums: list[int] = [1, 2, 3]\n    for a, b in nums:\n        print(a)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("for loop tuple target expects iterable elements of tuple type")
    }));
}

#[test]
fn test_generic_class_subscript_requires_declared_type_params() {
    let result = lower_source(
        "T = TypeVar(\"T\")\nclass LegacyBox:\n    value: T\ndef f(x: LegacyBox[int]) -> int:\n    return 1\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("does not declare type parameters")
            && e.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)));
}

#[test]
fn test_generic_class_subscript_arity_mismatch_errors() {
    let result = lower_source(
        "class Pair[T]:\n    left: T\n    right: T\ndef f(x: Pair[int, str]) -> int:\n    return 1\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("expects 1 type argument(s), got 2")
            && e.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)));
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
fn test_auto_init_inheritance_missing_super_has_class_code() {
    let result = lower_source(
        "class Animal:\n    name: str\n\n    def __init__(self, name: str):\n        self.name = name\n\nclass Dog(Animal):\n    breed: str\n\ndef main():\n    d: Dog = Dog(\"Rex\", \"Labrador\")\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            == "class 'Dog' has fields but no __init__; parent fields will not be initialized. Define an explicit __init__ with super().__init__(...)"
            && e.code == Some(DiagnosticCode::CLASS_MISSING_INITIALIZER)
    }));
}

#[test]
fn test_auto_init_required_after_default_has_class_code() {
    let result = lower_source(
        "class BadConfig:\n    debug: bool = False\n    name: str\n\ndef main():\n    c: BadConfig = BadConfig(True, \"test\")\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            == "class 'BadConfig': required field 'name' declared after field with default value"
            && e.code == Some(DiagnosticCode::CLASS_REQUIRED_FIELD_AFTER_DEFAULT)
    }));
}

#[test]
fn test_enum_duplicate_value_has_class_code() {
    let result = lower_source(
        "from enum import Enum\n\nclass Status(Enum):\n    OK = 200\n    SUCCESS = 200\n    NOT_FOUND = 404\n\ndef main():\n    s: Status = Status.OK\n    print(s)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message == "enum 'Status' has duplicate value 200: variants 'OK' and 'SUCCESS'"
            && e.code == Some(DiagnosticCode::CLASS_DUPLICATE_OR_INVALID_VALUE)
    }));
}

#[test]
fn test_missing_field_has_class_code() {
    let result = lower_source(
        "class Point:\n    x: float\n    y: float\n\n    def __init__(self, x: float, y: float):\n        self.x = x\n        self.y = y\n\ndef main():\n    p: Point = Point(1.0, 2.0)\n    print(p.z)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message == "type 'Point' has no field 'z'"
            && e.code == Some(DiagnosticCode::CLASS_MISSING_MEMBER)
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
