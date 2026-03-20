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
        HirExpr::Call { func, args, ty }
            if func == "iter" && args.len() == 1 && matches!(ty, Type::Iterator(_))
    ));
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
fn test_generator_rejects_nested_yield_shape() {
    let result = lower_source(
        "def bad(n: int):\n    i: int = 0\n    while i < n:\n        while i < n:\n            yield i\n            i = i + 1\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| { e.message.contains("unsupported lazy generator shape") }));
}

#[test]
fn test_generator_rejects_trailing_statements_after_loop() {
    let result = lower_source(
        "def bad(n: int):\n    i: int = 0\n    while i < n:\n        yield i\n        i = i + 1\n    i = i + 1\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("cannot have trailing statements after the generator while loop")
    }));
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
