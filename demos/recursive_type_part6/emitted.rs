#[derive(Debug, Clone, PartialEq)]
struct BinaryBranch {
    val: i64,
    left: Option<Box<BinaryBranch>>,
    right: Option<Box<BinaryBranch>>,
}

impl BinaryBranch {
    fn new(val: i64, left: Option<Box<BinaryBranch>>, right: Option<Box<BinaryBranch>>) -> Self {
        return Self { val: val, left: left, right: right };
    }
}

fn tree_sum(node: &Option<BinaryBranch>) -> i64 {
    let Some(node) = node.as_ref() else {
        return 0 as i64;
    };
    let left: Option<BinaryBranch> = (node.left).as_deref().cloned();
    let right: Option<BinaryBranch> = (node.right).as_deref().cloned();
    return (node.val + tree_sum(&left)) + tree_sum(&right);
}

fn main() {
    let left: BinaryBranch = BinaryBranch::new(2 as i64, None, None);
    let right: BinaryBranch = BinaryBranch::new(3 as i64, None, None);
    let root: BinaryBranch = BinaryBranch::new(1 as i64, Some(Box::new(left)), Some(Box::new(right)));
    assert!(tree_sum(&Some(root)) == (6 as i64));
    println!("tree sum ok");
    println!("packet alias declared");
}
