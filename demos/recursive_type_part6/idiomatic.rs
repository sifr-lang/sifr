struct BinaryBranch {
    val: i64,
    left: Option<Box<BinaryBranch>>,
    right: Option<Box<BinaryBranch>>,
}

impl BinaryBranch {
    fn new(val: i64, left: Option<BinaryBranch>, right: Option<BinaryBranch>) -> Self {
        Self {
            val,
            left: left.map(Box::new),
            right: right.map(Box::new),
        }
    }
}

#[allow(dead_code)]
enum Packet<T> {
    Value(T),
    List(Vec<Packet<T>>),
}

fn tree_sum(node: Option<&BinaryBranch>) -> i64 {
    match node {
        Some(node) => node.val + tree_sum(node.left.as_deref()) + tree_sum(node.right.as_deref()),
        None => 0,
    }
}

fn main() {
    let root = BinaryBranch::new(
        1,
        Some(BinaryBranch::new(2, None, None)),
        Some(BinaryBranch::new(3, None, None)),
    );
    let _packet = Packet::List(vec![
        Packet::Value(1_i64),
        Packet::List(vec![Packet::Value(2_i64)]),
    ]);

    assert_eq!(tree_sum(Some(&root)), 6);
    println!("tree sum ok");
    println!("packet alias declared");
}
