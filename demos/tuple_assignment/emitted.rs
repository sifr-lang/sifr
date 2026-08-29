// src/main.rs
use ::sifr_runtime::SifrInt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Pair {
    x: SifrInt,
    y: SifrInt,
}

impl Pair {
    fn new(x: SifrInt, y: SifrInt) -> Self {
        let __sifr_field_init_0: SifrInt = x.clone();
        let __sifr_field_init_1: SifrInt = y.clone();
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
    left: SifrInt,
    right: SifrInt,
}

impl RunningBounds {
    fn new(left: SifrInt, right: SifrInt) -> Self {
        let __sifr_field_init_0: SifrInt = left.clone();
        let __sifr_field_init_1: SifrInt = right.clone();
        Self { left: __sifr_field_init_0, right: __sifr_field_init_1 }
    }
}

impl RunningBounds {
    fn rotate(&mut self, next_value: &SifrInt) {
        let (__sifr_tuple_unpack_0, __sifr_tuple_unpack_1) = (self.right.clone(), next_value.clone());
        self.left = __sifr_tuple_unpack_0;
        self.right = __sifr_tuple_unpack_1;
    }
}

impl RunningBounds {
    fn as_text(&self) -> String {
        {
    let mut __sifr_concat: String = String::with_capacity((((1usize + 0usize) + 2usize) + 0usize) + 1usize);
    __sifr_concat.push('(');
    __sifr_concat.push_str((format!("{}", self.left.clone())).as_str());
    __sifr_concat.push_str(", ");
    __sifr_concat.push_str((format!("{}", self.right.clone())).as_str());
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
    let (__sifr_tuple_unpack_0, __sifr_tuple_unpack_1) = (pair.y.clone(), pair.x.clone());
    pair.x = __sifr_tuple_unpack_0;
    pair.y = __sifr_tuple_unpack_1;
}

fn add_points(points: &Vec<(SifrInt, SifrInt)>) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for point in points.iter().cloned() {
        total = &total + &(&(point).0.clone() + &(point).1.clone());
    }
    total.clone()
}

fn main() {
    let mut pair: Pair = Pair::new(SifrInt::from_i64(2), SifrInt::from_i64(5));
    swap_pair(&mut pair);
    assert!((&pair.x.clone() == &SifrInt::from_i64(5)));
    assert!((&pair.y.clone() == &SifrInt::from_i64(2)));
    assert!((&add_points(&vec![(SifrInt::from_i64(1), SifrInt::from_i64(2)), (SifrInt::from_i64(3), SifrInt::from_i64(4)), (SifrInt::from_i64(5), SifrInt::from_i64(6))]) == &SifrInt::from_i64(21)));
    let mut bounds: RunningBounds = RunningBounds::new(SifrInt::from_i64(10), SifrInt::from_i64(20));
    bounds.rotate(&SifrInt::from_i64(30));
    assert!((&bounds.left.clone() == &SifrInt::from_i64(20)));
    assert!((&bounds.right.clone() == &SifrInt::from_i64(30)));
    assert!((bounds.as_text() == "(20, 30)"));
    println!("tuple_assignment: ok");
}
