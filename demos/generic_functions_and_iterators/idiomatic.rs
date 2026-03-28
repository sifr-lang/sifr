#[derive(Debug, Clone)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            kind: "Other".to_string(),
        };
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for IOError {}

fn __io_err(e: std::io::Error) -> IOError {
    let msg = e.to_string();
    let kind = if e.kind() == std::io::ErrorKind::NotFound {
        "FileNotFound".to_string()
    } else {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            "PermissionDenied".to_string()
        } else {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                "FileExists".to_string()
            } else {
                "Other".to_string()
            }
        }
    };
    return IOError {
        message: msg,
        kind: kind,
    };
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

impl std::error::Error for ParseError {}

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

impl std::error::Error for ValueError {}

#[derive(Debug, Clone)]
struct JSONDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl JSONDecodeError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            line: 0,
            column: 0,
        };
    }
}

impl std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for JSONDecodeError {}

#[derive(Debug, Clone)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl TOMLDecodeError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            line: 0,
            column: 0,
        };
    }
}

impl std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for TOMLDecodeError {}

#[derive(Debug, Clone)]
struct RegexError {
    message: String,
    detail: String,
}

impl RegexError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            detail: String::new(),
        };
    }
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for RegexError {}

#[derive(Debug, Clone, PartialEq)]
struct Container<T: Clone + std::fmt::Display + PartialOrd> {
    value: T,
}

impl<T: Clone + std::fmt::Display + PartialOrd> Container<T> {
    fn new(value: T) -> Self {
        return Self { value: value };
    }
    fn get(&self) -> T {
        return self.value.clone();
    }
}

trait Printable {
    fn display(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct User {
    name: String,
}

impl User {
    fn new(name: String) -> Self {
        return Self { name: name };
    }
    fn display(&self) -> String {
        return format!("User({})", self.name.clone());
    }
}

impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "User(name={})", self.name);
    }
}

impl Printable for User {
    fn display(&self) -> String {
        return User::display(self);
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Product {
    title: String,
    price: f64,
}

impl Product {
    fn new(title: String, price: f64) -> Self {
        return Self {
            title: title,
            price: price,
        };
    }
    fn display(&self) -> String {
        return format!("Product({}, ${})", self.title.clone(), self.price);
    }
}

impl std::fmt::Display for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Product(title={}, price={})", self.title, self.price);
    }
}

impl Printable for Product {
    fn display(&self) -> String {
        return Product::display(self);
    }
}

fn identity<T: Clone + std::fmt::Display + PartialOrd + 'static>(x: &T) -> T {
    return x.clone();
}

fn repeat<T: Clone + std::fmt::Display + PartialOrd + 'static>(x: &T, n: i64) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    let mut i: i64 = 0 as i64;
    while i < n {
        result.push(x.clone());
        i = i + (1 as i64);
    }
    return result;
}

fn show(item: Box<dyn Printable>) {
    println!("{}", item.display());
}

fn main() {
    println!("=== PEP 695 Generic Functions ===");
    println!("{}", identity(&(42 as i64)));
    println!("{}", identity(&"hello".to_string()));
    println!("{:?}", repeat(&"x".to_string(), 3 as i64));
    println!("=== PEP 695 Generic Classes ===");
    let mut c = Container::new(99 as i64);
    println!("{}", c.get());
    let mut c2 = Container::new("wrapped".to_string());
    println!("{}", c2.get());
    println!("=== Protocol Method Dispatch ===");
    let u: User = User::new("Alice".to_string());
    let pr: Product = Product::new("Widget".to_string(), 9.99 as f64);
    show(Box::new(u));
    show(Box::new(pr));
    println!("=== Multi-Generator Comprehensions ===");
    let matrix: Vec<Vec<i64>> = vec![
        vec![1 as i64, 2 as i64, 3 as i64],
        vec![4 as i64, 5 as i64, 6 as i64],
        vec![7 as i64, 8 as i64, 9 as i64],
    ];
    let flat: Vec<i64> = {
        let mut __sifr_list_comp = vec![];
        for row in matrix.iter().cloned() {
            for x in row.iter().copied() {
                __sifr_list_comp.push(x);
            }
        }
        __sifr_list_comp
    };
    println!("{:?}", flat);
    println!("=== Stdlib Math Functions ===");
    println!("{}", (1.0 as f64).ln());
    println!("{}", (0.0 as f64).sin());
    println!("{}", (0.0 as f64).cos());
    println!("{}", (-(42.0 as f64)).abs());
    println!("{}", (2.0 as f64).powf(10.0 as f64));
    println!("{}", (3.14 as f64).round() as i64);
}
