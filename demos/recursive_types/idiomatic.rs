struct ChainCell {
    val: i64,
    next: Option<Box<ChainCell>>,
}

impl ChainCell {
    fn new(val: i64, next: Option<ChainCell>) -> Self {
        Self {
            val,
            next: next.map(Box::new),
        }
    }
}

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

fn main() {
    let n1 = ChainCell::new(1, Some(ChainCell::new(2, Some(ChainCell::new(3, None)))));
    println!("{}", n1.val);

    let root = BinaryBranch::new(
        1,
        Some(BinaryBranch::new(2, None, None)),
        Some(BinaryBranch::new(3, None, None)),
    );
    println!("{}", root.val);

    let _tail = &n1.next;
    let _children = (&root.left, &root.right);
}
