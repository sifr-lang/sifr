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
        let __sifr_field_init_0: SifrInt = val.clone();
        let __sifr_field_init_1: Option<Box<TreeNode>> = left;
        let __sifr_field_init_2: Option<Box<TreeNode>> = right;
        Self { val: __sifr_field_init_0, left: __sifr_field_init_1, right: __sifr_field_init_2 }
    }
}

impl TreeNode {
}

fn tree_value_sum(node: &Option<TreeNode>) -> SifrInt {
    let Some(node) = node.as_ref() else {
        return SifrInt::from_i64(0);
    };
    let left: Option<TreeNode> = (node.left).as_deref().cloned();
    let right: Option<TreeNode> = (node.right).as_deref().cloned();
    &(&node.val.clone() + &tree_value_sum(&left)) + &tree_value_sum(&right)
}

fn paired_tree_value_sum(p: &Option<TreeNode>, q: &Option<TreeNode>) -> SifrInt {
    if !p.is_some() && !q.is_some() {
        return SifrInt::from_i64(0);
    }
    let (Some(p), Some(q)) = (p.as_ref(), q.as_ref()) else {
        return -&SifrInt::from_i64(1);
    };
    &(&(&p.val.clone() + &q.val.clone()) + &paired_tree_value_sum(&(p.left).as_deref().cloned(), &(q.left).as_deref().cloned())) + &paired_tree_value_sum(&(p.right).as_deref().cloned(), &(q.right).as_deref().cloned())
}

fn main() {
    let left_a: TreeNode = TreeNode::new(SifrInt::from_i64(2), None, None);
    let right_a: TreeNode = TreeNode::new(SifrInt::from_i64(3), None, None);
    let root_a: TreeNode = TreeNode::new(SifrInt::from_i64(1), Some(Box::new(left_a)), Some(Box::new(right_a)));
    let left_b: TreeNode = TreeNode::new(SifrInt::from_i64(2), None, None);
    let right_b: TreeNode = TreeNode::new(SifrInt::from_i64(3), None, None);
    let root_b: TreeNode = TreeNode::new(SifrInt::from_i64(1), Some(Box::new(left_b)), Some(Box::new(right_b)));
    let left_c: TreeNode = TreeNode::new(SifrInt::from_i64(2), None, None);
    let right_c: TreeNode = TreeNode::new(SifrInt::from_i64(3), None, None);
    let root_c: TreeNode = TreeNode::new(SifrInt::from_i64(1), Some(Box::new(left_c)), Some(Box::new(right_c)));
    assert!((&tree_value_sum(&Some((root_a).clone())) == &SifrInt::from_i64(6)));
    assert!((&paired_tree_value_sum(&Some((root_b).clone()), &Some((root_c).clone())) == &SifrInt::from_i64(12)));
}
