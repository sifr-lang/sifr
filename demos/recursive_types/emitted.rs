// src/main.rs
use ::sifr_runtime::SifrInt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LinkedNode {
    val: SifrInt,
    next: Option<Box<LinkedNode>>,
}

impl LinkedNode {
    fn new(val: SifrInt, next: Option<Box<LinkedNode>>) -> Self {
        let __sifr_field_init_0: SifrInt = val.clone();
        let __sifr_field_init_1: Option<Box<LinkedNode>> = next;
        Self { val: __sifr_field_init_0, next: __sifr_field_init_1 }
    }
}

impl LinkedNode {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TreeNode {
    val: SifrInt,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}

impl TreeNode {
    fn new(val: SifrInt, left: Option<Box<TreeNode>>, right: Option<Box<TreeNode>>) -> Self {
        let __sifr_field_init_0: SifrInt = val.clone();
        let __sifr_field_init_1: Option<Box<TreeNode>> = left;
        let __sifr_field_init_2: Option<Box<TreeNode>> = right;
        Self { val: __sifr_field_init_0, left: __sifr_field_init_1, right: __sifr_field_init_2 }
    }
}

impl TreeNode {
}

fn main() {
    let n3: LinkedNode = LinkedNode::new(SifrInt::from_i64(3), None);
    let n2: LinkedNode = LinkedNode::new(SifrInt::from_i64(2), Some(Box::new(n3)));
    let n1: LinkedNode = LinkedNode::new(SifrInt::from_i64(1), Some(Box::new(n2)));
    println!("{}", n1.val.clone());
    let left: TreeNode = TreeNode::new(SifrInt::from_i64(2), None, None);
    let right: TreeNode = TreeNode::new(SifrInt::from_i64(3), None, None);
    let root: TreeNode = TreeNode::new(SifrInt::from_i64(1), Some(Box::new(left)), Some(Box::new(right)));
    println!("{}", root.val.clone());
}
