use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn distance(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Rectangle {
    width: f64,
    height: f64,
}

impl Rectangle {
    fn area(self) -> f64 {
        self.width * self.height
    }

    fn perimeter(self) -> f64 {
        2.0 * (self.width + self.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Circle {
    radius: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Square {
    side: f64,
}

enum Shape {
    Circle(Circle),
    Square(Square),
}

fn describe_shape(shape: Shape) {
    match shape {
        Shape::Circle(circle) => println!("Circle: radius={}", circle.radius),
        Shape::Square(square) => println!("Square: side={}", square.side),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Color {
    r: i64,
    g: i64,
    b: i64,
}

fn stable_hash(value: impl Hash) -> i64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish() as i64
}

fn main() {
    println!("=== Basic Class ===");
    let origin = Point { x: 0.0, y: 0.0 };
    let target = Point { x: 3.0, y: 4.0 };
    println!("Distance from origin to (3,4): {}", origin.distance(target));

    println!("=== Methods ===");
    let rect = Rectangle {
        width: 5.0,
        height: 3.0,
    };
    println!("Rectangle area: {}", rect.area());
    println!("Rectangle perimeter: {}", rect.perimeter());

    println!("=== Field Access ===");
    println!("Rectangle width: {}", rect.width);
    println!("Rectangle height: {}", rect.height);

    println!("=== Union + isinstance ===");
    describe_shape(Shape::Circle(Circle { radius: 10.0 }));
    describe_shape(Shape::Square(Square { side: 7.0 }));

    println!("=== Hash ===");
    let red = Color { r: 255, g: 0, b: 0 };
    let also_red = Color { r: 255, g: 0, b: 0 };
    println!(
        "Same color same hash: {}",
        stable_hash(red) == stable_hash(also_red)
    );

    println!("=== Done ===");
}
