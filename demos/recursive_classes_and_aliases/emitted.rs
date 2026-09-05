// src/main.rs
use ::sifr_runtime::SifrInt;
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
fn tree_value_sum(node: Option<&TreeNode>) -> SifrInt {
    let Some(node) = node else {
        return SifrInt::from_i64(0);
    };
    let left: Option<&TreeNode> = node.left.as_deref();
    let right: Option<&TreeNode> = node.right.as_deref();
    &(&node.val.clone() + &tree_value_sum(left)) + &tree_value_sum(right)
}
fn main() {
    let left: TreeNode = TreeNode::new(SifrInt::from_i64(2), None, None);
    let right: TreeNode = TreeNode::new(SifrInt::from_i64(3), None, None);
    let root: TreeNode = TreeNode::new(
        SifrInt::from_i64(1),
        Some(Box::new(left)),
        Some(Box::new(right)),
    );
    assert_eq!(&tree_value_sum(Some(&root)), &SifrInt::from_i64(6));
    println!("tree sum ok");
    println!("packet alias declared");
}
