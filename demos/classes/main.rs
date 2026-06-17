// Reference: classes
// Reference: classes
#[derive(Debug, Clone)]
enum CircleOrSquare {
    Circle(Circle),
    Square(Square),
}

impl std::fmt::Display for CircleOrSquare {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircleOrSquare::Circle(v) => write!(f, "{:?}", v),
            CircleOrSquare::Square(v) => write!(f, "{:?}", v),
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Self {
            x: x,
            y: y,
        }
    }

    fn distance(&self, other: &Point) -> f64 {
        let dx: f64 = self.x - other.x;
        let dy: f64 = self.y - other.y;
        return ((dx * dx) + (dy * dy) as f64).powf(0.5_f64 as f64);
    }

}

#[derive(Debug, Clone, PartialEq)]
struct Rectangle {
    width: f64,
    height: f64,
}

impl Rectangle {
    fn new(width: f64, height: f64) -> Self {
        Self {
            width: width,
            height: height,
        }
    }

    fn area(&self) -> f64 {
        return self.width * self.height;
    }

    fn perimeter(&self) -> f64 {
        return 2.0_f64 * (self.width + self.height);
    }

}

#[derive(Debug, Clone, PartialEq)]
struct Circle {
    radius: f64,
}

impl Circle {
    fn new(radius: f64) -> Self {
        Self {
            radius: radius,
        }
    }

}

#[derive(Debug, Clone, PartialEq)]
struct Square {
    side: f64,
}

impl Square {
    fn new(side: f64) -> Self {
        Self {
            side: side,
        }
    }

}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Color {
    r: i64,
    g: i64,
    b: i64,
}

impl Color {
    fn new(r: i64, g: i64, b: i64) -> Self {
        Self {
            r: r,
            g: g,
            b: b,
        }
    }

}

fn describe_shape(shape: CircleOrSquare) {
    match shape {
        CircleOrSquare::Circle(shape) => {
            println!("Circle: radius={}", shape.radius);
        }
        CircleOrSquare::Square(shape) => {
            println!("Square: side={}", shape.side);
        }
    }
}

fn main() {
    println!("{}", "=== Basic Class ===");
    let origin: Point = Point::new(0.0_f64, 0.0_f64);
    let target: Point = Point::new(3.0_f64, 4.0_f64);
    let d: f64 = origin.distance(&target);
    println!("Distance from origin to (3,4): {}", d);
    println!("{}", "=== Methods ===");
    let rect: Rectangle = Rectangle::new(5.0_f64, 3.0_f64);
    println!("Rectangle area: {}", rect.area());
    println!("Rectangle perimeter: {}", rect.perimeter());
    println!("{}", "=== Field Access ===");
    println!("Rectangle width: {}", rect.width);
    println!("Rectangle height: {}", rect.height);
    println!("{}", "=== Union + isinstance ===");
    let c: Circle = Circle::new(10.0_f64);
    let s: Square = Square::new(7.0_f64);
    describe_shape(CircleOrSquare::Circle(c));
    describe_shape(CircleOrSquare::Square(s));
    println!("{}", "=== Hash ===");
    let red: Color = Color::new(255_i64, 0_i64, 0_i64);
    let also_red: Color = Color::new(255_i64, 0_i64, 0_i64);
    let h1: i64 = { use std::hash::{Hash, Hasher}; let mut _h = std::collections::hash_map::DefaultHasher::new(); red.hash(&mut _h); _h.finish() as i64 };
    let h2: i64 = { use std::hash::{Hash, Hasher}; let mut _h = std::collections::hash_map::DefaultHasher::new(); also_red.hash(&mut _h); _h.finish() as i64 };
    println!("Same color same hash: {}", h1 == h2);
    println!("{}", "=== Done ===");
}
