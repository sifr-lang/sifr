#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Pair {
    x: i64,
    y: i64,
}

impl Pair {
    fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}

impl std::fmt::Display for Pair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Pair(x={}, y={})", self.x, self.y);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RunningBounds {
    left: i64,
    right: i64,
}

impl RunningBounds {
    fn new(left: i64, right: i64) -> Self {
        Self { left, right }
    }
    fn rotate(&mut self, next_value: i64) {
        self.left = std::mem::replace(&mut self.right, next_value);
    }
    fn as_text(&self) -> String {
        format!("({}, {})", self.left, self.right)
    }
}

impl std::fmt::Display for RunningBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "RunningBounds(left={}, right={})", self.left, self.right);
    }
}

fn swap_pair(pair: &mut Pair) {
    std::mem::swap(&mut pair.x, &mut pair.y);
}

fn add_points(points: &[(i64, i64)]) -> i64 {
    let mut total = 0;
    for (x, y) in points.iter().copied() {
        total += x + y;
    }
    total
}

fn main() {
    let mut pair = Pair::new(2, 5);
    swap_pair(&mut pair);
    assert_eq!(pair.x, 5);
    assert_eq!(pair.y, 2);
    assert_eq!(add_points(&[(1, 2), (3, 4), (5, 6)]), 21);

    let mut bounds = RunningBounds::new(10, 20);
    bounds.rotate(30);
    assert_eq!(bounds.left, 20);
    assert_eq!(bounds.right, 30);
    assert_eq!(bounds.as_text(), "(20, 30)");
    println!("tuple_assignment: ok");
}
