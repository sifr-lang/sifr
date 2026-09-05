use super::generate_rust_from_source;
use quote::ToTokens;
use syn::visit::Visit;

fn canonical(source: &str) -> String {
    let raw = generate_rust_from_source(source);
    assert!(!raw.contains("compile_error!"), "{raw}");
    crate::generated_rust_canonicalizer::canonicalize_generated_rust_source(&raw)
        .expect("generated Rust canonicalizes")
}

fn parameters(rust: &str, target: &str) -> Vec<String> {
    struct Functions<'a> {
        target: &'a str,
        parameters: Vec<String>,
    }
    impl<'ast> Visit<'ast> for Functions<'_> {
        fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
            if function.sig.ident == self.target {
                self.parameters
                    .push(function.sig.inputs.to_token_stream().to_string());
            }
            syn::visit::visit_item_fn(self, function);
        }
    }
    let mut functions = Functions {
        target,
        parameters: Vec::new(),
    };
    functions.visit_file(&syn::parse_file(rust).expect("Rust parses"));
    assert!(!functions.parameters.is_empty(), "missing {target}: {rust}");
    functions.parameters
}

#[test]
fn corpus_repair_exception_capture_handler_is_local() {
    let rust = generate_rust_from_source(
        r#"
def outer(n: int) -> Result[int, Error]:
    def helper(i: int) -> Result[int, Error]:
        try:
            if i == 0:
                return 0
            result: int = helper(i - 1)
            return result + 1
        except Error as error:
            raise error
    try:
        result: int = helper(n)
        return result
    except Error as error:
        raise error
"#,
    );
    assert!(
        parameters(&rust, "helper")
            .iter()
            .all(|parameters| !parameters.contains("error")),
        "{rust}"
    );
}

#[test]
fn corpus_repair_exception_capture_keeps_outer_same_name() {
    let rust = generate_rust_from_source(
        r#"
def outer(error: int, n: int) -> Result[int, Error]:
    def helper(i: int) -> Result[int, Error]:
        value = error
        try:
            if i == 0:
                return value
            result: int = helper(i - 1)
            return result + value
        except Error as error:
            raise error
    try:
        result: int = helper(n)
        return result
    except Error as error:
        raise error
"#,
    );
    assert!(
        parameters(&rust, "helper")
            .iter()
            .all(|parameters| parameters.contains("error")),
        "{rust}"
    );
}

#[test]
fn corpus_repair_checked_read_control_flow_disjunction() {
    let rust = generate_rust_from_source(
        r#"
def total(left: list[int], right: list[int]) -> int:
    result = 0
    i = 0
    while i < len(left) or i < len(right):
        if i < len(left):
            value = left[i]
            if value is None:
                return -1
            result += value
        if i < len(right):
            value = right[i]
            if value is None:
                return -2
            result += value
        i += 1
    return result
"#,
    );
    struct Loops;
    impl<'ast> Visit<'ast> for Loops {
        fn visit_expr_while(&mut self, loop_: &'ast syn::ExprWhile) {
            assert!(
                !matches!(loop_.body.stmts.first(), Some(syn::Stmt::Local(local)) if local.init.as_ref().is_some_and(|init| init.diverge.is_some())),
                "loop hoisted a branch-only read: {}",
                loop_.to_token_stream()
            );
            syn::visit::visit_expr_while(self, loop_);
        }
    }
    Loops.visit_file(&syn::parse_file(&rust).expect("Rust parses"));
    assert!(!rust.contains("== None"), "{rust}");
}

#[test]
fn corpus_repair_checked_read_control_flow_none_operand_effects() {
    let rust = generate_rust_from_source(
        r#"
def next_value(mut events: list[int]) -> int:
    events.append(1)
    return 3
def probe(mut events: list[int]) -> bool:
    return next_value(events) is None
"#,
    );
    assert!(
        rust.split("fn probe")
            .nth(1)
            .is_some_and(|body| body.contains("next_value(")),
        "{rust}"
    );
    assert!(!rust.contains("== None"), "{rust}");
    assert!(!rust.contains("compile_error!"), "{rust}");
}

#[test]
fn corpus_repair_repeated_value_ownership_comprehension_bindings() {
    let rust = generate_rust_from_source(
        r#"
def repeat(seed: int, n: int) -> list[list[int]]:
    return [[seed for j in range(n)] for i in range(n)]
def fresh(n: int) -> list[int]:
    return [i for i in range(n)]
def outer_binding(n: int) -> list[int]:
    return [i for i in range(n) for j in range(n)]
"#,
    );
    assert!(rust.contains("seed.clone()"), "{rust}");
    assert!(rust.contains("push(i.clone())"), "{rust}");
    assert!(rust.contains("push(i)"), "{rust}");
}

#[test]
fn corpus_repair_repeated_value_ownership_nested_indices_and_option_comparison() {
    let rust = generate_rust_from_source(
        r#"
def lookup(values: list[list[int]], i: int, j: int) -> int:
    first: int | None = values[i][j]
    second: int | None = values[i][j]
    if first is not None:
        if first == second:
            return i + j + first
    return -1
"#,
    );
    assert!(rust.matches("i.clone()").count() >= 2, "{rust}");
    assert!(rust.matches("j.clone()").count() >= 2, "{rust}");
    assert!(rust.contains("Some(first.clone()) == second"), "{rust}");
    assert!(!rust.contains("if Some(first) =="), "{rust}");
    syn::parse_file(&rust).expect("Rust parses");
}

#[test]
fn corpus_repair_recursive_optional_mutability_preserves_take() {
    let rust = canonical(
        r#"
class Node:
    value: int
    next: Node | None
    def __init__(self, value: int, next: Node | None):
        self.value = value
        self.next = next
def advance(own node: Node | None) -> Node | None:
    if node is None:
        return None
    return node.next
def main():
    node = Node(1, Node(2, None))
    following = advance(node)
    assert following is not None
"#,
    );
    assert!(rust.contains(".take()"), "{rust}");
    assert!(
        rust.contains("let mut node") || rust.contains("Some(mut node)"),
        "{rust}"
    );
}

#[test]
fn corpus_repair_structured_exception_nested_return_and_field_update() {
    let rust = generate_rust_from_source(
        r#"
class Counter:
    count: int
    def __init__(self):
        self.count = 0
    def increment(mut self) -> Result[None, Error]:
        try:
            self.count += 1
            return None
        except Error as error:
            raise error
def nested() -> Result[int, Error]:
    try:
        values = [[1]]
        values[0][0] = 2
        result: int | None = values[0][0]
        if result is None:
            raise IndexError("missing result")
        return result
    except Error as error:
        raise error
"#,
    );
    assert!(!rust.contains("compile_error!"), "{rust}");
    assert!(rust.contains("IndexError"), "{rust}");
    syn::parse_file(&rust).expect("Rust parses");
}

#[test]
fn corpus_repair_method_retention_escaped_keyword_identity() {
    let rust = canonical(
        r#"
class Group:
    count: int
    def __init__(self):
        self.count = 1
    def union(mut self, value: int) -> int:
        self.count += value
        return self.count
def main():
    group = Group()
    assert group.union(2) == 3
"#,
    );
    assert!(rust.contains("fn r#union("), "{rust}");
    assert!(rust.contains(".r#union("), "{rust}");
    assert!(rust.contains("let mut group"), "{rust}");
}

#[test]
fn corpus_repair_checked_read_control_flow_short_circuit_assignment() {
    let rust = canonical(
        r#"
def probe() -> Result[bool, Error]:
    try:
        matrix: list[list[bool]] = [[True]]
        for i in range(1):
            matrix[i][i] = matrix[i][i] or (i + 1 < len(matrix) and matrix[i + 1][i])
        result: bool | None = matrix[0][0]
        if result is None:
            raise IndexError("missing result")
        return result
    except Error as error:
        raise error
"#,
    );
    assert!(rust.contains("||"), "{rust}");
    assert!(!rust.contains("break;"), "{rust}");
    assert!(rust.contains("IndexError::new"), "{rust}");
}

#[test]
fn corpus_repair_structured_exception_root_error_and_dictionary_reads() {
    let rust = canonical(
        r#"
def read(values: list[list[int]]) -> Result[int, Error]:
    try:
        size = len(values)
        if size == 0:
            return 0
        row = values[0]
        if row is None:
            return 0
        memo: dict[int, int] = {}
        memo[0] = len(row)
        cached = memo[0]
        if cached is None:
            raise KeyError("missing cache")
        return cached
    except Error as error:
        raise error
"#,
    );
    assert!(rust.contains("IndexError::new"), "{rust}");
    assert!(rust.contains("KeyError::new"), "{rust}");
}

#[test]
fn corpus_repair_structured_exception_nested_while_checked_comparison() {
    let rust = canonical(
        r#"
def count(values: list[int]) -> Result[int, Error]:
    try:
        left, right = 0, 0
        while right < len(values):
            while right + 1 < len(values) and values[right] == values[right + 1]:
                right += 1
            left += 1
            right += 1
        return left
    except Error as error:
        raise error
"#,
    );
    assert!(rust.contains("while"), "{rust}");
    assert!(rust.contains("&&"), "{rust}");
}

#[test]
fn corpus_repair_repeated_value_ownership_condition_and_branch() {
    let rust = generate_rust_from_source(
        r#"
def child(edges: list[dict[str, int]], node: int, key: str) -> int:
    if key in edges[node]:
        value: int | None = edges[node][key]
        if value is not None:
            return value
    return -1
"#,
    );
    assert_eq!(
        rust.matches("let __idx_raw = node.clone();").count(),
        1,
        "{rust}"
    );
}

#[test]
fn corpus_repair_repeated_value_ownership_nested_arithmetic_and_defaults() {
    let rust = generate_rust_from_source(
        r#"
def consume(value: int | None) -> int:
    return 0
def probe(a: int, b: int, values: dict[str, int]) -> int:
    first = max(a + b, 0)
    flipped = ~a
    defaulted = max(first, values.get("missing", a))
    used = consume(a)
    return first + flipped + defaulted + used + a + b
"#,
    );
    assert!(!rust.contains("!a"), "{rust}");
    assert!(!rust.contains("a + b"), "{rust}");
    assert!(!rust.contains("unwrap_or(a)"), "{rust}");
    assert!(!rust.contains("Some(a)"), "{rust}");
}

#[test]
fn corpus_repair_empty_collection_assertion_in_exception_carrier() {
    let rust = canonical(
        r#"
def values() -> Result[list[int], Error]:
    return []
def main():
    try:
        result: list[int] = values()
        assert result == []
        assert [] == result
    except Error:
        assert False
"#,
    );
    assert!(!rust.contains("assert_eq!(result, Vec::new())"), "{rust}");
    assert!(!rust.contains("assert_eq!(Vec::new(), result)"), "{rust}");
}

#[test]
fn corpus_repair_proven_read_at_optional_call_boundary() {
    let rust = canonical(
        r#"
def accept(value: int | None) -> int:
    if value is None:
        return -1
    return value
def total(queue: list[int]) -> Result[int, Error]:
    try:
        head = 0
        result = 0
        while head < len(queue):
            result += accept(queue[head])
            head += 1
        return result
    except Error as error:
        raise error
"#,
    );
    assert!(rust.contains("IndexError::new"), "{rust}");
    assert!(!rust.contains("Some(queue.get"), "{rust}");
    assert!(!rust.contains(".unwrap()"), "{rust}");
    assert!(!rust.contains(".expect("), "{rust}");
}

#[test]
fn corpus_repair_explicit_optional_nested_mutation_contract() {
    let rust = canonical(
        r#"
def update(mut grid: list[list[int]], pos: list[int]) -> Result[None, Error]:
    try:
        row = pos[0]
        column = pos[1]
        if row is None or column is None:
            raise IndexError("position is missing")
        grid[row][column] = 7
        return None
    except Error as error:
        raise error
"#,
    );
    assert!(rust.contains("get_mut"), "{rust}");
    assert!(rust.contains("position is missing"), "{rust}");
    assert!(!rust.contains(".unwrap()"), "{rust}");
    assert!(!rust.contains(".expect("), "{rust}");
}

#[test]
fn corpus_repair_nested_arithmetic_preserves_borrowed_parameter() {
    let rust = canonical(
        r#"
class Cursor:
    position: int
    def __init__(self):
        self.position = 0
    def advance(mut self, steps: int) -> int:
        self.position = min(self.position + steps, 100)
        self.position = max(self.position - steps, 0)
        return self.position + steps
"#,
    );
    assert!(rust.contains("steps: &SifrInt"), "{rust}");
    assert!(!rust.contains("&steps"), "{rust}");
    assert!(rust.contains("+ steps"), "{rust}");
    assert!(rust.contains("- steps"), "{rust}");
}

#[test]
fn corpus_repair_empty_collection_assertion_has_element_type() {
    let rust = generate_rust_from_source(
        r#"
def main():
    values: list[int] = []
    assert values == []
    assert [] == values
"#,
    );
    assert!(rust.matches("Vec::<SifrInt>::new()").count() >= 2, "{rust}");
    syn::parse_file(&rust).expect("Rust parses");
}

#[test]
fn corpus_repair_structured_exception_proven_nested_read_uses_typed_carrier() {
    use crate::{HirExpr, RustEmitter, Type};
    let error = Type::Class {
        identity: None,
        type_args: vec![],
        name: "IndexError".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: vec![],
        parent_class: None,
    };
    let mut emitter = RustEmitter::new();
    let row = HirExpr::Index {
        object: Box::new(HirExpr::Name {
            name: "matrix".to_string(),
            binding_id: Some(sifr_ir::BindingId(1)),
            ty: Type::List(Box::new(Type::List(Box::new(Type::Int)))),
        }),
        index: Box::new(HirExpr::IntLiteral(0)),
        ty: Type::List(Box::new(Type::Int)),
    };
    assert!(
        emitter
            .lower_proven_read_with_error_carrier(&row, &HirExpr::IntLiteral(0))
            .expect("lowering succeeds")
            .is_none()
    );
    emitter.current_return_type = Some(Type::Result(Box::new(Type::Int), Box::new(error.clone())));
    let value = emitter
        .lower_proven_read_with_error_carrier(&row, &HirExpr::IntLiteral(0))
        .expect("lowering succeeds")
        .expect("typed carrier supports the read");
    let rust = crate::render_stmts(&[crate::RustStmt::Expr(value)]);
    assert_eq!(rust.matches("IndexError::new").count(), 2, "{rust}");
    assert!(!rust.contains("unwrap"), "{rust}");
    let Type::Class {
        mut identity,
        type_args,
        name,
        fields,
        methods,
        parent_class,
    } = error
    else {
        unreachable!()
    };
    identity.replace("user.IndexError".to_string());
    emitter.current_return_type = Some(Type::Result(
        Box::new(Type::Int),
        Box::new(Type::Class {
            identity,
            type_args,
            name,
            fields,
            methods,
            parent_class,
        }),
    ));
    assert!(
        emitter
            .lower_proven_read_with_error_carrier(&row, &HirExpr::IntLiteral(0))
            .expect("lowering succeeds")
            .is_none()
    );
}
