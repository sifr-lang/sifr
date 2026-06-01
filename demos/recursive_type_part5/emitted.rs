// src/main.rs
#[derive(Debug, Clone, PartialEq)]
struct TreeNode {
    val: i64,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}

impl TreeNode {
    fn new(val: i64, left: Option<Box<TreeNode>>, right: Option<Box<TreeNode>>) -> Self {
        Self { val, left, right }
    }
}

fn tree_value_sum(node: &Option<TreeNode>) -> i64 {
    let Some(node) = node.as_ref() else {
        return 0_i64;
    };
    let left: Option<TreeNode> = (node.left).as_deref().cloned();
    let right: Option<TreeNode> = (node.right).as_deref().cloned();
    (node.val + tree_value_sum(&left)) + tree_value_sum(&right)
}

fn paired_tree_value_sum(p: &Option<TreeNode>, q: &Option<TreeNode>) -> i64 {
    if p.is_none() && q.is_none() {
        return 0_i64;
    }
    let (Some(p), Some(q)) = (p.as_ref(), q.as_ref()) else {
        return -(1_i64);
    };
    ((p.val + q.val) + paired_tree_value_sum(&(p.left).as_deref().cloned(), &(q.left).as_deref().cloned())) + paired_tree_value_sum(&(p.right).as_deref().cloned(), &(q.right).as_deref().cloned())
}

fn main() {
    let left_a: TreeNode = TreeNode::new(2_i64, None, None);
    let right_a: TreeNode = TreeNode::new(3_i64, None, None);
    let root_a: TreeNode = TreeNode::new(1_i64, Some(Box::new(left_a)), Some(Box::new(right_a)));
    let left_b: TreeNode = TreeNode::new(2_i64, None, None);
    let right_b: TreeNode = TreeNode::new(3_i64, None, None);
    let root_b: TreeNode = TreeNode::new(1_i64, Some(Box::new(left_b)), Some(Box::new(right_b)));
    let left_c: TreeNode = TreeNode::new(2_i64, None, None);
    let right_c: TreeNode = TreeNode::new(3_i64, None, None);
    let root_c: TreeNode = TreeNode::new(1_i64, Some(Box::new(left_c)), Some(Box::new(right_c)));
    assert!((tree_value_sum(&Some((root_a).clone())) == (6_i64)));
    assert!((paired_tree_value_sum(&Some((root_b).clone()), &Some((root_c).clone())) == (12_i64)));
}
