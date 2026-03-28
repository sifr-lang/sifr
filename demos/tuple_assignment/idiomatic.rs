#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Pair {
    x: i64,
    y: i64,
}

impl Pair {
    fn new(x: i64, y: i64) -> Self {
        return Self { x: x, y: y };
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
        return Self {
            left: left,
            right: right,
        };
    }
    fn rotate(&self, next_value: i64) {
        let (__sifr_tuple_unpack_0, __sifr_tuple_unpack_1) = (self.right, next_value);
        self.left = __sifr_tuple_unpack_0;
        self.right = __sifr_tuple_unpack_1;
    }
    fn as_text(&self) -> String {
        return format!(
            "{}{}{}{}{}",
            "(".to_string(),
            format!("{}", self.left),
            ", ".to_string(),
            format!("{}", self.right),
            ")".to_string()
        );
    }
}

impl std::fmt::Display for RunningBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "RunningBounds(left={}, right={})", self.left, self.right);
    }
}

fn swap_pair(pair: &mut Pair) {
    let (__sifr_tuple_unpack_0, __sifr_tuple_unpack_1) = (pair.y, pair.x);
    pair.x = __sifr_tuple_unpack_0;
    pair.y = __sifr_tuple_unpack_1;
}

fn add_points(points: &Vec<(i64, i64)>) -> i64 {
    let mut total: i64 = 0 as i64;
    for point in points.iter().copied() {
        total += (point).0 + (point).1;
    }
    return total;
}

fn main() {
    let mut pair: Pair = Pair::new(2 as i64, 5 as i64);
    swap_pair(&mut pair);
    assert!(pair.x == (5 as i64));
    assert!(pair.y == (2 as i64));
    assert!(
        add_points(&vec![
            (1 as i64, 2 as i64),
            (3 as i64, 4 as i64),
            (5 as i64, 6 as i64)
        ]) == (21 as i64)
    );
    let mut bounds: RunningBounds = RunningBounds::new(10 as i64, 20 as i64);
    bounds.rotate(30 as i64);
    assert!(bounds.left == (20 as i64));
    assert!(bounds.right == (30 as i64));
    assert!(bounds.as_text() == "(20, 30)".to_string());
    println!("tuple_assignment: ok");
}
