// src/main.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LinkedNode {
    val: i64,
    next: Option<Box<LinkedNode>>,
}

impl LinkedNode {
    fn new(val: i64, next: Option<Box<LinkedNode>>) -> Self {
        let __sifr_field_init_0: i64 = val;
        let __sifr_field_init_1: Option<Box<LinkedNode>> = next;
        Self { val: __sifr_field_init_0, next: __sifr_field_init_1 }
    }
}

impl LinkedNode {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TreeNode {
    val: i64,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}

impl TreeNode {
    fn new(val: i64, left: Option<Box<TreeNode>>, right: Option<Box<TreeNode>>) -> Self {
        let __sifr_field_init_0: i64 = val;
        let __sifr_field_init_1: Option<Box<TreeNode>> = left;
        let __sifr_field_init_2: Option<Box<TreeNode>> = right;
        Self { val: __sifr_field_init_0, left: __sifr_field_init_1, right: __sifr_field_init_2 }
    }
}

impl TreeNode {
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
