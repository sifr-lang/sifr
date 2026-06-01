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

fn main() {
    let left: TreeNode = TreeNode::new(2_i64, None, None);
    let right: TreeNode = TreeNode::new(3_i64, None, None);
    let root: TreeNode = TreeNode::new(1_i64, Some(Box::new(left)), Some(Box::new(right)));
    assert!((tree_value_sum(&Some((root).clone())) == (6_i64)));
    println!("tree sum ok");
    println!("packet alias declared");
}
