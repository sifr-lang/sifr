struct LinkedNode {
    val: i64,
    next: Option<Box<LinkedNode>>,
}

impl LinkedNode {
    fn new(val: i64, next: Option<LinkedNode>) -> Self {
        Self {
            val,
            next: next.map(Box::new),
        }
    }
}

struct TreeNode {
    val: i64,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}

impl TreeNode {
    fn new(val: i64, left: Option<TreeNode>, right: Option<TreeNode>) -> Self {
        Self {
            val,
            left: left.map(Box::new),
            right: right.map(Box::new),
        }
    }
}

fn main() {
    let n1 = LinkedNode::new(1, Some(LinkedNode::new(2, Some(LinkedNode::new(3, None)))));
    println!("{}", n1.val);

    let root = TreeNode::new(
        1,
        Some(TreeNode::new(2, None, None)),
        Some(TreeNode::new(3, None, None)),
    );
    println!("{}", root.val);

    let _tail = &n1.next;
    let _children = (&root.left, &root.right);
}
