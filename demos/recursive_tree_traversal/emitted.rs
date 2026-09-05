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
fn paired_tree_value_sum(p: Option<&TreeNode>, q: Option<&TreeNode>) -> SifrInt {
    if !p.is_some() && !q.is_some() {
        return SifrInt::from_i64(0);
    }
    let (Some(p), Some(q)) = (p, q) else {
        return -&SifrInt::from_i64(1);
    };
    &(&(&p.val.clone() + &q.val.clone())
        + &paired_tree_value_sum(p.left.as_deref(), q.left.as_deref()))
        + &paired_tree_value_sum(p.right.as_deref(), q.right.as_deref())
}
fn main() {
    let left_a: TreeNode = TreeNode::new(SifrInt::from_i64(2), None, None);
    let right_a: TreeNode = TreeNode::new(SifrInt::from_i64(3), None, None);
    let root_a: TreeNode = TreeNode::new(
        SifrInt::from_i64(1),
        Some(Box::new(left_a)),
        Some(Box::new(right_a)),
    );
    let left_b_value_2fdbe280d22f35cd: TreeNode = TreeNode::new(SifrInt::from_i64(2), None, None);
    let right_b_value_824555ba1ee1cde4: TreeNode = TreeNode::new(SifrInt::from_i64(3), None, None);
    let root_b_value_f7653824868658a4: TreeNode = TreeNode::new(
        SifrInt::from_i64(1),
        Some(Box::new(left_b_value_2fdbe280d22f35cd)),
        Some(Box::new(right_b_value_824555ba1ee1cde4)),
    );
    let left_c_value_2fdbe180d22f341a: TreeNode = TreeNode::new(SifrInt::from_i64(2), None, None);
    let right_c_value_824556ba1ee1cf97: TreeNode = TreeNode::new(SifrInt::from_i64(3), None, None);
    let root_c_value_f765392486865a57: TreeNode = TreeNode::new(
        SifrInt::from_i64(1),
        Some(Box::new(left_c_value_2fdbe180d22f341a)),
        Some(Box::new(right_c_value_824556ba1ee1cf97)),
    );
    assert_eq!(&tree_value_sum(Some(&root_a)), &SifrInt::from_i64(6));
    assert_eq!(
        &paired_tree_value_sum(
            Some(&root_b_value_f7653824868658a4),
            Some(&root_c_value_f765392486865a57)
        ),
        &SifrInt::from_i64(12)
    );
}
