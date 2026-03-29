use crate::{lower_module, HirExpr, HirModule, HirStmt, LoweringError};
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
fn test_use_after_move() {
    let result = lower_source(
        "def consume(own s: str) -> str:\n    return s\ndef main():\n    s: str = \"hello\"\n    x: str = consume(s)\n    print(s)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("moved value")));
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
fn test_unguarded_list_pop_stays_optional() {
    let result = lower_source("def main():\n    values: list[int] = [1, 2]\n    item: int = values.pop()\n");
    assert!(
        result.is_err(),
        "unguarded list.pop() should remain optional"
    );
}

#[test]
fn test_unguarded_zero_index_list_pop_stays_optional() {
    let result =
        lower_source("def main():\n    values: list[int] = [1, 2]\n    item: int = values.pop(0)\n");
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
                "legacy iterator builtin call node found in canonical wave2 lowering: {value:?}"
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
fn test_sum_min_max_accept_iterator_inputs() {
    let result = lower_source(
        "def main():\n    nums: list[int] = [3, 1, 2]\n    total: int = sum(iter(nums))\n    lo: int | None = min(iter(nums))\n    hi: int | None = max(iter(nums))\n",
    );
    assert!(result.is_ok(), "{result:?}");
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
        .any(|e| e.message.contains("'break' outside of loop")));
}

#[test]
fn test_continue_outside_loop() {
    let result = lower_source("def main():\n    continue\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("'continue' outside of loop")));
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
        .any(|e| e.message.contains("does not declare type parameters")));
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
        .any(|e| e.message.contains("expects 1 type argument(s), got 2")));
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
    let result = lower_source("def main():\n    data = {}\n    data[1] = 10\n    data[\"x\"] = 20\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.message.contains("empty literal type conflict")));
}
