// src/main.rs
mod sifr_generated_project_unions {
    #[derive(Debug, Clone, PartialEq)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a224X3a5X3aclass11X3amainX2eCircle1X3a024X3a5X3aclass11X3amainX2eSquare1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass11X3amainX2eCircle1X3a0(crate::Circle),
        SifrGeneratedUnionVariant5X3aclass11X3amainX2eSquare1X3a0(crate::Square),
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a224X3a5X3aclass11X3amainX2eCircle1X3a024X3a5X3aclass11X3amainX2eSquare1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::SifrGeneratedUnionVariant5X3aclass11X3amainX2eCircle1X3a0(v) => {
                    write!(f, "{v}")
                }
                Self::SifrGeneratedUnionVariant5X3aclass11X3amainX2eSquare1X3a0(v) => {
                    write!(f, "{v}")
                }
            }
        }
    }
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a224X3a5X3aclass11X3amainX2eCircle1X3a024X3a5X3aclass11X3amainX2eSquare1X3a0;
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}
impl Point {
    const fn new(x: f64, y: f64) -> Self {
        let sifr_generated_field_value_af63f54c86021707_78: f64 = x;
        let sifr_generated_field_value_af63f44c86021554_79: f64 = y;
        Self {
            x: sifr_generated_field_value_af63f54c86021707_78,
            y: sifr_generated_field_value_af63f44c86021554_79,
        }
    }
}
impl Point {
    #[expect(
        clippy::suboptimal_flops,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    fn distance(&self, other: &Self) -> f64 {
        let dx: f64 = self.x - other.x;
        let dy: f64 = self.y - other.y;
        (dx * dx + dy * dy).powf(0.5_f64)
    }
}
impl ::std::fmt::Display for Point {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Point(x={}, y={})", self.x, self.y)
    }
}
#[derive(Debug, Clone, PartialEq)]
struct Rectangle {
    width: f64,
    height: f64,
}
impl Rectangle {
    const fn new(width: f64, height: f64) -> Self {
        let sifr_generated_field_value_dbdacd932fd1e9bf_7769647468: f64 = width;
        let sifr_generated_field_value_17720bf67d347222_686569676874: f64 = height;
        Self {
            width: sifr_generated_field_value_dbdacd932fd1e9bf_7769647468,
            height: sifr_generated_field_value_17720bf67d347222_686569676874,
        }
    }
}
impl Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}
impl Rectangle {
    fn perimeter(&self) -> f64 {
        2.0_f64 * (self.width + self.height)
    }
}
impl ::std::fmt::Display for Rectangle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Rectangle(width={}, height={})", self.width, self.height)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    radius: f64,
}
impl Circle {
    const fn new(radius: f64) -> Self {
        let sifr_generated_field_value_a293b946d5782cf3_726164697573: f64 = radius;
        Self {
            radius: sifr_generated_field_value_a293b946d5782cf3_726164697573,
        }
    }
}
impl ::std::fmt::Display for Circle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Circle(radius={})", self.radius)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Square {
    side: f64,
}
impl Square {
    const fn new(side: f64) -> Self {
        let sifr_generated_field_value_4e0c8a18e635803e_73696465: f64 = side;
        Self {
            side: sifr_generated_field_value_4e0c8a18e635803e_73696465,
        }
    }
}
impl ::std::fmt::Display for Square {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Square(side={})", self.side)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Color {
    r: SifrInt,
    g: SifrInt,
    b: SifrInt,
}
impl Color {
    fn new(r: &SifrInt, g: &SifrInt, b: &SifrInt) -> Self {
        let sifr_generated_field_value_af63ef4c86020cd5_72: SifrInt = (*r).clone();
        let sifr_generated_field_value_af63da4c8601e926_67: SifrInt = (*g).clone();
        let sifr_generated_field_value_af63df4c8601f1a5_62: SifrInt = (*b).clone();
        Self {
            r: sifr_generated_field_value_af63ef4c86020cd5_72,
            g: sifr_generated_field_value_af63da4c8601e926_67,
            b: sifr_generated_field_value_af63df4c8601f1a5_62,
        }
    }
}
impl ::std::fmt::Display for Color {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Color(r={}, g={}, b={})", self.r, self.g, self.b)
    }
}
fn describe_shape(
    shape: &SifrGeneratedUnion8X3asequence5X3aunion1X3a224X3a5X3aclass11X3amainX2eCircle1X3a024X3a5X3aclass11X3amainX2eSquare1X3a0,
) {
    match shape {
        SifrGeneratedUnion8X3asequence5X3aunion1X3a224X3a5X3aclass11X3amainX2eCircle1X3a024X3a5X3aclass11X3amainX2eSquare1X3a0::SifrGeneratedUnionVariant5X3aclass11X3amainX2eCircle1X3a0(
            shape,
        ) => {
            println!("Circle: radius={}", shape.radius);
        }
        SifrGeneratedUnion8X3asequence5X3aunion1X3a224X3a5X3aclass11X3amainX2eCircle1X3a024X3a5X3aclass11X3amainX2eSquare1X3a0::SifrGeneratedUnionVariant5X3aclass11X3amainX2eSquare1X3a0(
            shape,
        ) => {
            println!("Square: side={}", shape.side);
        }
    }
}
fn main() {
    println!("=== Basic Class ===");
    let origin: Point = Point::new(0.0_f64, 0.0_f64);
    let target: Point = Point::new(3.0_f64, 4.0_f64);
    let d: f64 = origin.distance(&target);
    println!("Distance from origin to (3,4): {d}");
    println!("=== Methods ===");
    let rect: Rectangle = Rectangle::new(5.0_f64, 3.0_f64);
    println!("Rectangle area: {}", rect.area());
    println!("Rectangle perimeter: {}", rect.perimeter());
    println!("=== Field Access ===");
    println!("Rectangle width: {}", rect.width);
    println!("Rectangle height: {}", rect.height);
    println!("=== Union + isinstance ===");
    let c: Circle = Circle::new(10.0_f64);
    let s: Square = Square::new(7.0_f64);
    describe_shape(
        &SifrGeneratedUnion8X3asequence5X3aunion1X3a224X3a5X3aclass11X3amainX2eCircle1X3a024X3a5X3aclass11X3amainX2eSquare1X3a0::SifrGeneratedUnionVariant5X3aclass11X3amainX2eCircle1X3a0(
            c,
        ),
    );
    describe_shape(
        &SifrGeneratedUnion8X3asequence5X3aunion1X3a224X3a5X3aclass11X3amainX2eCircle1X3a024X3a5X3aclass11X3amainX2eSquare1X3a0::SifrGeneratedUnionVariant5X3aclass11X3amainX2eSquare1X3a0(
            s,
        ),
    );
    println!("=== Hash ===");
    let red: Color = Color::new(
        &SifrInt::from_i64(255),
        &SifrInt::from_i64(0),
        &SifrInt::from_i64(0),
    );
    let also_red: Color = Color::new(
        &SifrInt::from_i64(255),
        &SifrInt::from_i64(0),
        &SifrInt::from_i64(0),
    );
    let h1: SifrInt = {
        let mut sifr_generated_hash = ::std::collections::hash_map::DefaultHasher::new();
        ::std::hash::Hash::hash(&red, &mut sifr_generated_hash);
        SifrInt::from(::std::hash::Hasher::finish(&sifr_generated_hash))
    };
    let h2: SifrInt = {
        let mut sifr_generated_hash = ::std::collections::hash_map::DefaultHasher::new();
        ::std::hash::Hash::hash(&also_red, &mut sifr_generated_hash);
        SifrInt::from(::std::hash::Hasher::finish(&sifr_generated_hash))
    };
    println!("Same color same hash: {}", h1 == h2);
    println!("=== Done ===");
}
