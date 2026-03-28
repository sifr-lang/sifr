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

fn main() {
    let base: f64 = 9.0 as f64;
    let root: f64 = (base).sqrt();
    let rounded_down: i64 = (3.9 as f64).floor() as i64;
    let rounded_up: i64 = (3.1 as f64).ceil() as i64;
    let powered: f64 = (2.0 as f64).powf(3.0 as f64);
    let rounded: i64 = (3.6 as f64).round() as i64;
    let angle: f64 = (1.0 as f64).atan2(1.0 as f64);
    let finite: bool = (powered).is_finite();
    println!("root = {}", root);
    assert!(format!("{}", format!("root = {}", root)) == "root = 3".to_string());
    println!("rounded_down = {}", rounded_down);
    assert!(format!("{}", format!("rounded_down = {}", rounded_down)) == "rounded_down = 3".to_string());
    println!("rounded_up = {}", rounded_up);
    assert!(format!("{}", format!("rounded_up = {}", rounded_up)) == "rounded_up = 4".to_string());
    println!("powered = {}", powered);
    assert!(format!("{}", format!("powered = {}", powered)) == "powered = 8".to_string());
    println!("rounded = {}", rounded);
    assert!(format!("{}", format!("rounded = {}", rounded)) == "rounded = 4".to_string());
    println!("angle_positive = {}", angle > (0.0 as f64));
    assert!(format!("{}", format!("angle_positive = {}", angle > (0.0 as f64))) == "angle_positive = true".to_string());
    println!("finite = {}", finite);
    assert!(format!("{}", format!("finite = {}", finite)) == "finite = true".to_string());
}
