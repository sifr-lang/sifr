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

#[allow(dead_code)]
enum Packet<T> {
    Value(T),
    List(Vec<Packet<T>>),
}

fn tree_value_sum(node: Option<&TreeNode>) -> i64 {
    match node {
        Some(node) => node.val + tree_value_sum(node.left.as_deref()) + tree_value_sum(node.right.as_deref()),
        None => 0,
    }
}

fn main() {
    let root = TreeNode::new(
        1,
        Some(TreeNode::new(2, None, None)),
        Some(TreeNode::new(3, None, None)),
    );
    let _packet = Packet::List(vec![
        Packet::Value(1_i64),
        Packet::List(vec![Packet::Value(2_i64)]),
    ]);

    assert_eq!(tree_value_sum(Some(&root)), 6);
    println!("tree sum ok");
    println!("packet alias declared");
}
