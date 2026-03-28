#[derive(Debug, Clone)]
enum CircleOrSquare {
    Circle(Circle),
    Square(Square),
}

impl std::fmt::Display for CircleOrSquare {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircleOrSquare::Circle(v) => {
                return write!(f, "{:?}", v);
            }
            CircleOrSquare::Square(v) => {
                return write!(f, "{:?}", v);
            }
        }
    }
}

trait Printable {
    fn describe(&self) -> String;
}

#[derive(Debug, Clone)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn new(x: f64, y: f64) -> Self {
        return Self { x: x, y: y };
    }
    fn describe(&self) -> String {
        return format!("A 2D vector at ({}, {})", self.x, self.y);
    }
}

impl std::ops::Add<&Vec2> for &Vec2 {
    type Output = Vec2;
    fn add(self, other: &Vec2) -> Self::Output {
        return Vec2::new(self.x + other.x, self.y + other.y);
    }
}

impl PartialEq for Vec2 {
    fn eq(&self, other: &Vec2) -> bool {
        return ((self.x == other.x) && (self.y == other.y));
    }
}

impl std::fmt::Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", format!("Vec2({}, {})", self.x, self.y));
    }
}

impl Printable for Vec2 {
    fn describe(&self) -> String {
        return Vec2::describe(self);
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Circle {
    radius: f64,
}

impl Circle {
    fn new(radius: f64) -> Self {
        return Self { radius: radius };
    }
    fn describe(&self) -> String {
        return format!("Circle with radius {}", self.radius);
    }
}

impl std::fmt::Display for Circle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Circle(radius={})", self.radius);
    }
}

impl Printable for Circle {
    fn describe(&self) -> String {
        return Circle::describe(self);
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Square {
    side: f64,
}

impl Square {
    fn new(side: f64) -> Self {
        return Self { side: side };
    }
    fn describe(&self) -> String {
        return format!("Square with side {}", self.side);
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Square(side={})", self.side);
    }
}

impl Printable for Square {
    fn describe(&self) -> String {
        return Square::describe(self);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Port(i64);

impl Port {
    fn new(value: i64) -> Self {
        return Self(value);
    }
    fn value(&self) -> i64 {
        return self.0;
    }
}

impl std::fmt::Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.0);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Email(String);

impl Email {
    fn new(value: String) -> Self {
        return Self(value);
    }
    fn value(&self) -> String {
        return self.0.clone();
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.0);
    }
}

fn area(shape: &CircleOrSquare) -> f64 {
    if let CircleOrSquare::Circle(shape) = shape {
        return ((3.14 as f64) * shape.radius) * shape.radius;
    } else {
        if let CircleOrSquare::Square(shape) = shape {
            return shape.side * shape.side;
        } else {
            unreachable!("sifr union narrowing fell through exhaustive branch chain");
        }
    }
}

fn main() {
    let a: Vec2 = Vec2::new(1.0 as f64, 2.0 as f64);
    let b: Vec2 = Vec2::new(3.0 as f64, 4.0 as f64);
    let c: Vec2 = &a + &b;
    println!("{}", c);
    println!("{}", a == Vec2::new(1.0 as f64, 2.0 as f64));
    println!("{}", a == b);
    let circle: Circle = Circle::new(5.0 as f64);
    let square: Square = Square::new(4.0 as f64);
    println!("{}", area(&CircleOrSquare::Circle(circle)));
    println!("{}", area(&CircleOrSquare::Square(square)));
    let port: Port = Port::new(8080 as i64);
    println!("{}", port);
    println!("{}", port.value());
    let email: Email = Email::new("user@example.com".to_string());
    println!("{}", email);
}
