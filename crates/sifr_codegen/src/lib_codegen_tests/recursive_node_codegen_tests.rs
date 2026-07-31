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
fn test_mutually_recursive_local_binding_is_mutable_for_child_moves() {
    let rust_code = generate_rust_from_source(
        r#"class Branch:
    value: int
    leaf: Leaf | None

    def __init__(self, value: int, leaf: Leaf | None):
        self.value = value
        self.leaf = leaf

class Leaf:
    value: int
    branch: Branch | None

    def __init__(self, value: int, branch: Branch | None):
        self.value = value
        self.branch = branch

def detachLeaf(own branch: Branch) -> Leaf | None:
    local_branch: Branch = branch
    next_leaf: Leaf | None = local_branch.leaf
    return next_leaf
"#,
    );

    assert!(
        rust_code.contains("let mut local_branch: Branch = branch;"),
        "mutually recursive local bindings must use the SCC registry when child reads take boxed fields:\n{rust_code}"
    );
    assert!(
        rust_code.contains("local_branch.leaf.take().map(|__sifr_boxed_recursive_value|"),
        "mutually recursive child reads should take the boxed field:\n{rust_code}"
    );
}

#[test]
fn test_recursive_option_let_else_binding_is_mutable_for_child_moves() {
    let rust_code = generate_rust_from_source(
        r#"class Expr:
    value: int
    term: Term | None

    def __init__(self, value: int, term: Term | None):
        self.value = value
        self.term = term

class Term:
    factor: int
    expr: Expr | None

    def __init__(self, factor: int, expr: Expr | None):
        self.factor = factor
        self.expr = expr

def measure(expr: Expr | None) -> int:
    if not expr:
        return 0
    term: Term | None = expr.term
    if not term:
        return expr.value
    parent: Expr | None = term.expr
    return expr.value + term.factor + measure(parent)
"#,
    );

    assert!(
        rust_code.contains("let Some(mut term) = term else"),
        "recursive option narrowing must bind mutable locals when later field reads use .take():\n{rust_code}"
    );
    assert!(
        rust_code.contains("term.expr.take().map(|__sifr_boxed_recursive_value|"),
        "owned recursive option field reads should still move boxed children:\n{rust_code}"
    );
}

#[test]
fn test_simple_let_else_marks_owned_recursive_option_bindings_mutable() {
    let rust_code = generate_rust_from_source(
        r#"class TreeNode:
    val: int
    left: TreeNode | None
    right: TreeNode | None

    def __init__(self, val: int = 0, left: TreeNode | None = None, right: TreeNode | None = None):
        self.val = val
        self.left = left
        self.right = right

def mergeChildren(own first: TreeNode | None, own second: TreeNode | None) -> TreeNode | None:
    if first is None:
        return second
    if second is None:
        return first
    left: TreeNode | None = mergeChildren(first.left, second.left)
    right: TreeNode | None = mergeChildren(first.right, second.right)
    return TreeNode(first.val + second.val, left, right)
"#,
    );

    assert!(
        rust_code.contains("let Some(mut first) = first else"),
        "simple let-else lowering must make an owned recursive class mutable before taking child fields:\n{rust_code}"
    );
    assert!(
        rust_code.contains("let Some(mut second) = second else"),
        "each owned recursive option narrowed by simple lowering needs a mutable binding:\n{rust_code}"
    );
    assert!(
        rust_code.contains("first.left.take().map(|__sifr_boxed_recursive_value|")
            && rust_code.contains("second.right.take().map(|__sifr_boxed_recursive_value|"),
        "recursive child extraction must continue moving boxed fields without cloning:\n{rust_code}"
    );
}

#[test]
fn test_simple_let_else_keeps_non_recursive_option_binding_immutable() {
    let rust_code = generate_rust_from_source(
        r#"class Value:
    number: int

    def __init__(self, number: int):
        self.number = number

def valueOrZero(own value: Value | None) -> int:
    if value is None:
        return 0
    return value.number
"#,
    );

    assert!(
        rust_code.contains("let Some(value) = value else"),
        "non-recursive optional classes do not require mutable extraction:\n{rust_code}"
    );
    assert!(
        !rust_code.contains("let Some(mut value) = value else"),
        "recursive extraction mutability must not broaden to ordinary optional classes:\n{rust_code}"
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
        rust_code.contains("let Some(node) = node else"),
        "shared recursive options must narrow through an immutable borrowed binding:\n{rust_code}"
    );
    assert!(
        !rust_code.contains("let Some(mut node) = node else"),
        "shared recursive options must not be moved by a mutable pattern:\n{rust_code}"
    );
    assert!(
        rust_code.contains("(node.next).as_deref().cloned()"),
        "borrowed recursive field read must keep cloning semantics:\n{rust_code}"
    );
}

#[test]
fn test_mut_borrowed_recursive_option_binding_stays_immutable() {
    let rust_code = generate_rust_from_source(
        r#"class LinkedNode:
    val: int
    next: LinkedNode | None

    def __init__(self, val: int = 0, next: LinkedNode | None = None):
        self.val = val
        self.next = next

def nextNode(mut node: LinkedNode | None) -> LinkedNode | None:
    if node is None:
        return None
    return node.next
"#,
    );

    assert!(
        rust_code.contains("let Some(node) = node else"),
        "mutable-borrowed recursive options must narrow without moving through the borrow:\n{rust_code}"
    );
    assert!(
        !rust_code.contains("let Some(mut node)"),
        "mutable-borrowed recursive options do not own the narrowed class value:\n{rust_code}"
    );
}

#[test]
fn test_mutually_recursive_option_bindings_use_scc_metadata() {
    let rust_code = generate_rust_from_source(
        r#"class Branch:
    value: int
    leaf: Leaf | None

    def __init__(self, value: int, leaf: Leaf | None):
        self.value = value
        self.leaf = leaf

class Leaf:
    value: int
    branch: Branch | None

    def __init__(self, value: int, branch: Branch | None):
        self.value = value
        self.branch = branch

def detach(own branch: Branch | None, own leaf: Leaf | None) -> int:
    if branch is None:
        return 0
    if leaf is None:
        return branch.value
    next_leaf: Leaf | None = branch.leaf
    next_branch: Branch | None = leaf.branch
    return branch.value + leaf.value
"#,
    );

    assert!(
        rust_code.contains("let Some(mut branch) = branch else")
            && rust_code.contains("let Some(mut leaf) = leaf else"),
        "SCC-recursive owned options must use the same mutable narrowing as self-recursive classes:\n{rust_code}"
    );
    assert!(
        rust_code.contains("branch.leaf.take().map(|__sifr_boxed_recursive_value|")
            && rust_code.contains("leaf.branch.take().map(|__sifr_boxed_recursive_value|"),
        "mutually recursive child fields must move from mutable narrowed bindings:\n{rust_code}"
    );
}

#[test]
fn test_recursive_option_mutability_covers_simple_narrowing_shapes() {
    let rust_code = generate_rust_from_source(
        r#"class LinkedNode:
    val: int
    next: LinkedNode | None

    def __init__(self, val: int = 0, next: LinkedNode | None = None):
        self.val = val
        self.next = next

def fromIfLet(own if_node: LinkedNode | None) -> LinkedNode | None:
    if if_node is not None:
        return if_node.next
    return None

def fromAnd(own and_left: LinkedNode | None, own and_right: LinkedNode | None) -> LinkedNode | None:
    if and_left is not None and and_right is not None:
        right_child: LinkedNode | None = and_right.next
        return and_left.next
    return None

def fromOr(own or_left: LinkedNode | None, own or_right: LinkedNode | None) -> LinkedNode | None:
    if or_left is None or or_right is None:
        return None
    right_child: LinkedNode | None = or_right.next
    return or_left.next

def fromTruthiness(own truthy_node: LinkedNode | None) -> LinkedNode | None:
    if truthy_node:
        return truthy_node.next
    return None

def fromNestedFunction(own outer_node: LinkedNode | None) -> LinkedNode | None:
    def detach(own nested_node: LinkedNode | None) -> LinkedNode | None:
        if nested_node is None:
            return None
        return nested_node.next
    return detach(outer_node)

def fromNestedBlock(own block_node: LinkedNode | None, enabled: bool) -> LinkedNode | None:
    if enabled:
        if block_node is None:
            return None
        return block_node.next
    return None
"#,
    );

    assert!(
        rust_code.contains("if let Some(mut if_node) = if_node"),
        "if-let recursive narrowing must preserve owned mutability:\n{rust_code}"
    );
    assert!(
        rust_code.contains("if let Some(mut and_left) = and_left")
            && rust_code.contains("if let Some(mut and_right) = and_right"),
        "and-chain recursive narrowing must preserve each owned binding:\n{rust_code}"
    );
    assert!(
        rust_code.contains("let (Some(mut or_left), Some(mut or_right))"),
        "or-tuple let-else narrowing must preserve each owned binding:\n{rust_code}"
    );
    assert!(
        rust_code.contains("if let Some(mut truthy_node) = truthy_node"),
        "truthiness narrowing must preserve owned recursive mutability:\n{rust_code}"
    );
    assert!(
        rust_code.contains("let Some(mut nested_node) = nested_node else"),
        "nested-function lowering must retain recursive field metadata:\n{rust_code}"
    );
    assert!(
        rust_code.contains("let Some(mut block_node) = block_node else"),
        "nested simple blocks must retain the complete binding context:\n{rust_code}"
    );
}

#[test]
fn test_nested_copy_parameter_is_not_registered_as_borrowed() {
    let rust_code = generate_rust_from_source(
        r#"def outer(value: int) -> int:
    def addOne(copy_value: int) -> int:
        return copy_value + 1
    return addOne(value)
"#,
    );

    assert!(
        rust_code.contains("|copy_value: i64|"),
        "copy-valued nested parameters must remain direct values:\n{rust_code}"
    );
    assert!(
        !rust_code.contains("|copy_value: &i64|"),
        "default borrow syntax must not classify Copy nested parameters as borrowed storage:\n{rust_code}"
    );
}
