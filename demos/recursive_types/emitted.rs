// src/main.rs
use ::sifr_runtime::SifrInt;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LinkedNode {
    val: SifrInt,
    next: Option<Box<LinkedNode>>,
}
impl LinkedNode {
    fn new(val: SifrInt, next: Option<Box<LinkedNode>>) -> Self {
        let sifr_generated_field_value_690422194ed16e3c_76616c: SifrInt = val.clone();
        let sifr_generated_field_value_e5316cbaa025f028_6e657874: Option<Box<LinkedNode>> = next;
        Self {
            val: sifr_generated_field_value_690422194ed16e3c_76616c,
            next: sifr_generated_field_value_e5316cbaa025f028_6e657874,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TreeNode {
    val: SifrInt,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}
impl TreeNode {
    fn new(val: SifrInt, left: Option<Box<TreeNode>>, right: Option<Box<TreeNode>>) -> Self {
        let sifr_generated_field_value_690422194ed16e3c_76616c: SifrInt = val.clone();
        let sifr_generated_field_value_24b070ada2041cb0_6c656674: Option<Box<TreeNode>> = left;
        let sifr_generated_field_value_76aaaa535714d805_7269676874: Option<Box<TreeNode>> = right;
        Self {
            val: sifr_generated_field_value_690422194ed16e3c_76616c,
            left: sifr_generated_field_value_24b070ada2041cb0_6c656674,
            right: sifr_generated_field_value_76aaaa535714d805_7269676874,
        }
    }
}
fn main() {
    let n3: LinkedNode = LinkedNode::new(SifrInt::from_i64(3), None);
    let n2: LinkedNode = LinkedNode::new(SifrInt::from_i64(2), Some(Box::new(n3)));
    let n1: LinkedNode = LinkedNode::new(SifrInt::from_i64(1), Some(Box::new(n2)));
    println!("{}", n1.val.clone());
    let left: TreeNode = TreeNode::new(SifrInt::from_i64(2), None, None);
    let right: TreeNode = TreeNode::new(SifrInt::from_i64(3), None, None);
    let root: TreeNode = TreeNode::new(
        SifrInt::from_i64(1),
        Some(Box::new(left)),
        Some(Box::new(right)),
    );
    println!("{}", root.val.clone());
}
