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

fn same_shape_and_sum(p: &Option<BinaryBranch>, q: &Option<BinaryBranch>) -> i64 {
    if ((p.is_none()) && (q.is_none())) {
        return 0 as i64;
    }
    let (Some(p), Some(q)) = (p.as_ref(), q.as_ref()) else {
        return -(1 as i64);
    };
    return ((p.val + q.val) + same_shape_and_sum(&(p.left).as_deref().cloned(), &(q.left).as_deref().cloned())) + same_shape_and_sum(&(p.right).as_deref().cloned(), &(q.right).as_deref().cloned());
}

fn main() {
    let left_a: BinaryBranch = BinaryBranch::new(2 as i64, None, None);
    let right_a: BinaryBranch = BinaryBranch::new(3 as i64, None, None);
    let root_a: BinaryBranch = BinaryBranch::new(1 as i64, Some(Box::new(left_a)), Some(Box::new(right_a)));
    let left_b: BinaryBranch = BinaryBranch::new(2 as i64, None, None);
    let right_b: BinaryBranch = BinaryBranch::new(3 as i64, None, None);
    let root_b: BinaryBranch = BinaryBranch::new(1 as i64, Some(Box::new(left_b)), Some(Box::new(right_b)));
    let left_c: BinaryBranch = BinaryBranch::new(2 as i64, None, None);
    let right_c: BinaryBranch = BinaryBranch::new(3 as i64, None, None);
    let root_c: BinaryBranch = BinaryBranch::new(1 as i64, Some(Box::new(left_c)), Some(Box::new(right_c)));
    assert!(tree_sum(&Some(root_a)) == (6 as i64));
    assert!(same_shape_and_sum(&Some(root_b), &Some(root_c)) == (12 as i64));
}
