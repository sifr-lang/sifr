use super::generate_rust_from_source;

#[test]
fn corpus_repair_loop_sentinel_reuse() {
    let rust = generate_rust_from_source(
        r#"
def sentinels(limit: int) -> int:
    low = -2147483648
    high = 2147483647
    total = 0
    i = 0
    while i < limit:
        left = low
        right = high
        if i == 1:
            left = high
        other = low
        total += left + right + other
        i += 1
    return total + low + high
"#,
    );
    assert!(rust.matches("low.clone()").count() >= 2, "{rust}");
    assert!(rust.matches("high.clone()").count() >= 2, "{rust}");
    syn::parse_file(&rust).expect("generated sentinel Rust parses");
}

#[test]
fn corpus_repair_loop_sentinel_reuse_nested_control_flow() {
    let rust = generate_rust_from_source(
        r#"
def nested(limit: int) -> int:
    sentinel = 2147483647
    total = 0
    for i in range(limit):
        if i == 1:
            continue
        value = sentinel
        for j in range(2):
            another = sentinel
            total += another
            if j == 1:
                break
        total += value
    return total + sentinel
"#,
    );
    assert!(rust.matches("sentinel.clone()").count() >= 2, "{rust}");
    syn::parse_file(&rust).expect("generated nested-loop Rust parses");
}

#[test]
fn corpus_repair_repeat_count_reuse() {
    let rust = generate_rust_from_source(
        r#"
def repeat(count: int) -> int:
    values = [""] * count
    reverse = count * [1, 2]
    text = "a" * count
    return len(values) + len(reverse) + len(text) + count
"#,
    );
    assert!(rust.contains("count.clone()"), "{rust}");
    assert!(!rust.contains("let __sifr_repeat_n = count;"), "{rust}");
    assert!(!rust.contains("let __n = count;"), "{rust}");
    syn::parse_file(&rust).expect("generated repeat Rust parses");
}

#[test]
fn corpus_repair_repeat_count_reuse_effectful_and_nested() {
    let rust = generate_rust_from_source(
        r#"
class Counter:
    calls: int

    def __init__(self):
        self.calls = 0

    def next(mut self) -> int:
        self.calls += 1
        return 2

    def values(mut self) -> list[int]:
        self.calls += 10
        return [1, 2]

    def text(mut self) -> str:
        self.calls += 10
        return "ab"

def repeat(mut counter: Counter, count: int) -> int:
    first = [1, 2] * counter.next()
    total = len(first)
    for i in range(2):
        values = [0] * count
        total += len(values) + count
    return total + counter.calls + count
"#,
    );
    assert_eq!(rust.matches("counter.next()").count(), 1, "{rust}");
    assert!(!rust.contains("counter.next().clone()"), "{rust}");
    assert!(rust.contains("count.clone()"), "{rust}");
    syn::parse_file(&rust).expect("generated effectful-count Rust parses");

    for expression in [
        "counter.next() * counter.values()",
        "counter.values() * counter.next()",
        "counter.next() * counter.text()",
        "counter.text() * counter.next()",
    ] {
        let source = format!(
            r#"
class Counter:
    calls: int
    def __init__(self):
        self.calls = 0
    def next(mut self) -> int:
        self.calls += 1
        return 2
    def values(mut self) -> list[int]:
        self.calls += 10
        return [1, 2]
    def text(mut self) -> str:
        self.calls += 10
        return "ab"
def repeat(mut counter: Counter) -> int:
    value = {expression}
    return len(value) + counter.calls
"#
        );
        let rust = generate_rust_from_source(&source);
        let count = rust.find("counter.next()").expect("count evaluated");
        let value_call = if expression.contains("values") {
            "counter.values()"
        } else {
            "counter.text()"
        };
        let value = rust.find(value_call).expect("sequence evaluated");
        assert_eq!(
            count < value,
            expression.starts_with("counter.next"),
            "{rust}"
        );
        assert_eq!(rust.matches("counter.next()").count(), 1, "{rust}");
        assert_eq!(rust.matches(value_call).count(), 1, "{rust}");
    }
}
