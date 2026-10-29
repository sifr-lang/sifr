use super::*;

#[test]
pub(super) fn test_string_padding_methods_expose_typed_allocation_failure() {
    let result = lower_source(
        "def pad(value: str, width: int) -> Result[str, OverflowError]:\n    return value.center(width)\n",
    );

    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_string_padding_width_requires_exact_integer() {
    let source = "def main():\n    padded = \"x\".ljust(3.5)\n";
    let errors = lower_source(source).expect_err("float padding width should be rejected");

    assert!(errors.iter().any(|error| {
        error.message.contains("str.ljust() width must be 'int'")
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for(source, "3.5"))
    }));
}

#[test]
pub(super) fn test_varargs_adopt_declared_optional_element_type() {
    let result = lower_source(
        "def count_values(*values: int | None) -> int:\n    return len(values)\n\ndef main():\n    count: int = count_values(1, 2)\n",
    );

    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_duplicate_optional_method_keyword_is_rejected() {
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
pub(super) fn test_user_defined_method_defaults_and_keywords_lower() {
    let result = lower_source(
        "class CounterBox:\n    value: int\n\n    def __init__(self, value: int):\n        self.value = value\n\n    def bump(self, amount: int = 1) -> int:\n        return self.value + amount\n\ndef main():\n    box: CounterBox = CounterBox(4)\n    a: int = box.bump()\n    b: int = box.bump(amount=3)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_break_outside_loop() {
    let source = "def main():\n    break\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("'break' outside of loop")
                && e.code == Some(DiagnosticCode::FLOW_BREAK_OUTSIDE_LOOP)
                && e.primary_range == Some(range_for(source, "break")))
    );
}

#[test]
pub(super) fn test_continue_outside_loop() {
    let source = "def main():\n    continue\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("'continue' outside of loop")
                && e.code == Some(DiagnosticCode::FLOW_CONTINUE_OUTSIDE_LOOP)
                && e.primary_range == Some(range_for(source, "continue")))
    );
}

#[test]
pub(super) fn test_continue_guard_narrows_optional_for_rest_of_loop_body() {
    let result = lower_source(
        "def sum_present(values: list[int | None]) -> int:\n    total = 0\n    for value in values:\n        if value is None:\n            continue\n        total += value\n    return total\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_break_guard_narrows_optional_for_rest_of_loop_body() {
    let result = lower_source(
        "def sum_until_gap(mut values: list[int | None]) -> int:\n    total = 0\n    while len(values) > 0:\n        value: int | None = values.pop()\n        if value is None:\n            break\n        total += value\n    return total\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_break_inside_loop() {
    let module = lower_source("def main():\n    while True:\n        break\n").unwrap();
    assert_eq!(module.functions.len(), 1);
}

#[test]
pub(super) fn test_nested_loops() {
    let module = lower_source(
        "def main():\n    for i in range(3):\n        for j in range(2):\n            print(i)\n",
    )
    .unwrap();
    assert_eq!(module.functions.len(), 1);
}

#[test]
pub(super) fn test_fstring_basic() {
    let module = lower_source(
        "def main():\n    name: str = \"Alice\"\n    msg: str = f\"Hello, {name}!\"\n    print(msg)\n",
    )
    .unwrap();
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].body.len(), 3);
}

#[test]
pub(super) fn test_fstring_with_expression() {
    let module = lower_source(
        "def main():\n    a: int = 2\n    b: int = 3\n    print(f\"{a} + {b} = {a + b}\")\n",
    )
    .unwrap();
    assert_eq!(module.functions.len(), 1);
}

#[test]
pub(super) fn test_tuple_unpack() {
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
pub(super) fn test_tuple_unpack_wrong_count() {
    let source = "def main():\n    pair: tuple[int, str] = (1, \"hello\")\n    x, y, z = pair\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("expected 3 values, got 2")
                && e.code == Some(DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH)
                && e.primary_range == Some(range_for(source, "x, y, z")))
    );
}

#[test]
pub(super) fn test_tuple_unpack_non_tuple() {
    let source = "def main():\n    x: int = 42\n    a, b = x\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("cannot unpack non-tuple")
                && e.code == Some(DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH)
                && e.primary_range == Some(range_for_after(source, "a, b = ", "x")))
    );
}

#[test]
pub(super) fn test_tuple_unpack_invalid_target_has_unpack_code() {
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
pub(super) fn test_tuple_unpack_reassignment_type_mismatch_has_primary_range() {
    let source = "def main():\n    left = 1\n    left, label = (\"not an int\", \"name\")\n    print(label)\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected tuple unpack reassignment type mismatch");
    assert!(errors.iter().any(
        |e| e.message.contains("cannot assign 'str' to variable 'left'")
            && e.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && e.primary_range == Some(range_for_after_anchor(source, "left = 1\n    ", "left"))
    ));
}

#[test]
pub(super) fn test_star_unpack_multiple_starred_targets_have_unpack_code() {
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
pub(super) fn test_star_unpack_invalid_starred_target_has_unpack_code() {
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
pub(super) fn test_star_unpack_invalid_trailing_target_has_unpack_code() {
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
pub(super) fn test_star_unpack_requires_list_has_primary_range() {
    let source = "def main():\n    first, *rest = (1, 2, 3)\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected star unpack list-shape error");
    assert!(errors.iter().any(
        |e| e.code == Some(DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH)
            && e.primary_range == Some(range_for(source, "(1, 2, 3)"))
    ));
}

#[test]
pub(super) fn test_star_unpack_records_existing_target_rebindings() {
    let module = lower_source(
        "def main():\n    first = 0\n    middle: list[int] = []\n    last = 0\n    try:\n        first, *middle, last = [1, 2, 3]\n    except ValueError:\n        pass\n",
    )
    .expect("star unpack should lower");
    let HirStmt::TryExcept { body, .. } = &module.functions[0].body[3] else {
        panic!("expected try/except HIR");
    };
    let HirStmt::StarUnpack {
        before,
        star,
        after,
        ..
    } = &body[0]
    else {
        panic!("expected star unpack HIR");
    };
    assert!(before[0].rebind_existing);
    assert!(star.rebind_existing);
    assert!(after[0].rebind_existing);
}

#[test]
pub(super) fn test_tuple_unpack_allows_attribute_targets() {
    let module = lower_source(
        "class Pair:\n    x: int\n    y: int\n    def __init__(self):\n        self.x = 1\n        self.y = 2\n    def swap(mut self):\n        self.x, self.y = self.y, self.x\n",
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
pub(super) fn test_for_tuple_target_requires_tuple_elements() {
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
pub(super) fn test_for_tuple_target_arity_mismatch_has_primary_range() {
    let source = "def main():\n    pairs: list[tuple[int, int, int]] = [(1, 2, 3)]\n    for a, b in pairs:\n        print(a)\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected for tuple target arity mismatch");
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("expects 2 element(s)")
                && e.code == Some(DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH)
                && e.primary_range == Some(range_for(source, "a, b")))
    );
}

#[test]
pub(super) fn test_generic_class_subscript_requires_declared_type_params() {
    let source = "T = TypeVar(\"T\")\nclass LegacyBox:\n    value: T\ndef f(x: LegacyBox[int]) -> int:\n    return 1\n";
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
pub(super) fn test_generic_class_subscript_arity_mismatch_errors() {
    let source = "class Pair[T]:\n    left: T\n    right: T\ndef f(x: Pair[int, str]) -> int:\n    return 1\n";
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
pub(super) fn test_invalid_dict_type_annotation_has_primary_range() {
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
pub(super) fn test_callable_param_list_annotation_has_primary_range() {
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
pub(super) fn test_missing_function_parameter_annotation_has_primary_range() {
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
pub(super) fn test_missing_class_method_parameter_annotation_has_primary_range() {
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
pub(super) fn test_unsupported_function_default_argument_has_primary_range() {
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
pub(super) fn test_unsupported_method_default_argument_has_primary_range() {
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
pub(super) fn test_unknown_type_annotation_has_primary_range() {
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
pub(super) fn test_unknown_generic_type_annotation_has_primary_range() {
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
pub(super) fn test_typevar_constraints_violation_has_type_code() {
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
pub(super) fn test_typevar_invalid_bound_shape_has_primary_range() {
    let source = "from typing import TypeVar\n\nT = TypeVar(\"T\", bound=1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "TypeVar bound must be a type name"
            && error.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)
            && error.primary_range == Some(range_for_after_anchor(source, "bound=", "1"))
    }));
}

#[test]
pub(super) fn test_typevar_bound_constraints_conflict_has_primary_range() {
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
pub(super) fn test_pep695_typevar_constraint_shape_has_primary_range() {
    let source = "def echo[T: (int, 1)](x: T) -> T:\n    return x\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "TypeVar constraints must be type names"
            && error.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)
            && error.primary_range == Some(range_for_after_anchor(source, "(int, ", "1"))
    }));
}

#[test]
pub(super) fn test_auto_init_inheritance_missing_super_has_class_code() {
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
pub(super) fn test_auto_init_required_after_default_has_class_code() {
    let source = "class BadConfig:\n    debug: bool = False\n    name: str\n\ndef main():\n    c: BadConfig = BadConfig(True, \"test\")\n";
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
pub(super) fn test_enum_duplicate_value_has_class_code() {
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
pub(super) fn test_missing_field_has_class_code() {
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
pub(super) fn test_enum_missing_attribute_has_class_code() {
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
pub(super) fn test_unsupported_attribute_expression_has_type_code() {
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
pub(super) fn test_super_outside_parent_has_class_code() {
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
pub(super) fn test_missing_class_static_method_has_class_code() {
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
pub(super) fn test_unknown_parent_class_has_class_code() {
    let source =
        "class Child(MissingParent):\n    value: int\n\ndef main():\n    c: Child = Child(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| {
            e.message
                == "invalid base class for 'Child': parent type 'MissingParent' is not a class"
                && e.code == Some(DiagnosticCode::CLASS_INVALID_BASE)
                && e.primary_range == Some(range_for_after(source, "class Child(", "MissingParent"))
        }),
        "{errors:#?}"
    );
}

#[test]
pub(super) fn test_unsupported_class_field_default_has_class_code() {
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
pub(super) fn test_match_tuple_pattern_requires_tuple_subject() {
    let result = lower_source(
        "def main():\n    x: int = 1\n    match x:\n        case (a, b):\n            print(a)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("tuple pattern requires subject of tuple type")
    }));
}

#[test]
pub(super) fn test_match_tuple_pattern_arity_mismatch_errors() {
    let result = lower_source(
        "def main():\n    x: tuple[int, int] = (1, 2)\n    match x:\n        case (a, b, c):\n            print(a)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("tuple pattern expects 3 element(s), subject has 2")
    }));
}

#[test]
pub(super) fn test_protocol_bound_forwarding_accepts_conforming_typevar() {
    let result = lower_source(
        "class Runner(Protocol):\n    def run(self) -> int:\n        pass\n\nclass Job:\n    def run(self) -> int:\n        return 1\n\ndef use_runner[T: Runner](x: T) -> T:\n    return x\n\ndef relay_runner[U: Runner](x: U) -> U:\n    return use_runner(x)\n\ndef main():\n    j: Job = relay_runner(Job())\n    print(j.run())\n",
    );
    assert!(result.is_ok());
}

#[test]
pub(super) fn test_generic_declaration_reports_only_unknown_bound_name() {
    let result = lower_source(
        "def take_missing[T: MissingBound](x: T) -> T:\n    return x\n\ndef main():\n    print(1)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:?}");
    assert_eq!(errors[0].code, Some(DiagnosticCode::NAME_UNKNOWN_TYPE));
    assert_eq!(errors[0].message, "unknown type: 'MissingBound'");
}

#[test]
pub(super) fn test_protocol_bound_forwarding_rejects_non_conforming_typevar() {
    let result = lower_source(
        "class MissingBound(Protocol):\n    def required(self) -> int:\n        pass\n\ndef take_missing[T: MissingBound](x: T) -> T:\n    return x\n\ndef relay_missing[U](x: U) -> U:\n    return take_missing(x)\n\ndef main():\n    print(1)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:?}");
    assert_eq!(
        errors[0].code,
        Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
    );
    assert!(
        errors[0]
            .message
            .contains("does not implement protocol 'MissingBound'")
    );
}

#[test]
pub(super) fn test_comparable_bound_accepts_homogeneous_tuples() {
    let result = lower_source(
        "def choose[T: Comparable](x: T, y: T) -> T:\n    return x if x > y else y\n\ndef main():\n    left: tuple[int, int] = (1, 2)\n    right: tuple[int, int] = (2, 1)\n    out: tuple[int, int] = choose(left, right)\n    print(out)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_recursive_tree_attributes_narrow_after_truthiness_or_guard() {
    let result = lower_source(
        "class TreeNode:\n    val: int\n    left: TreeNode | None\n    right: TreeNode | None\n\n    def __init__(self, val: int, left: TreeNode | None, right: TreeNode | None):\n        self.val = val\n        self.left = left\n        self.right = right\n\ndef paired_tree_value_sum(p: TreeNode | None, q: TreeNode | None) -> int:\n    if not p and not q:\n        return 0\n    if not p or not q:\n        return 0\n    left: TreeNode | None = p.left\n    right: TreeNode | None = q.right\n    return p.val + q.val + paired_tree_value_sum(left, q.left) + paired_tree_value_sum(p.right, right)\n",
    );
    assert!(
        result.is_ok(),
        "recursive tree attributes should lower after `if not p or not q` early-return narrowing"
    );
}

#[test]
pub(super) fn test_empty_dict_literal_specializes_from_first_subscript_write_and_get_default() {
    let result = lower_source(
        "def main():\n    counts = {}\n    key: str = \"x\"\n    counts[key] = 1 + counts.get(key, 0)\n    value: int = counts.get(key, 0)\n    assert value == 1\n",
    );
    assert!(
        result.is_ok(),
        "empty dict literal should specialize to dict[str, int] from first write/get-default flow"
    );
}

#[test]
pub(super) fn test_empty_dict_literal_conflicting_write_reports_deterministic_error() {
    let result =
        lower_source("def main():\n    data = {}\n    data[1] = 10\n    data[\"x\"] = 20\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("empty literal type conflict"))
    );
}

#[test]
pub(super) fn test_empty_dict_specialization_with_split_zip_word_pattern_shape() {
    let result = lower_source(
        "def wordPattern(pattern: str, s: str) -> bool:\n    words = s.split(\" \")\n    if len(pattern) != len(words):\n        return False\n    charToWord = {}\n    wordToChar = {}\n    for c, w in zip(pattern, words):\n        if c in charToWord and charToWord[c] != w:\n            return False\n        if w in wordToChar and wordToChar[w] != c:\n            return False\n        charToWord[c] = w\n        wordToChar[w] = c\n    return True\n",
    );
    assert!(
        result.is_ok(),
        "word-pattern split/zip flow should specialize empty dicts to dict[str, str]: {:?}",
        result.err()
    );
}
