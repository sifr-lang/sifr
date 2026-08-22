use super::generate_rust_from_source;

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
