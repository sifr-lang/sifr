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

pub trait Printable {
    fn describe(&self) -> String;
}

#[derive(Debug, Clone)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn new(x: f64, y: f64) -> Self {
        let __sifr_field_init_0: f64 = x;
        let __sifr_field_init_1: f64 = y;
        Self { x: __sifr_field_init_0, y: __sifr_field_init_1 }
    }
}

impl Vec2 {
    fn describe(&self) -> String {
        format!("A 2D vector at ({}, {})", self.x, self.y)
    }
}

impl ::std::ops::Add<&Vec2> for &Vec2 {
    type Output = Vec2;
    fn add(self, other: &Vec2) -> Self::Output {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl PartialEq for Vec2 {
    fn eq(&self, other: &Vec2) -> bool {
        (((self.x == other.x)) && ((self.y == other.y)))
    }
}

impl ::std::fmt::Display for Vec2 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", format!("Vec2({}, {})", self.x, self.y))
    }
}

impl Printable for Vec2 {
    fn describe(&self) -> String {
        Vec2::describe(self)
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
    fn describe(&self) -> String {
        format!("Circle with radius {}", self.radius)
    }
}

impl ::std::fmt::Display for Circle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Circle(radius={})", self.radius)
    }
}

impl Printable for Circle {
    fn describe(&self) -> String {
        Circle::describe(self)
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
    fn describe(&self) -> String {
        format!("Square with side {}", self.side)
    }
}

impl ::std::fmt::Display for Square {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Square(side={})", self.side)
    }
}

impl Printable for Square {
    fn describe(&self) -> String {
        Square::describe(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Port(SifrInt);

impl Port {
    fn new(value: SifrInt) -> Self {
        Self(value)
    }
    fn value(&self) -> SifrInt {
        self.0.clone()
    }
}

impl ::std::fmt::Display for Port {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Email(String);

impl Email {
    fn new(value: String) -> Self {
        Self(value)
    }
    fn value(&self) -> String {
        self.0.clone()
    }
}

impl ::std::fmt::Display for Email {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn area(shape: &__SifrUnion_8_x3asequence5_x3aunion1_x3a224_x3a5_x3aclass11_x3amain_x2eCircle1_x3a024_x3a5_x3aclass11_x3amain_x2eSquare1_x3a0) -> f64 {
    match shape {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a224_x3a5_x3aclass11_x3amain_x2eCircle1_x3a024_x3a5_x3aclass11_x3amain_x2eSquare1_x3a0::__SifrUnionVariant_5_x3aclass11_x3amain_x2eCircle1_x3a0(shape) => {
            return ((3.14_f64) * shape.radius) * shape.radius;
        },
        __SifrUnion_8_x3asequence5_x3aunion1_x3a224_x3a5_x3aclass11_x3amain_x2eCircle1_x3a024_x3a5_x3aclass11_x3amain_x2eSquare1_x3a0::__SifrUnionVariant_5_x3aclass11_x3amain_x2eSquare1_x3a0(shape) => {
            return shape.side * shape.side;
        },
    }
}

fn main() {
    let a: Vec2 = Vec2::new(1.0_f64, 2.0_f64);
    let b: Vec2 = Vec2::new(3.0_f64, 4.0_f64);
    let c: Vec2 = &a + &b;
    println!("{}", c);
    println!("{}", (a == Vec2::new(1.0_f64, 2.0_f64)));
    println!("{}", (a == b));
    let circle: Circle = Circle::new(5.0_f64);
    let square: Square = Square::new(4.0_f64);
    println!("{}", area(&__SifrUnion_8_x3asequence5_x3aunion1_x3a224_x3a5_x3aclass11_x3amain_x2eCircle1_x3a024_x3a5_x3aclass11_x3amain_x2eSquare1_x3a0::__SifrUnionVariant_5_x3aclass11_x3amain_x2eCircle1_x3a0((circle).clone())));
    println!("{}", area(&__SifrUnion_8_x3asequence5_x3aunion1_x3a224_x3a5_x3aclass11_x3amain_x2eCircle1_x3a024_x3a5_x3aclass11_x3amain_x2eSquare1_x3a0::__SifrUnionVariant_5_x3aclass11_x3amain_x2eSquare1_x3a0((square).clone())));
    let port: Port = Port::new(SifrInt::from_i64(8080));
    println!("{}", port);
    println!("{}", port.value());
    let email: Email = Email::new("user@example.com".to_string());
    println!("{}", email);
}
