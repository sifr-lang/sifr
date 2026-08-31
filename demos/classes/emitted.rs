// src/main.rs
mod __sifr_project_unions {
    #[derive(Debug, Clone, PartialEq)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a224_x3a5_x3aclass11_x3amain_x2eCircle1_x3a024_x3a5_x3aclass11_x3amain_x2eSquare1_x3a0 {
        __SifrUnionVariant_5_x3aclass11_x3amain_x2eCircle1_x3a0(crate::Circle),
        __SifrUnionVariant_5_x3aclass11_x3amain_x2eSquare1_x3a0(crate::Square),
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a224_x3a5_x3aclass11_x3amain_x2eCircle1_x3a024_x3a5_x3aclass11_x3amain_x2eSquare1_x3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a224_x3a5_x3aclass11_x3amain_x2eCircle1_x3a024_x3a5_x3aclass11_x3amain_x2eSquare1_x3a0::__SifrUnionVariant_5_x3aclass11_x3amain_x2eCircle1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a224_x3a5_x3aclass11_x3amain_x2eCircle1_x3a024_x3a5_x3aclass11_x3amain_x2eSquare1_x3a0::__SifrUnionVariant_5_x3aclass11_x3amain_x2eSquare1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
}
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a224_x3a5_x3aclass11_x3amain_x2eCircle1_x3a024_x3a5_x3aclass11_x3amain_x2eSquare1_x3a0;
use ::sifr_runtime::SifrInt;

#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        let __sifr_field_init_0: f64 = x;
        let __sifr_field_init_1: f64 = y;
        Self { x: __sifr_field_init_0, y: __sifr_field_init_1 }
    }
}

impl Point {
    fn distance(&self, other: &Point) -> f64 {
        let dx: f64 = self.x - other.x;
        let dy: f64 = self.y - other.y;
        ((dx * dx) + (dy * dy)).powf((0.5_f64) as f64)
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
    fn new(width: f64, height: f64) -> Self {
        let __sifr_field_init_0: f64 = width;
        let __sifr_field_init_1: f64 = height;
        Self { width: __sifr_field_init_0, height: __sifr_field_init_1 }
    }
}

impl Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

impl Rectangle {
    fn perimeter(&self) -> f64 {
        (2.0_f64) * (self.width + self.height)
    }
}

impl ::std::fmt::Display for Rectangle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Rectangle(width={}, height={})", self.width, self.height)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Circle {
    radius: f64,
}

impl Circle {
    fn new(radius: f64) -> Self {
        let __sifr_field_init_0: f64 = radius;
        Self { radius: __sifr_field_init_0 }
    }
}

impl Circle {
}

impl ::std::fmt::Display for Circle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Circle(radius={})", self.radius)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Square {
    side: f64,
}

impl Square {
    fn new(side: f64) -> Self {
        let __sifr_field_init_0: f64 = side;
        Self { side: __sifr_field_init_0 }
    }
}

impl Square {
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
    fn new(r: SifrInt, g: SifrInt, b: SifrInt) -> Self {
        let __sifr_field_init_0: SifrInt = r.clone();
        let __sifr_field_init_1: SifrInt = g.clone();
        let __sifr_field_init_2: SifrInt = b.clone();
        Self { r: __sifr_field_init_0, g: __sifr_field_init_1, b: __sifr_field_init_2 }
    }
}

impl Color {
}

impl ::std::fmt::Display for Color {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Color(r={}, g={}, b={})", self.r, self.g, self.b)
    }
}

fn describe_shape(shape: &__SifrUnion_8_x3asequence5_x3aunion1_x3a224_x3a5_x3aclass11_x3amain_x2eCircle1_x3a024_x3a5_x3aclass11_x3amain_x2eSquare1_x3a0) {
    match shape {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a224_x3a5_x3aclass11_x3amain_x2eCircle1_x3a024_x3a5_x3aclass11_x3amain_x2eSquare1_x3a0::__SifrUnionVariant_5_x3aclass11_x3amain_x2eCircle1_x3a0(shape) => {
            println!("Circle: radius={}", shape.radius);
        },
        __SifrUnion_8_x3asequence5_x3aunion1_x3a224_x3a5_x3aclass11_x3amain_x2eCircle1_x3a024_x3a5_x3aclass11_x3amain_x2eSquare1_x3a0::__SifrUnionVariant_5_x3aclass11_x3amain_x2eSquare1_x3a0(shape) => {
            println!("Square: side={}", shape.side);
        },
    }
}

fn main() {
    println!("=== Basic Class ===");
    let origin: Point = Point::new(0.0_f64, 0.0_f64);
    let target: Point = Point::new(3.0_f64, 4.0_f64);
    let d: f64 = origin.distance(&target);
    println!("Distance from origin to (3,4): {}", d);
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
    describe_shape(&__SifrUnion_8_x3asequence5_x3aunion1_x3a224_x3a5_x3aclass11_x3amain_x2eCircle1_x3a024_x3a5_x3aclass11_x3amain_x2eSquare1_x3a0::__SifrUnionVariant_5_x3aclass11_x3amain_x2eCircle1_x3a0(c.clone()));
    describe_shape(&__SifrUnion_8_x3asequence5_x3aunion1_x3a224_x3a5_x3aclass11_x3amain_x2eCircle1_x3a024_x3a5_x3aclass11_x3amain_x2eSquare1_x3a0::__SifrUnionVariant_5_x3aclass11_x3amain_x2eSquare1_x3a0(s.clone()));
    println!("=== Hash ===");
    let red: Color = Color::new(SifrInt::from_i64(255), SifrInt::from_i64(0), SifrInt::from_i64(0));
    let also_red: Color = Color::new(SifrInt::from_i64(255), SifrInt::from_i64(0), SifrInt::from_i64(0));
    let h1: SifrInt = {
    let mut __sifr_hash = ::std::collections::hash_map::DefaultHasher::new();
    ::std::hash::Hash::hash(&red, &mut __sifr_hash);
    SifrInt::from(::std::hash::Hasher::finish(&__sifr_hash))
};
    let h2: SifrInt = {
    let mut __sifr_hash = ::std::collections::hash_map::DefaultHasher::new();
    ::std::hash::Hash::hash(&also_red, &mut __sifr_hash);
    SifrInt::from(::std::hash::Hasher::finish(&__sifr_hash))
};
    println!("Same color same hash: {}", (&h1 == &h2));
    println!("=== Done ===");
}
