#[derive(Debug, Clone, PartialEq)]
struct ListNode {
    val: i64,
    next: Option<Box<ListNode>>,
}

impl ListNode {
    fn new(val: i64, next: Option<Box<ListNode>>) -> Self {
        return Self { val: val, next: next };
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
        return Self { val: val, left: left, right: right };
    }
}

fn main() {
    let n3: ListNode = ListNode::new(3 as i64, None);
    let n2: ListNode = ListNode::new(2 as i64, Some(Box::new(n3)));
    let n1: ListNode = ListNode::new(1 as i64, Some(Box::new(n2)));
    println!("{}", n1.val);
    let left: TreeNode = TreeNode::new(2 as i64, None, None);
    let right: TreeNode = TreeNode::new(3 as i64, None, None);
    let root: TreeNode = TreeNode::new(1 as i64, Some(Box::new(left)), Some(Box::new(right)));
    println!("{}", root.val);
}
