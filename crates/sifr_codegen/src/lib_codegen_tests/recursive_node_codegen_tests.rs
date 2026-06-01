use super::*;

#[test]
fn test_owned_recursive_option_field_moves_without_tail_clone() {
    let rust_code = generate_rust_from_source(
        r#"class LinkedNode:
    val: int
    next: LinkedNode | None

    def __init__(self, val: int = 0, next: LinkedNode | None = None):
        self.val = val
        self.next = next

def moveNextInto(own mut cur: LinkedNode | None, own prev: LinkedNode | None) -> LinkedNode | None:
    if cur is None:
        return prev
    next_node: LinkedNode | None = cur.next
    cur.next = prev
    return moveNextInto(next_node, cur)
"#,
    );

    assert!(
        rust_code.contains("cur.next.take().map(|__sifr_boxed_recursive_value|"),
        "owned recursive field read should take and move the boxed child instead of cloning it:\n{rust_code}"
    );
    assert!(
        !rust_code.contains("(cur.next).as_deref().cloned()"),
        "owned recursive field read should not clone the remaining list tail:\n{rust_code}"
    );
    assert!(
        !rust_code.contains("Some((cur).clone())"),
        "owned optional parameter wrapping should move cur instead of cloning it:\n{rust_code}"
    );
}

#[test]
fn test_owned_recursive_option_field_take_preserves_parent_use() {
    let rust_code = generate_rust_from_source(
        r#"class LinkedNode:
    val: int
    next: LinkedNode | None

    def __init__(self, val: int = 0, next: LinkedNode | None = None):
        self.val = val
        self.next = next

def detachNextOrKeepParent(own mut head: LinkedNode | None) -> LinkedNode | None:
    if head is None:
        return None
    child: LinkedNode | None = head.next
    if child is None:
        return head
    rest: LinkedNode | None = child.next
    head.next = rest
    return child
"#,
    );

    assert!(
        rust_code.contains("head.next.take().map(|__sifr_boxed_recursive_value|"),
        "owned recursive field read should leave the parent usable after an empty child:\n{rust_code}"
    );
    assert!(
        rust_code.contains("return Some(head);"),
        "regression should keep returning the parent after taking an empty child:\n{rust_code}"
    );
    assert!(
        !rust_code.contains("(head.next).map(|__sifr_boxed_recursive_value|"),
        "owned recursive field read must not partially move the parent field:\n{rust_code}"
    );
}

#[test]
fn test_local_recursive_node_binding_is_mutable_for_child_moves() {
    let rust_code = generate_rust_from_source(
        r#"class TreeNode:
    val: int
    left: TreeNode | None
    right: TreeNode | None

    def __init__(self, val: int = 0, left: TreeNode | None = None, right: TreeNode | None = None):
        self.val = val
        self.left = left
        self.right = right

def walk(own root: TreeNode | None) -> list[int]:
    if root is None:
        return []
    q: list[TreeNode] = []
    q.append(root)
    while q:
        node = q.pop(0)
        left_child: TreeNode | None = node.left
    return []
"#,
    );

    assert!(
        rust_code.contains("let mut node: TreeNode ="),
        "local recursive class binding must be mutable so child reads can use .take():\n{rust_code}"
    );
    assert!(
        rust_code.contains("node.left.take().map(|__sifr_boxed_recursive_value|"),
        "owned recursive tree field read should take the child without cloning:\n{rust_code}"
    );
}

#[test]
fn test_borrowed_optional_wrapper_clones_recursive_node() {
    let rust_code = generate_rust_from_source(
        r#"class TreeNode:
    val: int
    left: TreeNode | None
    right: TreeNode | None

    def __init__(self, val: int = 0, left: TreeNode | None = None, right: TreeNode | None = None):
        self.val = val
        self.left = left
        self.right = right

def value_or_zero(node: TreeNode | None) -> int:
    if node is None:
        return 0
    return node.val

def read_then_store(own root: TreeNode | None) -> int:
    if root is None:
        return 0
    root_node: TreeNode = root
    answer: int = value_or_zero(root_node)
    q: list[TreeNode] = [root_node]
    return answer
"#,
    );

    assert!(
        rust_code.contains("value_or_zero(&Some((root_node).clone()))")
            || rust_code.contains("value_or_zero(&Some(root_node.clone()))"),
        "borrowed optional helper call should not move a local recursive node:\n{rust_code}"
    );
    assert!(
        !rust_code.contains("value_or_zero(&Some(root_node));"),
        "borrowed optional wrapper must clone before the local is reused:\n{rust_code}"
    );
}

#[test]
fn test_borrowed_recursive_option_field_still_clones() {
    let rust_code = generate_rust_from_source(
        r#"class LinkedNode:
    val: int
    next: LinkedNode | None

    def __init__(self, val: int = 0, next: LinkedNode | None = None):
        self.val = val
        self.next = next

def nextNode(node: LinkedNode | None) -> LinkedNode | None:
    if node is None:
        return None
    return node.next
"#,
    );

    assert!(
        rust_code.contains("(node.next).as_deref().cloned()"),
        "borrowed recursive field read must keep cloning semantics:\n{rust_code}"
    );
}
