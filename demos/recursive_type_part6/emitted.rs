#[derive(Debug, Clone, PartialEq)]
struct TreeNode {
    val: i64,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}

impl TreeNode {
    fn new(val: i64, left: Option<Box<TreeNode>>, right: Option<Box<TreeNode>>) -> Self {
        return Self { val: val, left: left, right: right };
    }
}

fn tree_sum(node: &Option<TreeNode>) -> i64 {
    let Some(node) = node.as_ref() else {
        return 0 as i64;
    };
    let left: Option<TreeNode> = (node.left).as_deref().cloned();
    let right: Option<TreeNode> = (node.right).as_deref().cloned();
    return (node.val + tree_sum(&left)) + tree_sum(&right);
}

fn main() {
    let left: TreeNode = TreeNode::new(2 as i64, None, None);
    let right: TreeNode = TreeNode::new(3 as i64, None, None);
    let root: TreeNode = TreeNode::new(1 as i64, Some(Box::new(left)), Some(Box::new(right)));
    assert!(tree_sum(&Some(root)) == (6 as i64));
    println!("tree sum ok");
    println!("packet alias declared");
}
