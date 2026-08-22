use super::generate_rust_from_source;
use crate::generate_rust;
use sifr_ir::{HirExpr, HirStmt};
use sifr_lowering::lower_module;
use sifr_python_parser::parse_module;
use sifr_type_system::Type;

const PROBE_ERROR: &str = r#"
class ProbeError(Error):
    message: str

def load_int(value: int) -> Result[int, ProbeError]:
    if value < 0:
        raise ProbeError("negative")
    return value
"#;

#[test]
fn successful_sequential_try_bindings_escape_generated_rust_closures() {
    let generated = generate_rust_from_source(&format!(
        r#"{PROBE_ERROR}
def collect_pair() -> Result[int, ProbeError]:
    try:
        first: int = load_int(20)
    except ProbeError as error:
        raise error
    try:
        second: int = load_int(22)
    except ProbeError as error:
        raise error
    return first + second
"#
    ));

    assert!(generated.contains("let (first,) = match __sifr_try_res"));
    assert!(generated.contains("let (second,) = match __sifr_try_res"));
    assert!(!generated.contains(".unwrap()"));
    assert!(!generated.contains(".expect("));
    syn::parse_file(&generated).expect("sequential try binding Rust should parse");
}

#[test]
fn reassigned_successful_try_binding_is_mutable_after_promotion() {
    let generated = generate_rust_from_source(&format!(
        r#"{PROBE_ERROR}
def increment() -> Result[int, ProbeError]:
    try:
        value: int = load_int(41)
    except ProbeError as error:
        raise error
    value = value + 1
    return value
"#
    ));

    assert!(generated.contains("let (mut value,) = match __sifr_try_res"));
    syn::parse_file(&generated).expect("mutable promoted binding Rust should parse");
}

#[test]
fn handler_return_does_not_hide_successful_try_binding() {
    let generated = generate_rust_from_source(&format!(
        r#"{PROBE_ERROR}
def value_or_zero() -> int:
    value: int = 1
    try:
        value: int = load_int(41)
    except ProbeError:
        return 0
    return value
"#
    ));

    assert!(generated.contains("let (value,) = match __sifr_try_res"));
    assert!(generated.contains("return 0_i64;"));
    syn::parse_file(&generated).expect("handler return binding Rust should parse");
}

#[test]
fn nested_try_finally_uses_enclosing_try_error_channel_without_return_capture() {
    let generated = generate_rust_from_source(&format!(
        r#"{PROBE_ERROR}
def run() -> None:
    try:
        try:
            _value: int = load_int(-1)
        finally:
            marker: int = 1
    except ProbeError:
        marker = 2
"#
    ));

    assert!(generated.contains("return Err(__sifr_finally_err.into());"));
    assert!(!generated.contains("sifr try/finally error propagation in non-Result function"));
    syn::parse_file(&generated).expect("nested try/finally Rust should parse");
}

#[test]
fn try_body_return_and_fallthrough_binding_use_distinct_carriers() {
    let generated = generate_rust_from_source(&format!(
        r#"{PROBE_ERROR}
def conditional(flag: bool) -> Result[int, ProbeError]:
    try:
        value: int = load_int(41)
        if flag:
            return value
    except ProbeError as error:
        raise error
    return value + 1
"#
    ));

    assert!(generated.contains("let mut __sifr_successful_try_bindings: Option<(i64,)> = None"));
    assert!(generated.contains("let Some((value,)) = __sifr_successful_try_bindings else"));
    assert!(!generated.contains(".unwrap()"));
    assert!(!generated.contains(".expect("));
    syn::parse_file(&generated).expect("combined try carriers should parse");
}

#[test]
fn ioerror_subclass_handlers_remain_guarded_before_base_handler() {
    let generated = generate_rust_from_source(
        r#"
def load_io() -> Result[str, IOError]:
    raise IOError("missing")

def length_or_error(flag: bool) -> Result[int, IOError]:
    try:
        content: str = load_io()
        if flag:
            return len(content)
    except FileNotFoundError as error:
        raise error
    except PermissionError as error:
        raise error
    except FileExistsError as error:
        raise error
    except IsADirectoryError as error:
        raise error
    except NotADirectoryError as error:
        raise error
    except DirectoryNotEmptyError as error:
        raise error
    except IOError as error:
        raise error
    return len(content)
"#,
    );

    assert!(generated.contains("let Some((content,)) = __sifr_successful_try_bindings else"));
    assert!(generated.contains("\"NotADirectory\".to_string()"));
    assert_eq!(generated.matches("__sifr_try_err.kind ==").count(), 6);
    assert!(!generated.contains(".unwrap()"));
    assert!(!generated.contains(".expect("));
    syn::parse_file(&generated).expect("exhaustive IOError handler Rust should parse");
}

#[test]
fn promoted_string_binding_initializes_enclosing_character_cache() {
    let generated = generate_rust_from_source(
        r#"
class ProbeError(Error):
    message: str

def load_text() -> Result[str, ProbeError]:
    return "value"

def text_length() -> Result[int, ProbeError]:
    try:
        text: str = load_text()
    except ProbeError as error:
        raise error
    return len(text)
"#,
    );

    let promoted = generated
        .find("let (text,) = match __sifr_try_res")
        .expect("promoted text binding should exist");
    let cache = generated
        .find("let __sifr_chars_text: Vec<char> = text.chars().collect")
        .unwrap_or_else(|| panic!("enclosing text cache should exist: {generated}"));
    assert!(cache > promoted, "{generated}");
    assert_eq!(generated.matches("let __sifr_chars_text").count(), 1);
    syn::parse_file(&generated).expect("promoted string binding Rust should parse");
}

#[test]
fn dead_partially_moved_try_binding_is_not_returned_from_closure() {
    let generated = generate_rust_from_source(&format!(
        r#"{PROBE_ERROR}
def load_pair() -> Result[tuple[str, str], ProbeError]:
    return ("left", "right")

def consume(own value: str) -> None:
    _ = value

def parse_pair() -> Result[int, ProbeError]:
    try:
        parsed_pair: tuple[str, str] = load_pair()
        left, right = parsed_pair
        consume(left)
        consume(right)
    except ProbeError as error:
        raise error
    return 1
"#
    ));

    assert!(!generated.contains("Ok((parsed_pair,))"), "{generated}");
    syn::parse_file(&generated).expect("dead try binding Rust should parse");
}

#[test]
fn try_binding_captured_by_following_nested_function_remains_live() {
    let generated = generate_rust_from_source(&format!(
        r#"{PROBE_ERROR}
def outer() -> Result[int, ProbeError]:
    try:
        value: int = load_int(42)
    except ProbeError as error:
        raise error

    def read_value() -> int:
        return value

    return read_value()
"#
    ));

    assert!(generated.contains("let (value,) = match __sifr_try_res"));
    syn::parse_file(&generated).expect("captured live try binding Rust should parse");
}

#[test]
fn nested_function_with_literal_default_uses_structured_codegen() {
    let generated = generate_rust_from_source(
        r#"
def outer() -> int:
    def read_value(candidate: int = 42) -> int:
        return candidate

    return read_value() + read_value(candidate=5)
"#,
    );

    assert!(generated.contains("let read_value = |candidate: i64|"));
    assert!(generated.contains("read_value(42_i64)"));
    assert!(generated.contains("read_value(5_i64)"));
    syn::parse_file(&generated).expect("nested literal default Rust should parse");
}

#[test]
fn post_try_hir_default_reference_keeps_the_binding_live() {
    let source = format!(
        r#"{PROBE_ERROR}
def outer() -> Result[int, ProbeError]:
    try:
        value: int = load_int(42)
    except ProbeError as error:
        raise error

    def read_value(candidate: int = 1) -> int:
        return candidate

    return read_value()
"#
    );
    let parsed = parse_module(&source).expect("parse failed");
    let mut module = lower_module(parsed.suite())
        .expect("lowering failed")
        .module;
    let outer = module
        .functions
        .iter_mut()
        .find(|function| function.name == "outer")
        .expect("outer function should lower");
    let nested = outer
        .body
        .iter_mut()
        .find_map(|stmt| match stmt {
            HirStmt::NestedFunction { func, .. } if func.name == "read_value" => Some(func),
            _ => None,
        })
        .expect("nested function should lower");
    nested.params[0].default = Some(HirExpr::Name {
        name: "value".to_string(),
        binding_id: None,
        ty: Type::Int,
    });

    let generated = generate_rust(&module);

    assert!(generated.contains("let (value,) = match __sifr_try_res"));
    syn::parse_file(&generated).expect("post-try HIR default Rust should parse");
}

#[test]
fn shadowing_nested_parameter_does_not_keep_try_binding_live() {
    let generated = generate_rust_from_source(&format!(
        r#"{PROBE_ERROR}
def outer() -> Result[int, ProbeError]:
    try:
        value: int = load_int(42)
    except ProbeError as error:
        raise error

    def read_value(value: int) -> int:
        return value

    return read_value(7)
"#
    ));

    assert!(!generated.contains("Ok((value,))"));
    assert!(!generated.contains("let value: i64;"));
    syn::parse_file(&generated).expect("shadowing nested parameter Rust should parse");
}

#[test]
fn try_binding_replaced_before_read_keeps_only_its_declaration() {
    let generated = generate_rust_from_source(&format!(
        r#"{PROBE_ERROR}
def replace_value() -> Result[int, ProbeError]:
    try:
        total: int = load_int(3)
    except ProbeError as error:
        raise error
    total = 9
    return 0
"#
    ));

    assert!(generated.contains("let total: i64;"));
    assert!(!generated.contains("Ok((total,))"));
    assert!(!generated.contains("let (mut total,) = match __sifr_try_res"));
    syn::parse_file(&generated).expect("declaration-only try binding Rust should parse");
}

#[test]
fn moved_try_binding_can_be_replaced_without_value_transport() {
    let generated = generate_rust_from_source(&format!(
        r#"{PROBE_ERROR}
def load_text() -> Result[str, ProbeError]:
    return "old"

def consume(own value: str) -> None:
    _ = value

def replace_text() -> Result[str, ProbeError]:
    try:
        text: str = load_text()
        consume(text)
    except ProbeError as error:
        raise error
    text = "new"
    return text
"#
    ));

    assert!(generated.contains("let text: String;"));
    assert!(!generated.contains("Ok((text,))"));
    syn::parse_file(&generated).expect("replaced moved try binding Rust should parse");
}

#[test]
fn declaration_only_try_binding_is_mutable_for_repeated_replacement() {
    let generated = generate_rust_from_source(&format!(
        r#"{PROBE_ERROR}
def replace_twice() -> Result[int, ProbeError]:
    try:
        value: int = load_int(1)
    except ProbeError as error:
        raise error
    value = 2
    value = 3
    return value
"#
    ));

    assert!(generated.contains("let mut value: i64;"));
    assert!(!generated.contains("Ok((value,))"));
    syn::parse_file(&generated).expect("mutable declaration-only try binding Rust should parse");
}

#[test]
fn try_binding_used_only_as_following_subscript_target_remains_live() {
    let generated = generate_rust_from_source(&format!(
        r#"{PROBE_ERROR}
def replace_first() -> Result[int, ProbeError]:
    try:
        data: list[int] = [1]
    except ProbeError as error:
        raise error
    data[0] = 5
    return 0
"#
    ));

    assert!(generated.contains("let (mut data,) = match __sifr_try_res"));
    syn::parse_file(&generated).expect("subscript-live try binding Rust should parse");
}

#[test]
fn always_raising_try_body_uses_total_result_match() {
    let generated = generate_rust_from_source(&format!(
        r#"{PROBE_ERROR}
def classify() -> str:
    try:
        raise ProbeError("bad")
    except ProbeError as error:
        return error.message
"#
    ));

    assert!(generated.contains("match __sifr_try_res"));
    assert!(generated.contains("Ok(()) =>"));
    assert!(generated.contains("sifr try/except raising body returned success"));
    assert!(!generated.contains("if let Err(__sifr_try_err) = __sifr_try_res"));
    syn::parse_file(&generated).expect("total raising try Rust should parse");
}

#[test]
fn unmatched_ioerror_kind_returns_through_the_checked_error_channel() {
    let generated = generate_rust_from_source(
        r#"
def classify() -> Result[str, IOError]:
    try:
        raise IOError("other")
    except FileNotFoundError:
        return "missing"
"#,
    );

    assert!(generated.contains("__sifr_try_err.kind == \"FileNotFound\""));
    assert!(generated.contains("return Err(__sifr_try_err);"));
    syn::parse_file(&generated).expect("residual IOError propagation Rust should parse");
}

#[test]
fn branchless_handler_member_returns_through_the_checked_error_channel() {
    let generated = generate_rust_from_source(
        r#"
def classify() -> Result[str, IOError]:
    try:
        raise IOError("other")
    except ValueError:
        return "value"
"#,
    );

    assert!(generated.contains("Err(__sifr_try_err) => {\n            return Err(__sifr_try_err);"));
    assert!(!generated.contains("structured statement emission missing"));
    syn::parse_file(&generated).expect("branchless residual Rust should parse");
}

#[test]
fn union_member_without_a_handler_keeps_its_residual() {
    let generated = generate_rust_from_source(
        r#"
def classify(flag: bool) -> Result[str, IOError | ValueError]:
    try:
        if flag:
            raise IOError("other")
        raise ValueError("value")
    except FileNotFoundError:
        return "missing"
"#,
    );

    assert!(generated.contains("__sifr_try_variant_error.kind == \"FileNotFound\""));
    assert!(generated.matches("return Err(").count() >= 2);
    assert!(!generated.contains("structured statement emission missing"));
    syn::parse_file(&generated).expect("union residual Rust should parse");
}

#[test]
fn user_error_parent_handler_converts_child_and_preserves_unrelated_residual() {
    let generated = generate_rust_from_source(
        r#"
class BaseError(Error):
    message: str

class ChildError(BaseError):
    def __init__(self, message: str) -> None:
        super().__init__(message)

class OtherError(Error):
    message: str

def classify(flag: bool) -> Result[str, ChildError | OtherError]:
    if flag:
        raise ChildError("child")
    raise OtherError("other")

def handle(flag: bool) -> Result[str, OtherError]:
    try:
        value: str = classify(flag)
        return value
    except BaseError as error:
        return error.message
"#,
    );

    assert!(generated.contains("Into::<BaseError>::into(__sifr_try_variant_error.clone())"));
    assert!(generated.contains("return Err(__sifr_try_variant_error);"));
    syn::parse_file(&generated).expect("user error parent-handler Rust should parse");
}
