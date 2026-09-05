use std::fmt::{self, Display};

#[derive(Clone, PartialEq, Eq)]
struct Point {
    x: i64,
    y: i64,
}

impl Point {
    fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Point(x={}, y={})", self.x, self.y)
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Config {
    debug: bool,
    timeout: i64,
    name: String,
}

impl Config {
    fn new(debug: bool, timeout: i64, name: impl Into<String>) -> Self {
        Self {
            debug,
            timeout,
            name: name.into(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(false, 30, "default")
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Person {
    first_name: String,
    last_name: String,
    age: i64,
}

impl Person {
    fn new(first_name: impl Into<String>, last_name: impl Into<String>, age: i64) -> Self {
        Self {
            first_name: first_name.into(),
            last_name: last_name.into(),
            age,
        }
    }
}

impl Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Person(first_name={}, last_name={}, age={})",
            self.first_name, self.last_name, self.age
        )
    }
}

struct Rectangle {
    width: i64,
    height: i64,
}

impl Rectangle {
    fn new(width: i64, height: i64) -> Self {
        Self { width, height }
    }

    fn area(&self) -> i64 {
        self.width * self.height
    }
}

impl Display for Rectangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rectangle({}x{})", self.width, self.height)
    }
}

fn main() {
    let p = Point::new(3, 4);
    println!("point x = {}", p.x);
    println!("point y = {}", p.y);
    println!("point str = {p}");

    let equal_point = Point::new(3, 4);
    let different_point = Point::new(5, 6);
    println!("point eq = {}", p == equal_point);
    println!("point neq = {}", p == different_point);

    let c1 = Config::default();
    println!("config debug default = {}", c1.debug);
    println!("config timeout default = {}", c1.timeout);
    println!("config name default = {}", c1.name);

    let c2 = Config::new(true, 60, "production");
    println!("config debug custom = {}", c2.debug);
    println!("config timeout custom = {}", c2.timeout);
    println!("config name custom = {}", c2.name);

    let person = Person::new("Alice", "Smith", 30);
    println!("person str = {person}");

    let rect = Rectangle::new(5, 3);
    println!("rect area = {}", rect.area());
    println!("rect str = {rect}");
}
