#[derive(Debug, Clone, PartialEq)]
struct ChainCell {
    val: i64,
    next: Option<Box<ChainCell>>,
}

impl ChainCell {
    fn new(val: i64, next: Option<Box<ChainCell>>) -> Self {
        return Self { val: val, next: next };
    }
}

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

fn main() {
    let n3: ChainCell = ChainCell::new(3 as i64, None);
    let n2: ChainCell = ChainCell::new(2 as i64, Some(Box::new(n3)));
    let n1: ChainCell = ChainCell::new(1 as i64, Some(Box::new(n2)));
    println!("{}", n1.val);
    let left: BinaryBranch = BinaryBranch::new(2 as i64, None, None);
    let right: BinaryBranch = BinaryBranch::new(3 as i64, None, None);
    let root: BinaryBranch = BinaryBranch::new(1 as i64, Some(Box::new(left)), Some(Box::new(right)));
    println!("{}", root.val);
}
