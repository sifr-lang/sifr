// src/main.rs
use ::sifr_runtime::SifrInt;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Pair {
    x: SifrInt,
    y: SifrInt,
}
impl Pair {
    const fn new(x: SifrInt, y: SifrInt) -> Self {
        let sifr_generated_field_value_af63f54c86021707_78: SifrInt = x;
        let sifr_generated_field_value_af63f44c86021554_79: SifrInt = y;
        Self {
            x: sifr_generated_field_value_af63f54c86021707_78,
            y: sifr_generated_field_value_af63f44c86021554_79,
        }
    }
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
    const fn new(left: SifrInt, right: SifrInt) -> Self {
        let sifr_generated_field_value_24b070ada2041cb0_6c656674: SifrInt = left;
        let sifr_generated_field_value_76aaaa535714d805_7269676874: SifrInt = right;
        Self {
            left: sifr_generated_field_value_24b070ada2041cb0_6c656674,
            right: sifr_generated_field_value_76aaaa535714d805_7269676874,
        }
    }
}
impl RunningBounds {
    fn rotate(&mut self, next_value: &SifrInt) {
        let (sifr_generated_tuple_unpack_0, sifr_generated_tuple_unpack_1) =
            (self.right.clone(), next_value.clone());
        self.left = sifr_generated_tuple_unpack_0;
        self.right = sifr_generated_tuple_unpack_1;
    }
}
impl RunningBounds {
    fn as_text(&self) -> String {
        {
            let mut sifr_generated_concat: String = String::with_capacity(
                1usize
                    .saturating_add(0usize)
                    .saturating_add(2usize)
                    .saturating_add(0usize)
                    .saturating_add(1usize),
            );
            sifr_generated_concat.push('(');
            sifr_generated_concat.push_str(self.left.clone().to_string().as_str());
            sifr_generated_concat.push_str(", ");
            sifr_generated_concat.push_str(self.right.clone().to_string().as_str());
            sifr_generated_concat.push(')');
            sifr_generated_concat
        }
    }
}
impl ::std::fmt::Display for RunningBounds {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "RunningBounds(left={}, right={})", self.left, self.right)
    }
}
fn swap_pair(pair: &mut Pair) {
    let (sifr_generated_tuple_unpack_0, sifr_generated_tuple_unpack_1) =
        (pair.y.clone(), pair.x.clone());
    pair.x = sifr_generated_tuple_unpack_0;
    pair.y = sifr_generated_tuple_unpack_1;
}
fn add_points(points: &[(SifrInt, SifrInt)]) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for point in points.iter().cloned() {
        total = ::std::ops::Add::add(
            &total,
            &::std::ops::Add::add(&point.0.clone(), &point.1.clone()),
        );
    }
    total
}
fn main() {
    let mut pair: Pair = Pair::new(SifrInt::from_i64(2), SifrInt::from_i64(5));
    swap_pair(&mut pair);
    assert_eq!(pair.x, SifrInt::from_i64(5));
    assert_eq!(pair.y, SifrInt::from_i64(2));
    assert_eq!(
        add_points(&[
            (SifrInt::from_i64(1), SifrInt::from_i64(2)),
            (SifrInt::from_i64(3), SifrInt::from_i64(4)),
            (SifrInt::from_i64(5), SifrInt::from_i64(6))
        ]),
        SifrInt::from_i64(21)
    );
    let mut bounds: RunningBounds =
        RunningBounds::new(SifrInt::from_i64(10), SifrInt::from_i64(20));
    bounds.rotate(&SifrInt::from_i64(30));
    assert_eq!(bounds.left, SifrInt::from_i64(20));
    assert_eq!(bounds.right, SifrInt::from_i64(30));
    assert_eq!(bounds.as_text(), "(20, 30)");
    println!("tuple_assignment: ok");
}
