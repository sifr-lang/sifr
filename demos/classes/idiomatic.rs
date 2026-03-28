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

#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        return Self { x: x, y: y };
    }
    fn distance(&self, other: &Point) -> f64 {
        let dx: f64 = self.x - other.x;
        let dy: f64 = self.y - other.y;
        return ((dx * dx) + (dy * dy)).powf((0.5 as f64) as f64);
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Point(x={}, y={})", self.x, self.y);
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Rectangle {
    width: f64,
    height: f64,
}

impl Rectangle {
    fn new(width: f64, height: f64) -> Self {
        return Self {
            width: width,
            height: height,
        };
    }
    fn area(&self) -> f64 {
        return self.width * self.height;
    }
    fn perimeter(&self) -> f64 {
        return (2.0 as f64) * (self.width + self.height);
    }
}

impl std::fmt::Display for Rectangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Rectangle(width={}, height={})", self.width, self.height);
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
}

impl std::fmt::Display for Circle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Circle(radius={})", self.radius);
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
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Square(side={})", self.side);
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
        return Self { r: r, g: g, b: b };
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Color(r={}, g={}, b={})", self.r, self.g, self.b);
    }
}

fn describe_shape(shape: &CircleOrSquare) {
    if let CircleOrSquare::Circle(shape) = shape {
        println!("Circle: radius={}", shape.radius);
    } else {
        if let CircleOrSquare::Square(shape) = shape {
            println!("Square: side={}", shape.side);
        } else {
            unreachable!("sifr union narrowing fell through exhaustive branch chain");
        }
    }
}

fn main() {
    println!("=== Basic Class ===");
    let mut origin: Point = Point::new(0.0 as f64, 0.0 as f64);
    let target: Point = Point::new(3.0 as f64, 4.0 as f64);
    let d: f64 = origin.distance(&target);
    println!("Distance from origin to (3,4): {}", d);
    println!("=== Methods ===");
    let mut rect: Rectangle = Rectangle::new(5.0 as f64, 3.0 as f64);
    println!("Rectangle area: {}", rect.area());
    println!("Rectangle perimeter: {}", rect.perimeter());
    println!("=== Field Access ===");
    println!("Rectangle width: {}", rect.width);
    println!("Rectangle height: {}", rect.height);
    println!("=== Union + isinstance ===");
    let c: Circle = Circle::new(10.0 as f64);
    let s: Square = Square::new(7.0 as f64);
    describe_shape(&CircleOrSquare::Circle(c));
    describe_shape(&CircleOrSquare::Square(s));
    println!("=== Hash ===");
    let red: Color = Color::new(255 as i64, 0 as i64, 0 as i64);
    let also_red: Color = Color::new(255 as i64, 0 as i64, 0 as i64);
    let h1: i64 = {
        let mut __sifr_hash = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&red, &mut __sifr_hash);
        std::hash::Hasher::finish(&__sifr_hash) as i64
    };
    let h2: i64 = {
        let mut __sifr_hash = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&also_red, &mut __sifr_hash);
        std::hash::Hasher::finish(&__sifr_hash) as i64
    };
    println!("Same color same hash: {}", h1 == h2);
    println!("=== Done ===");
}
