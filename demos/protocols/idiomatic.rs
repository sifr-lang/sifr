use std::fmt::{self, Display};
use std::ops::Add;

trait Printable {
    fn describe(&self) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vec2({}, {})", self.x, self.y)
    }
}

impl Printable for Vec2 {
    fn describe(&self) -> String {
        format!("A 2D vector at ({}, {})", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Circle {
    radius: f64,
}

impl Printable for Circle {
    fn describe(&self) -> String {
        format!("Circle with radius {}", self.radius)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Square {
    side: f64,
}

impl Printable for Square {
    fn describe(&self) -> String {
        format!("Square with side {}", self.side)
    }
}

enum Shape {
    Circle(Circle),
    Square(Square),
}

fn area(shape: Shape) -> f64 {
    match shape {
        Shape::Circle(circle) => 3.14 * circle.radius * circle.radius,
        Shape::Square(square) => square.side * square.side,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Port(i64);

impl Port {
    fn value(&self) -> i64 {
        self.0
    }
}

impl Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Email(String);

impl Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

fn main() {
    let a = Vec2 { x: 1.0, y: 2.0 };
    let b = Vec2 { x: 3.0, y: 4.0 };
    let c = a + b;
    println!("{c}");
    println!("{}", a == Vec2 { x: 1.0, y: 2.0 });
    println!("{}", a == b);

    let circle = Circle { radius: 5.0 };
    let square = Square { side: 4.0 };
    let _ = circle.describe();
    let _ = square.describe();
    println!("{}", area(Shape::Circle(circle)));
    println!("{}", area(Shape::Square(square)));

    let port = Port(8080);
    println!("{port}");
    println!("{}", port.value());

    let email = Email("user@example.com".to_string());
    println!("{email}");
}
