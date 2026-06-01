// src/main.rs
#[derive(Debug, Clone, PartialEq)]
struct LinkedNode {
    val: i64,
    next: Option<Box<LinkedNode>>,
}

impl LinkedNode {
    fn new(val: i64, next: Option<Box<LinkedNode>>) -> Self {
        Self { val, next }
    }
}

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

fn main() {
    let n3: LinkedNode = LinkedNode::new(3_i64, None);
    let n2: LinkedNode = LinkedNode::new(2_i64, Some(Box::new(n3)));
    let n1: LinkedNode = LinkedNode::new(1_i64, Some(Box::new(n2)));
    println!("{}", n1.val);
    let left: TreeNode = TreeNode::new(2_i64, None, None);
    let right: TreeNode = TreeNode::new(3_i64, None, None);
    let root: TreeNode = TreeNode::new(1_i64, Some(Box::new(left)), Some(Box::new(right)));
    println!("{}", root.val);
}
