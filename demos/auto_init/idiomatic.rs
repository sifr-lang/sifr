#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Point {
    x: i64,
    y: i64,
}

impl Point {
    fn new(x: i64, y: i64) -> Self {
        return Self { x: x, y: y };
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Point(x={}, y={})", self.x, self.y);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Config {
    debug: bool,
    timeout: i64,
    name: String,
}

impl Config {
    fn new(debug: bool, timeout: i64, name: String) -> Self {
        return Self {
            debug: debug,
            timeout: timeout,
            name: name,
        };
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "Config(debug={}, timeout={}, name={})",
            self.debug, self.timeout, self.name
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Person {
    first_name: String,
    last_name: String,
    age: i64,
}

impl Person {
    fn new(first_name: String, last_name: String, age: i64) -> Self {
        return Self {
            first_name: first_name,
            last_name: last_name,
            age: age,
        };
    }
}

impl std::fmt::Display for Person {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "Person(first_name={}, last_name={}, age={})",
            self.first_name, self.last_name, self.age
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Rectangle {
    width: i64,
    height: i64,
}

impl Rectangle {
    fn new(width: i64, height: i64) -> Self {
        return Self {
            width: width,
            height: height,
        };
    }
    fn area(&self) -> i64 {
        return self.width * self.height;
    }
}

impl std::fmt::Display for Rectangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "{}",
            format!(
                "{}{}{}{}{}",
                "Rectangle(".to_string(),
                format!("{}", self.width),
                "x".to_string(),
                format!("{}", self.height),
                ")".to_string()
            )
        );
    }
}

fn main() {
    let p: Point = Point::new(3 as i64, 4 as i64);
    println!(
        "{}",
        format!("{}{}", "point x = ".to_string(), format!("{}", p.x))
    );
    println!(
        "{}",
        format!("{}{}", "point y = ".to_string(), format!("{}", p.y))
    );
    println!(
        "{}",
        format!("{}{}", "point str = ".to_string(), format!("{}", p))
    );
    let p2: Point = Point::new(3 as i64, 4 as i64);
    let p3: Point = Point::new(5 as i64, 6 as i64);
    println!(
        "{}",
        format!("{}{}", "point eq = ".to_string(), format!("{}", p == p2))
    );
    println!(
        "{}",
        format!("{}{}", "point neq = ".to_string(), format!("{}", p == p3))
    );
    let c1: Config = Config::new(false, 30 as i64, "default".to_string());
    println!(
        "{}",
        format!(
            "{}{}",
            "config debug default = ".to_string(),
            format!("{}", c1.debug)
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "config timeout default = ".to_string(),
            format!("{}", c1.timeout)
        )
    );
    println!(
        "{}",
        format!("{}{}", "config name default = ".to_string(), c1.name)
    );
    let c2: Config = Config::new(true, 60 as i64, "production".to_string());
    println!(
        "{}",
        format!(
            "{}{}",
            "config debug custom = ".to_string(),
            format!("{}", c2.debug)
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "config timeout custom = ".to_string(),
            format!("{}", c2.timeout)
        )
    );
    println!(
        "{}",
        format!("{}{}", "config name custom = ".to_string(), c2.name)
    );
    let person: Person = Person::new("Alice".to_string(), "Smith".to_string(), 30 as i64);
    println!(
        "{}",
        format!("{}{}", "person str = ".to_string(), format!("{}", person))
    );
    let mut r: Rectangle = Rectangle::new(5 as i64, 3 as i64);
    println!(
        "{}",
        format!("{}{}", "rect area = ".to_string(), format!("{}", r.area()))
    );
    println!(
        "{}",
        format!("{}{}", "rect str = ".to_string(), format!("{}", r))
    );
}
