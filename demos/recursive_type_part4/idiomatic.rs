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

fn tree_sum(node: Option<&TreeNode>) -> i64 {
    match node {
        Some(node) => node.val + tree_sum(node.left.as_deref()) + tree_sum(node.right.as_deref()),
        None => 0,
    }
}

fn same_shape_and_sum(p: Option<&TreeNode>, q: Option<&TreeNode>) -> i64 {
    match (p, q) {
        (None, None) => 0,
        (Some(_), None) | (None, Some(_)) => -1,
        (Some(p), Some(q)) => {
            p.val
                + q.val
                + same_shape_and_sum(p.left.as_deref(), q.left.as_deref())
                + same_shape_and_sum(p.right.as_deref(), q.right.as_deref())
        }
    }
}

fn main() {
    let root_a = TreeNode::new(
        1,
        Some(TreeNode::new(2, None, None)),
        Some(TreeNode::new(3, None, None)),
    );
    let root_b = TreeNode::new(
        1,
        Some(TreeNode::new(2, None, None)),
        Some(TreeNode::new(3, None, None)),
    );
    let root_c = TreeNode::new(
        1,
        Some(TreeNode::new(2, None, None)),
        Some(TreeNode::new(3, None, None)),
    );

    assert_eq!(tree_sum(Some(&root_a)), 6);
    assert_eq!(same_shape_and_sum(Some(&root_b), Some(&root_c)), 12);
}
