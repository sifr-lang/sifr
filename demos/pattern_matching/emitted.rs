#[derive(Debug, Clone)]
enum IntOrStr {
    Int(i64),
    Str(String),
}

impl std::fmt::Display for IntOrStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntOrStr::Int(v) => {
                return write!(f, "{}", v);
            },
            IntOrStr::Str(v) => {
                return write!(f, "{}", v);
            },
        }
    }
}

#[derive(Debug, Clone)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        return Self { message: message, kind: "Other".to_string() };
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for IOError {
}

fn __io_err(e: std::io::Error) -> IOError {
    let msg = e.to_string();
    let kind = if e.kind() == std::io::ErrorKind::NotFound { "FileNotFound".to_string() } else { if e.kind() == std::io::ErrorKind::PermissionDenied { "PermissionDenied".to_string() } else { if e.kind() == std::io::ErrorKind::AlreadyExists { "FileExists".to_string() } else { "Other".to_string() } } };
    return IOError { message: msg, kind: kind };
}

#[derive(Debug, Clone)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ParseError {
}

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ValueError {
}

#[derive(Debug, Clone)]
struct JSONDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl JSONDecodeError {
    fn new(message: String) -> Self {
        return Self { message: message, line: 0, column: 0 };
    }
}

impl std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for JSONDecodeError {
}

#[derive(Debug, Clone)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl TOMLDecodeError {
    fn new(message: String) -> Self {
        return Self { message: message, line: 0, column: 0 };
    }
}

impl std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for TOMLDecodeError {
}

#[derive(Debug, Clone)]
struct RegexError {
    message: String,
    detail: String,
}

impl RegexError {
    fn new(message: String) -> Self {
        return Self { message: message, detail: String::new() };
    }
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for RegexError {
}

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

fn describe_number(x: i64) -> String {
    match x {
        0 => {
            return "zero".to_string();
        },
        1 => {
            return "one".to_string();
        },
        2 => {
            return "two".to_string();
        },
        _ => {
            return "many".to_string();
        },
    }
}

fn classify_http(method: &String) -> String {
    match method {
        __s if (__s.as_str() == "GET".to_string().as_str()) || (__s.as_str() == "HEAD".to_string().as_str()) => {
            return "read".to_string();
        },
        __s if ((__s.as_str() == "POST".to_string().as_str()) || (__s.as_str() == "PUT".to_string().as_str())) || (__s.as_str() == "PATCH".to_string().as_str()) => {
            return "write".to_string();
        },
        __s if __s.as_str() == "DELETE".to_string().as_str() => {
            return "delete".to_string();
        },
        _ => {
            return "other".to_string();
        },
    }
}

fn classify_score(score: i64) -> String {
    match score {
        n if n >= (90 as i64) => {
            return "A".to_string();
        },
        n if n >= (80 as i64) => {
            return "B".to_string();
        },
        n if n >= (70 as i64) => {
            return "C".to_string();
        },
        n if n >= (60 as i64) => {
            return "D".to_string();
        },
        _ => {
            return "F".to_string();
        },
    }
}

fn describe_optional(x: Option<i64>) -> String {
    match x {
        None => {
            return "nothing".to_string();
        },
        _ => {
            return "something".to_string();
        },
    }
}

fn describe_union(x: &IntOrStr) -> String {
    match x {
        IntOrStr::Int(..) => {
            return "integer".to_string();
        },
        IntOrStr::Str(..) => {
            return "string".to_string();
        },
    }
}

fn make_int_union() -> IntOrStr {
    return IntOrStr::Int(42 as i64);
}

fn make_str_union() -> IntOrStr {
    return IntOrStr::Str("hello".to_string());
}

fn classify_point(p: &Point) -> String {
    match p {
        Point { x: 0, y: 0, .. } => {
            return "origin".to_string();
        },
        Point { x: px, y: 0, .. } => {
            return "on x-axis".to_string();
        },
        Point { x: 0, y: py, .. } => {
            return "on y-axis".to_string();
        },
        Point { x: px, y: py, .. } => {
            return "general".to_string();
        },
    }
}

fn classify_pair(p: (i64, i64)) -> String {
    match p {
        (0, 0) => {
            return "origin".to_string();
        },
        (x, 0) => {
            return "x-axis".to_string();
        },
        (0, y) => {
            return "y-axis".to_string();
        },
        (x, y) => {
            return "general".to_string();
        },
    }
}

fn classify_quadrant(p: &Point) -> String {
    match p {
        Point { x: 0, y: 0, .. } => {
            return "origin".to_string();
        },
        Point { x: px, y: py, .. } if (*px > (0 as i64)) && (*py > (0 as i64)) => {
            return "Q1".to_string();
        },
        Point { x: px, y: py, .. } if (*px < (0 as i64)) && (*py > (0 as i64)) => {
            return "Q2".to_string();
        },
        _ => {
            return "other".to_string();
        },
    }
}

fn main() {
    println!("=== Literal Patterns ===");
    println!("{}", describe_number(0 as i64));
    println!("{}", describe_number(1 as i64));
    println!("{}", describe_number(42 as i64));
    println!("=== OR Patterns ===");
    println!("{}", classify_http(&"GET".to_string()));
    println!("{}", classify_http(&"POST".to_string()));
    println!("{}", classify_http(&"DELETE".to_string()));
    println!("{}", classify_http(&"OPTIONS".to_string()));
    println!("=== Guard Patterns ===");
    println!("{}", classify_score(95 as i64));
    println!("{}", classify_score(85 as i64));
    println!("{}", classify_score(55 as i64));
    println!("=== Optional Matching ===");
    println!("{}", describe_optional(None));
    println!("{}", describe_optional(Some(42 as i64)));
    println!("=== Union Matching ===");
    let a: IntOrStr = make_int_union();
    let b: IntOrStr = make_str_union();
    println!("{}", describe_union(&a));
    println!("{}", describe_union(&b));
    println!("=== Class Destructuring ===");
    let p1: Point = Point::new(0 as i64, 0 as i64);
    let p2: Point = Point::new(3 as i64, 0 as i64);
    let p3: Point = Point::new(0 as i64, 4 as i64);
    let p4: Point = Point::new(3 as i64, 4 as i64);
    println!("{}", classify_point(&p1));
    println!("{}", classify_point(&p2));
    println!("{}", classify_point(&p3));
    println!("{}", classify_point(&p4));
    println!("=== Tuple Patterns ===");
    let t1: (i64, i64) = (0 as i64, 0 as i64);
    let t2: (i64, i64) = (3 as i64, 0 as i64);
    let t3: (i64, i64) = (0 as i64, 4 as i64);
    let t4: (i64, i64) = (3 as i64, 4 as i64);
    println!("{}", classify_pair(t1));
    println!("{}", classify_pair(t2));
    println!("{}", classify_pair(t3));
    println!("{}", classify_pair(t4));
    println!("=== Nested Patterns ===");
    println!("{}", classify_quadrant(&Point::new(0 as i64, 0 as i64)));
    println!("{}", classify_quadrant(&Point::new(3 as i64, 4 as i64)));
    println!("{}", classify_quadrant(&Point::new(-(2 as i64), 5 as i64)));
    println!("{}", classify_quadrant(&Point::new(-(1 as i64), -(1 as i64))));
}
