use super::generate_rust_from_source;
use crate::{RustExpr, methods, render_expr};
use sifr_ir::{HirExpr, HirFunction, HirParam, HirStmt, MethodKind};
use sifr_type_system::{ParamConvention, Type};
use std::collections::HashMap;

#[test]
fn unicode_indexing_and_character_comparison_have_strict_scan_and_allocation_budgets() {
    let generated = generate_rust_from_source(
        r#"
def recursive_match(depth: int, text: str, expected: str) -> bool:
    if depth == 0:
        return text[0] == expected
    return recursive_match(depth - 1, text, expected)
"#,
    );

    assert!(
        generated.contains("chars().collect::<Vec<char>>()"),
        "{generated}"
    );
    assert!(!generated.contains("chars().count()"), "{generated}");
    assert!(!generated.contains("chars().nth("), "{generated}");
    assert!(!generated.contains("character.to_string()"), "{generated}");
    assert!(generated.contains("__sifr_cmp_chars.next()"), "{generated}");
}

#[test]
fn nested_string_slicing_materializes_unicode_characters_once() {
    let generated = generate_rust_from_source(
        r#"
def measure(text: str) -> int:
    return len(text)

def trim_length(value: str) -> int:
    return measure(value[:len(value) - 1])
"#,
    );

    assert_eq!(
        generated
            .matches("value.chars().collect::<Vec<char>>()")
            .count(),
        1,
        "{generated}"
    );
    assert!(!generated.contains("value.chars().count()"), "{generated}");
    assert!(generated.contains("__sifr_slice_src.len()"), "{generated}");
}

#[test]
fn aggregate_boundaries_keep_byte_empty_list_and_dict_constructor_types_explicit() {
    let generated = generate_rust_from_source(
        r#"
def build_byte() -> bytes:
    return b"a"

def empty_matches(values: list[int]) -> bool:
    return values == []

def make_dict(items: list[tuple[str, int]]) -> dict[str, int]:
    return dict(items)
"#,
    );

    assert!(
        !generated.contains("SifrInt::from_i64(97) as u8"),
        "{generated}"
    );
    assert!(generated.contains("vec![97u8]"), "{generated}");
    assert!(generated.contains("Vec::<SifrInt>::new()"), "{generated}");
    assert!(
        generated.contains("collect::<std::collections::HashMap<_, _>>()"),
        "{generated}"
    );
}

#[test]
fn keyed_sort_computes_each_key_once_and_descends_without_reversing() {
    let generated = generate_rust_from_source(
        r#"
def rank(value: int) -> int:
    return value % 10

def order(values: list[int], descending: bool) -> list[int]:
    return sorted(values, key=rank, reverse=descending)
"#,
    );

    assert!(generated.contains("__sifr_sorted_pairs"), "{generated}");
    assert!(generated.contains(".into_iter().map"), "{generated}");
    assert!(
        generated.contains("__sifr_sorted_pairs.sort_by"),
        "{generated}"
    );
    assert_eq!(generated.matches("rank(").count(), 2, "{generated}");
    assert!(
        !generated.contains("__sifr_sorted_values.reverse()"),
        "{generated}"
    );
    assert!(
        !generated.contains("__sifr_sorted_pairs.reverse()"),
        "{generated}"
    );
}

#[test]
fn float_sort_uses_python_partial_order_for_both_directions() {
    let generated = generate_rust_from_source(
        r#"
def order(values: list[float], descending: bool) -> list[float]:
    return sorted(values, reverse=descending)
"#,
    );

    assert!(
        generated.contains("partial_cmp") && generated.contains("Ordering::Equal"),
        "{generated}"
    );
    assert!(generated.contains("__sifr_sorted_reverse"), "{generated}");
    assert!(
        generated.contains(".partial_cmp(__sifr_sorted_right)"),
        "{generated}"
    );
    assert!(
        !generated.contains(".partial_cmp(&__sifr_sorted_right)"),
        "{generated}"
    );
    assert!(!generated.contains("total_cmp"), "{generated}");
    assert!(
        !generated.contains("__sifr_sorted_values.reverse()"),
        "{generated}"
    );

    let in_place = methods::lower_method(
        &Type::List(Box::new(Type::Float)),
        "sort",
        &RustExpr::Ident("values".to_string()),
        &[RustExpr::Ident("descending".to_string())],
    )
    .expect("float list.sort lowers");
    let in_place = render_expr(&in_place.expr);
    assert!(in_place.contains("partial_cmp"), "{in_place}");
    assert!(in_place.contains("Ordering::Equal"), "{in_place}");
    assert!(!in_place.contains("total_cmp"), "{in_place}");
}

#[test]
fn field_iteration_borrows_container_storage_once() {
    let generated = generate_rust_from_source(
        r#"
class Store:
    values: list[str]

    def __init__(self):
        self.values = []

    def copy_values(self) -> list[str]:
        result: list[str] = []
        for value in self.values:
            result.append(value)
        return result
"#,
    );

    assert!(
        generated.contains("self.values.iter().cloned()"),
        "{generated}"
    );
    assert!(
        !generated.contains("self.values.clone().iter()"),
        "{generated}"
    );
    assert!(
        !generated.contains("self.values.clone().into_iter()"),
        "{generated}"
    );
}

#[test]
fn last_use_moves_apply_to_literal_entry_and_append_boundaries() {
    let generated = generate_rust_from_source(
        r#"
def build(mut output: list[str]) -> int:
    first: str = "first"
    second: str = "second"
    key: str = "key"
    value: str = "value"
    output.append(first)
    pair: list[str] = [second]
    mapping: dict[str, str] = {key: value}
    return len(output) + len(pair) + len(mapping)
"#,
    );

    assert!(generated.contains("output.push(first)"), "{generated}");
    assert!(generated.contains("vec![second]"), "{generated}");
    assert!(generated.contains("insert(key, value)"), "{generated}");
    assert!(
        !generated.contains("output.push((first).clone())"),
        "{generated}"
    );
    assert!(!generated.contains("vec![(second).clone()]"), "{generated}");
}

#[test]
fn reusable_owned_values_still_clone_at_collection_boundaries() {
    let generated = generate_rust_from_source(
        r#"
def build(mut output: list[str], value: str) -> int:
    output.append(value)
    print(value)
    return len(output)
"#,
    );

    assert!(
        generated.contains("output.push(value.to_owned())"),
        "{generated}"
    );
}

#[test]
fn discarded_setdefault_materializes_only_storage_arguments() {
    let generated = generate_rust_from_source(
        r#"
def populate(mut values: dict[str, str]) -> None:
    key: str = "key"
    value: str = "value"
    values.setdefault(key, value)
"#,
    );

    assert!(
        generated.contains("values.entry(key).or_insert(value)"),
        "{generated}"
    );
    assert!(
        !generated.contains("or_insert(value).to_owned()"),
        "{generated}"
    );
    assert!(!generated.contains("key.to_owned()"), "{generated}");
    assert!(!generated.contains("value.to_owned()"), "{generated}");
}

#[test]
fn deque_field_methods_use_constant_time_front_operations_and_direct_reversal() {
    let deque_storage = Type::List(Box::new(Type::Int));
    let object = RustExpr::Ident("self._data".to_string());
    let pop_front =
        methods::lower_method_with_context(&deque_storage, "popleft", &object, &[], true)
            .expect("deque popleft lowers");
    let reverse = methods::lower_method_with_context(&deque_storage, "reverse", &object, &[], true)
        .expect("deque reverse lowers");

    assert_eq!(render_expr(&pop_front.expr), "self._data.pop_front()");
    assert!(
        render_expr(&reverse.expr).contains("self._data.make_contiguous().reverse()"),
        "{}",
        render_expr(&reverse.expr)
    );
}

#[test]
fn while_condition_and_body_share_one_refresh_per_checked_place() {
    let generated = generate_rust_from_source(
        r#"
def increment(mut values: list[int]) -> int:
    if len(values) == 0:
        return -1
    while values[0] < 3:
        values[0] += values[0]
    return values[0]
"#,
    );

    let loop_body = &generated[generated.find("loop {").expect("rewritten loop")..];
    let condition = loop_body.find("if !(").expect("loop condition");
    assert_eq!(
        loop_body[..condition]
            .matches("let Some(__sifr_checked_value_")
            .count(),
        1,
        "{generated}"
    );
}

#[test]
fn checked_string_element_len_counts_unicode_scalars() {
    let generated = generate_rust_from_source(
        r#"
def element_len(rows: list[str], index: int) -> int:
    if index >= 0 and index < len(rows):
        return len(rows[index])
    return 0
"#,
    );

    assert!(generated.contains(".chars().count()"), "{generated}");
    assert!(!generated.contains("SifrInt::from(&"), "{generated}");
    assert!(
        !generated.contains("SifrInt::from(__sifr_checked_value_0.len())"),
        "{generated}"
    );
}

#[test]
fn optional_string_lookup_compares_owned_local_as_a_view() {
    let generated = generate_rust_from_source(
        r#"
def matches(words: list[str]) -> bool:
    target: str = "abc"
    return words[0] == target
"#,
    );

    assert!(generated.contains("Some(target.as_str())"), "{generated}");
    assert!(!generated.contains("Some(target)"), "{generated}");
}

#[test]
fn direct_owned_string_comparison_uses_partial_eq_without_a_redundant_view() {
    let generated = generate_rust_from_source(
        r#"
def matches() -> bool:
    target: str = "abc"
    return target == "abc"
"#,
    );

    assert!(generated.contains("target == \"abc\""), "{generated}");
    assert!(!generated.contains("target.as_str()"), "{generated}");
}

#[test]
fn closure_captures_block_last_use_move_promotion() {
    let generated = generate_rust_from_source(
        r#"
def build(mut output: list[str]) -> int:
    label: str = "x"
    def describe() -> str:
        return label
    output.append(label)
    return len(describe()) + len(output)
"#,
    );

    assert!(
        generated.contains("output.push(label.to_owned())"),
        "{generated}"
    );
    assert_eq!(
        generated.matches("label.to_owned()").count(),
        2,
        "{generated}"
    );
    assert!(
        !generated.contains("let describe = || {\n    label\n}"),
        "{generated}"
    );
    assert!(!generated.contains("output.push(label)"), "{generated}");
}

#[test]
fn nonrecursive_nested_function_keeps_module_constants_out_of_lexical_captures() {
    let generated = generate_rust_from_source(
        r#"
BIG_LIMIT: int = 100000000000000000000

def read_limit() -> int:
    def helper() -> int:
        return BIG_LIMIT + 1
    return helper()
"#,
    );

    assert!(
        generated.contains("&__sifr_const_4249475f4c494d4954() +"),
        "{generated}"
    );
    assert!(!generated.contains("&BIG_LIMIT +"), "{generated}");
}

#[test]
fn async_move_closure_keeps_owned_string_capture_as_owned_storage() {
    let generated = generate_rust_from_source(
        r#"
async def build() -> str:
    prefix: str = "nested:"
    async def nested_echo(own value: str) -> str:
        return prefix + value
    return await nested_echo("value")
"#,
    );

    assert!(
        generated.contains("async move |value: String|"),
        "{generated}"
    );
    assert!(
        generated.contains("__sifr_concat.push_str(prefix.as_str())"),
        "{generated}"
    );
    assert!(
        !generated.contains("__sifr_concat.push_str(prefix)"),
        "{generated}"
    );
}

#[test]
fn owned_mutable_call_arguments_are_mutations_in_the_body_prepass() {
    let rows_ty = Type::List(Box::new(Type::List(Box::new(Type::Int))));
    let call = HirStmt::Expr {
        expr: HirExpr::Call {
            func: "clear_rows".to_string(),
            args: vec![HirExpr::Name {
                name: "rows".to_string(),
                binding_id: None,
                ty: rows_ty.clone(),
            }],
            mutable_arg_places: Vec::new(),
            ty: Type::None,
        },
    };
    let function = HirFunction {
        name: "caller".to_string(),
        params: vec![HirParam {
            name: "rows".to_string(),
            ty: rows_ty.clone(),
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::None,
        body: vec![call],
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };
    let signatures = HashMap::from([(
        "clear_rows".to_string(),
        (vec![(rows_ty, ParamConvention::own_mut())], Type::None),
    )]);

    let (analysis, _) = crate::body_analysis::BodyAnalysis::build(&function, &signatures);

    assert!(analysis.mutated_in(&function.body).contains("rows"));
}

#[test]
fn repeated_local_string_len_in_loop_uses_a_char_cache() {
    let generated = generate_rust_from_source(
        r#"
def width() -> int:
    value: str = "a🦀z"
    index: int = 0
    while index < len(value):
        index += 1
    return index
"#,
    );

    assert!(generated.contains("let __sifr_chars_value"), "{generated}");
    assert!(
        generated.contains("SifrInt::from(__sifr_chars_value.len())"),
        "{generated}"
    );
    assert!(!generated.contains("value.chars().count()"), "{generated}");
}

#[test]
fn slice_temporaries_use_reserved_generated_names() {
    let generated = generate_rust_from_source(
        r#"
def trim(value: str, _slice_len: int) -> str:
    return value[_slice_len:]
"#,
    );

    assert!(generated.contains("let __sifr_slice_len ="), "{generated}");
    assert!(
        generated.contains("_slice_len.clamp_slice_bound"),
        "{generated}"
    );
}

#[test]
fn unproven_exact_integer_cannot_enter_byte_storage_codegen() {
    let value = HirExpr::Name {
        name: "value".to_string(),
        binding_id: None,
        ty: Type::Int,
    };
    assert!(
        crate::helpers::adapt_bytes_element_for_storage(
            &value,
            RustExpr::Ident("value".to_string()),
        )
        .is_none()
    );
}
