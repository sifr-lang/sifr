use super::generate_rust_from_source;

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
        "counter.next() * [counter.element()]",
        "[counter.element()] * counter.next()",
        "counter.next() * counter.data()",
        "counter.data() * counter.next()",
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
    def element(mut self) -> int:
        self.calls += 10
        return 7
    def data(mut self) -> bytes:
        self.calls += 10
        return b"ab"
def repeat(mut counter: Counter) -> int:
    value = {expression}
    return len(value) + counter.calls
"#
        );
        let rust = generate_rust_from_source(&source);
        let count = rust.find("counter.next()").expect("count evaluated");
        let value_call = if expression.contains("values") {
            "counter.values()"
        } else if expression.contains("element") {
            "counter.element()"
        } else if expression.contains("data") {
            "counter.data()"
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

#[test]
fn corpus_repair_repeat_count_reuse_exact_counts_and_general_sequences() {
    // An exact negative count beyond the host width must remain negative.
    // Positive large counts are inspected without attempting huge allocations.
    for count in ["0", "-1", "-18446744073709551616", "18446744073709551616"] {
        for expression in [format!("[0] * ({count})"), format!("({count}) * [0]")] {
            let rust = generate_rust_from_source(&format!(
                "def repeat() -> list[int]:\n    return {expression}\n"
            ));
            let body = rust.split("fn repeat(").nth(1).expect("repeat function");
            assert!(
                body.contains("std::iter::repeat(SifrInt::from_i64(0))"),
                "{body}"
            );
            assert!(body.contains("SifrRange::new_known_nonzero("), "{body}");
            assert!(body.contains("collect::<Vec<_>>()"), "{body}");
            assert!(
                !body.contains("as usize") && !body.contains(".take("),
                "{body}"
            );
            assert!(!body.contains("__sifr_repeat_out.extend"), "{body}");
            if count.contains("18446744073709551616") {
                assert!(body.contains("18446744073709551616"), "{body}");
            }
        }
    }
    // Variable collections retain the general path even when a caller might
    // supply a singleton. Counts and source sequences remain reusable.
    for (ty, result_ty) in [
        ("list[int]", "list[int]"),
        ("bytes", "bytes"),
        ("str", "str"),
    ] {
        for expression in ["values * count", "count * values"] {
            let source = format!(
                "def repeat(values: {ty}, count: int) -> {result_ty}:\n    first = {expression}\n    second = values * count\n    return first + second\n"
            );
            let rust = generate_rust_from_source(&source);
            let body = rust.split("fn repeat(").nth(1).expect("repeat function");
            assert!(!body.contains("std::iter::repeat("), "{body}");
            assert!(body.contains("count.clone()"), "{body}");
            assert!(body.contains("<= SifrInt::from_i64(0)"), "{body}");
            assert!(
                !body.contains("as usize") && !body.contains(".take("),
                "{body}"
            );
            syn::parse_file(&rust).expect("general sequence repeat parses");
        }
    }
}

#[test]
fn corpus_repair_repeat_count_reuse_statement_occurrences() {
    // A condition's direct count occurrence is not the statement's last use
    // when its child body still needs the same owned exact integer.
    for statement in [
        "if len([0] * count) > 0:\n        return count",
        "for i in range(count):\n        values = [0] * count\n        count += len(values)",
        "while len([0] * count) > 0:\n        count -= 1",
    ] {
        let source =
            format!("def repeat() -> int:\n    count = 3\n    {statement}\n    return 0\n");
        let parsed = sifr_python_parser::parse_module(&source).expect("parse");
        let lowering = sifr_lowering::lower_module(parsed.suite()).expect("lower");
        let function = &lowering.module.functions[0];
        let (_, moves) = crate::body_analysis::BodyAnalysis::build(function, &Default::default());
        let condition = match &function.body[1] {
            sifr_ir::HirStmt::If { condition, .. } | sifr_ir::HirStmt::While { condition, .. } => {
                condition
            }
            sifr_ir::HirStmt::For { iter, .. } => iter,
            other => panic!("expected compound statement: {other:?}"),
        };
        let mut occurrences = 0;
        crate::hir_analysis::traversal::walk_expr(condition, &mut |expr| {
            if matches!(expr, sifr_ir::HirExpr::Name { name, .. } if name == "count") {
                occurrences += 1;
                assert!(
                    !moves.contains(&crate::body_analysis::expr_key(expr)),
                    "{source}"
                );
            }
        });
        assert!(occurrences > 0, "must inspect the direct count occurrence");
        syn::parse_file(&crate::generate_rust(&lowering.module)).expect("repeat Rust parses");
    }
}
