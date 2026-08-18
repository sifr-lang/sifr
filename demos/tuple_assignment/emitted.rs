// src/main.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Pair {
    x: i64,
    y: i64,
}

impl Pair {
    fn new(x: i64, y: i64) -> Self {
        let __sifr_field_init_0: i64 = x;
        let __sifr_field_init_1: i64 = y;
        Self { x: __sifr_field_init_0, y: __sifr_field_init_1 }
    }
}

impl Pair {
}

impl ::std::fmt::Display for Pair {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Pair(x={}, y={})", self.x, self.y)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RunningBounds {
    left: i64,
    right: i64,
}

impl RunningBounds {
    fn new(left: i64, right: i64) -> Self {
        let __sifr_field_init_0: i64 = left;
        let __sifr_field_init_1: i64 = right;
        Self { left: __sifr_field_init_0, right: __sifr_field_init_1 }
    }
}

impl RunningBounds {
    fn rotate(&mut self, next_value: i64) {
        let (__sifr_tuple_unpack_0, __sifr_tuple_unpack_1) = (self.right, next_value);
        self.left = __sifr_tuple_unpack_0;
        self.right = __sifr_tuple_unpack_1;
    }
}

impl RunningBounds {
    fn as_text(&self) -> String {
        {
    let mut __sifr_concat: String = String::with_capacity((((1usize + 0usize) + 2usize) + 0usize) + 1usize);
    __sifr_concat.push('(');
    __sifr_concat.push_str((format!("{}", self.left)).as_str());
    __sifr_concat.push_str(", ");
    __sifr_concat.push_str((format!("{}", self.right)).as_str());
    __sifr_concat.push(')');
    __sifr_concat
}
    }
}

impl ::std::fmt::Display for RunningBounds {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "RunningBounds(left={}, right={})", self.left, self.right)
    }
}

fn swap_pair(pair: &mut Pair) {
    let (__sifr_tuple_unpack_0, __sifr_tuple_unpack_1) = (pair.y, pair.x);
    pair.x = __sifr_tuple_unpack_0;
    pair.y = __sifr_tuple_unpack_1;
}

fn add_points(points: &Vec<(i64, i64)>) -> i64 {
    let mut total: i64 = 0_i64;
    for point in points.iter().copied() {
        total += (point).0 + (point).1;
    }
    total
}

fn main() {
    let mut pair: Pair = Pair::new(2_i64, 5_i64);
    swap_pair(&mut pair);
    assert!((pair.x == (5_i64)));
    assert!((pair.y == (2_i64)));
    assert!((add_points(&vec![(1_i64, 2_i64), (3_i64, 4_i64), (5_i64, 6_i64)]) == (21_i64)));
    let mut bounds: RunningBounds = RunningBounds::new(10_i64, 20_i64);
    bounds.rotate(30_i64);
    assert!((bounds.left == (20_i64)));
    assert!((bounds.right == (30_i64)));
    assert!((bounds.as_text() == "(20, 30)"));
    println!("tuple_assignment: ok");
}
