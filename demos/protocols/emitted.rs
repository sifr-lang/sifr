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
pub trait Printable {}
#[derive(Debug, Clone)]
struct Vec2 {
    x: f64,
    y: f64,
}
impl Vec2 {
    const fn new(x: f64, y: f64) -> Self {
        let sifr_generated_field_value_af63f54c86021707_78: f64 = x;
        let sifr_generated_field_value_af63f44c86021554_79: f64 = y;
        Self {
            x: sifr_generated_field_value_af63f54c86021707_78,
            y: sifr_generated_field_value_af63f44c86021554_79,
        }
    }
}
impl ::std::ops::Add<&Vec2> for &Vec2 {
    type Output = Vec2;
    fn add(self, other: &Vec2) -> Self::Output {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}
impl PartialEq for Vec2 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl ::std::fmt::Display for Vec2 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Vec2({}, {})", self.x, self.y)
    }
}
impl Printable for Vec2 {}
#[derive(Debug, Clone, PartialEq)]
struct Circle {
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
impl Printable for Circle {}
#[derive(Debug, Clone, PartialEq)]
struct Square {
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
impl Printable for Square {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Port(SifrInt);
impl Port {
    const fn new(value: SifrInt) -> Self {
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
    const fn new(value: String) -> Self {
        Self(value)
    }
}
impl ::std::fmt::Display for Email {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
#[expect(
    clippy::approx_constant,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn area(
    shape: &SifrGeneratedUnion8X3asequence5X3aunion1X3a224X3a5X3aclass11X3amainX2eCircle1X3a024X3a5X3aclass11X3amainX2eSquare1X3a0,
) -> f64 {
    match shape {
        SifrGeneratedUnion8X3asequence5X3aunion1X3a224X3a5X3aclass11X3amainX2eCircle1X3a024X3a5X3aclass11X3amainX2eSquare1X3a0::SifrGeneratedUnionVariant5X3aclass11X3amainX2eCircle1X3a0(
            shape,
        ) => 3.14_f64 * shape.radius * shape.radius,
        SifrGeneratedUnion8X3asequence5X3aunion1X3a224X3a5X3aclass11X3amainX2eCircle1X3a024X3a5X3aclass11X3amainX2eSquare1X3a0::SifrGeneratedUnionVariant5X3aclass11X3amainX2eSquare1X3a0(
            shape,
        ) => shape.side * shape.side,
    }
}
fn main() {
    let a: Vec2 = Vec2::new(1.0_f64, 2.0_f64);
    let b: Vec2 = Vec2::new(3.0_f64, 4.0_f64);
    let c: Vec2 = ::std::ops::Add::add(&a, &b);
    println!("{c}");
    println!("{}", a == Vec2::new(1.0_f64, 2.0_f64));
    println!("{}", a == b);
    let circle: Circle = Circle::new(5.0_f64);
    let square: Square = Square::new(4.0_f64);
    println!(
        "{}", area(&
        SifrGeneratedUnion8X3asequence5X3aunion1X3a224X3a5X3aclass11X3amainX2eCircle1X3a024X3a5X3aclass11X3amainX2eSquare1X3a0::SifrGeneratedUnionVariant5X3aclass11X3amainX2eCircle1X3a0(circle))
    );
    println!(
        "{}", area(&
        SifrGeneratedUnion8X3asequence5X3aunion1X3a224X3a5X3aclass11X3amainX2eCircle1X3a024X3a5X3aclass11X3amainX2eSquare1X3a0::SifrGeneratedUnionVariant5X3aclass11X3amainX2eSquare1X3a0(square))
    );
    let port: Port = Port::new(SifrInt::from_i64(8080));
    println!("{port}");
    println!("{}", port.value());
    let email: Email = Email::new("user@example.com".to_string());
    println!("{email}");
}
