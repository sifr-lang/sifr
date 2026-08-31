use super::generate_rust_from_source;

#[test]
fn shared_string_and_sequence_parameters_use_unsized_views() {
    let generated = generate_rust_from_source(
        r#"
def inspect(text: str, values: list[int], data: bytes) -> int:
    print(text)
    return len(values) + len(data)
"#,
    );

    assert!(
        generated.contains("fn inspect(text: &str, values: &[SifrInt], data: &[u8])"),
        "{generated}"
    );
    assert!(!generated.contains("text: &String"), "{generated}");
    assert!(!generated.contains("values: &Vec"), "{generated}");
}

#[test]
fn borrowed_values_materialize_once_at_owned_boundaries() {
    let generated = generate_rust_from_source(
        r#"
def own_text(own value: str) -> str:
    return value

def own_values(own values: list[int]) -> list[int]:
    return values

def forward_text(value: str) -> str:
    return own_text(value)

def forward_values(values: list[int]) -> list[int]:
    return own_values(values)
"#,
    );

    assert!(
        generated.contains("own_text(value.to_owned())"),
        "{generated}"
    );
    assert!(
        generated.contains("own_values(values.to_vec())"),
        "{generated}"
    );
    assert!(
        !generated.contains("(value.clone()).clone()"),
        "{generated}"
    );
    assert!(
        !generated.contains("(values.clone()).clone()"),
        "{generated}"
    );
}

#[test]
fn explicit_clone_and_copy_materialize_borrowed_views() {
    let generated = generate_rust_from_source(
        r#"
def copy_text(text: str) -> str:
    return text.clone()

def copy_values[T](values: list[T]) -> list[T]:
    copied: list[T] = values.copy()
    return copied
"#,
    );

    assert!(generated.contains("text.to_owned()"), "{generated}");
    assert!(
        generated.contains("let copied: Vec<T> = values.to_vec()"),
        "{generated}"
    );
    assert!(!generated.contains("text.clone()"), "{generated}");
    assert!(
        !generated.contains("let copied: Vec<T> = values.clone()"),
        "{generated}"
    );
}

#[test]
fn addable_uses_one_ownership_correct_trait_for_numbers_and_strings() {
    let generated = generate_rust_from_source(
        r#"
def add_same[T: Addable](left: T, right: T) -> T:
    return left + right

def main():
    assert add_same(2, 3) == 5
    assert add_same("sifr", " rust") == "sifr rust"
"#,
    );

    assert!(generated.contains("trait __SifrAdd"), "{generated}");
    assert!(
        generated.contains("impl __SifrAdd for String"),
        "{generated}"
    );
    assert!(
        generated.contains("impl __SifrAdd for ::sifr_runtime::SifrInt"),
        "{generated}"
    );
    assert!(generated.contains("impl __SifrAdd for f64"), "{generated}");
    assert!(
        generated.contains("T: Clone + 'static + __SifrAdd"),
        "{generated}"
    );
    assert!(
        generated.contains("__SifrAdd::__sifr_add(left.clone(), right.clone())"),
        "{generated}"
    );
    assert!(!generated.contains("Add<Output = T>"), "{generated}");
}

#[test]
fn generic_class_addable_methods_emit_their_support_contract() {
    let generated = generate_rust_from_source(
        r#"
class Accumulator[T: Addable]:
    total: T

    def __init__(self, total: T):
        self.total = total

    def add(self, extra: T) -> T:
        return self.total + extra
"#,
    );

    assert!(generated.contains("trait __SifrAdd"), "{generated}");
    assert!(generated.contains("T: Clone + __SifrAdd"), "{generated}");
    assert!(generated.contains("__SifrAdd::__sifr_add"), "{generated}");
}

#[test]
fn collection_owned_boundaries_materialize_borrowed_string_views() {
    let generated = generate_rust_from_source(
        r#"
def store(mut items: list[str], mut keys: set[str], mut defaults: dict[str, str], value: str) -> None:
    items.append(value)
    items.insert(0, value)
    keys.add(value)
    defaults.setdefault(value, value)
"#,
    );

    assert!(
        generated.contains("items.push(value.to_owned())"),
        "{generated}"
    );
    assert!(generated.contains("items.insert("), "{generated}");
    assert!(
        generated.contains("keys.insert(value.to_owned())"),
        "{generated}"
    );
    assert!(
        generated.matches("value.to_owned()").count() == 5,
        "every owned key/value boundary must materialize its own value:\n{generated}"
    );
    assert!(!generated.contains("value.clone()"), "{generated}");
}

#[test]
fn setdefault_materializes_places_but_moves_owned_temporaries() {
    let generated = generate_rust_from_source(
        r#"
def literal_default(mut defaults: dict[str, str]) -> str:
    return defaults.setdefault("key", "value")

def reusable_default(mut defaults: dict[str, str]) -> str:
    key: str = "key"
    value: str = "value"
    selected: str = defaults.setdefault(key, value)
    print(key)
    print(value)
    return selected
"#,
    );

    assert!(
        generated.contains(
            "defaults.entry(\"key\".to_string()).or_insert(\"value\".to_string()).to_owned()"
        ),
        "{generated}"
    );
    assert!(
        generated.contains("defaults.entry(key.to_owned()).or_insert(value.to_owned()).to_owned()"),
        "{generated}"
    );
    assert!(
        !generated.contains(".to_string().to_owned()"),
        "{generated}"
    );
    assert!(!generated.contains(".to_owned().to_owned()"), "{generated}");
}

#[test]
fn setdefault_copy_values_are_moved_and_dereferenced_without_clone() {
    let generated = generate_rust_from_source(
        r#"
def copy_default(mut flags: dict[str, bool], key: str, flag: bool) -> bool:
    return flags.setdefault(key, flag)
"#,
    );

    assert!(
        generated.contains("*flags.entry(key.to_owned()).or_insert(flag)"),
        "{generated}"
    );
    assert!(!generated.contains("flag.clone()"), "{generated}");
    assert!(
        !generated.contains("or_insert(flag).clone()"),
        "{generated}"
    );
}

#[test]
fn recursive_and_dynamic_programming_shapes_have_clone_budgets() {
    let generated = generate_rust_from_source(
        r#"
def prefix_sums(values: list[int]) -> list[int]:
    result: list[int] = []
    total: int = 0
    for value in values:
        total = total + value
        result.append(total)
    return result
"#,
    );

    assert!(
        generated.contains("fn prefix_sums(values: &[SifrInt]"),
        "{generated}"
    );
    assert!(!generated.contains("values.clone()"), "{generated}");
    assert!(!generated.contains("result.clone()"), "{generated}");
    assert!(
        generated.matches(".clone()").count() <= 6,
        "clone budget exceeded:\n{generated}"
    );
}

#[test]
fn generic_membership_borrows_once_and_declares_equality() {
    let generated = generate_rust_from_source(
        r#"
def contains[T](items: list[T], value: T) -> bool:
    return value in items
"#,
    );

    assert!(
        generated.contains("T: Clone + 'static + PartialEq"),
        "{generated}"
    );
    assert!(generated.contains("items.contains(value)"), "{generated}");
    assert!(!generated.contains("items.contains(&value)"), "{generated}");
}

#[test]
fn recursive_tree_traversal_uses_borrowed_optional_views() {
    let generated = generate_rust_from_source(
        r#"
class TreeNode:
    value: int
    left: TreeNode | None
    right: TreeNode | None

    def __init__(self, value: int, left: TreeNode | None, right: TreeNode | None):
        self.value = value
        self.left = left
        self.right = right

def tree_sum(node: TreeNode | None) -> int:
    if not node:
        return 0
    left: TreeNode | None = node.left
    return node.value + tree_sum(left) + tree_sum(node.right)

def main():
    leaf: TreeNode = TreeNode(2, None, None)
    root: TreeNode = TreeNode(1, leaf, None)
    assert tree_sum(root) == 3
"#,
    );

    assert!(
        generated.contains("fn tree_sum(node: Option<&TreeNode>)"),
        "{generated}"
    );
    assert!(
        generated.contains("let left: Option<&TreeNode> = (node.left).as_deref();"),
        "{generated}"
    );
    assert!(!generated.contains("as_deref().cloned()"), "{generated}");
    assert!(!generated.contains("tree_sum(&Some"), "{generated}");
}

#[test]
fn optional_safe_index_set_removal_projects_the_present_value() {
    let generated = generate_rust_from_source(
        r#"
def remove_at(mut active: set[str], text: str, index: int) -> None:
    active.remove(text[index])
"#,
    );

    assert!(
        generated.contains("if let Some(__sifr_set_value) ="),
        "{generated}"
    );
    assert!(
        generated.contains("active.remove(&__sifr_set_value)"),
        "{generated}"
    );
}

#[test]
fn callable_bounds_share_the_same_unsized_view_abi_as_functions() {
    let generated = generate_rust_from_source(
        r#"
def apply_to_list(f: Callable[[list[int]], int], items: list[int]) -> int:
    return f(items)

def apply_to_str(f: Callable[[str], str], text: str) -> str:
    return f(text)
"#,
    );

    assert!(
        generated.contains("impl Fn(&[SifrInt]) -> SifrInt"),
        "{generated}"
    );
    assert!(generated.contains("impl Fn(&str) -> String"), "{generated}");
    assert!(!generated.contains("Fn(&Vec"), "{generated}");
    assert!(!generated.contains("Fn(&String"), "{generated}");
}

#[test]
fn higher_order_generics_adapt_storage_to_unsized_callable_views() {
    let generated = generate_rust_from_source(
        r#"
def apply[T](predicate: Callable[[T], bool], value: T) -> bool:
    return predicate(value)

def longer_than_one(value: str) -> bool:
    return len(value) > 1

def main():
    assert apply(longer_than_one, "sifr")
"#,
    );

    assert!(
        generated.contains("|__arg0| longer_than_one(__arg0.as_str())"),
        "{generated}"
    );
}

#[test]
fn annotated_parameter_rebinding_materializes_the_initial_borrow() {
    let generated = generate_rust_from_source(
        r#"
def rebind(text: str) -> str:
    print(text)
    text: str = "owned"
    print(text)
    return text.clone()
"#,
    );

    assert!(
        generated.contains("let text = text.to_owned()"),
        "{generated}"
    );
}
