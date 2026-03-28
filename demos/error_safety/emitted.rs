#[derive(Debug, Clone)]
struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for Error {
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
struct DivisionError {
    message: String,
}

impl DivisionError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for DivisionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for DivisionError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AppError {
    message: String,
}

impl AppError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.message);
    }
}

impl std::error::Error for AppError {
}

fn validate_age(age: i64) -> Result<i64, ValueError> {
    if age < (0 as i64) {
        return Err(ValueError::new("age must be positive".to_string()));
    }
    if age > (150 as i64) {
        return Err(ValueError::new("too large".to_string()));
    }
    return Ok(age);
}

fn safe_divide(a: i64, b: i64) -> Result<i64, DivisionError> {
    if b == (0 as i64) {
        return Err(DivisionError::new("division by zero".to_string()));
    }
    return Ok(a / b);
}

fn check_input(x: i64) -> Result<i64, AppError> {
    if x < (0 as i64) {
        return Err(AppError::new("invalid input".to_string()));
    }
    return Ok(x);
}

fn process_age(age: i64) -> Result<i64, ValueError> {
    if age < (0 as i64) {
        return Err(ValueError::new("age must be positive".to_string()));
    }
    if age > (150 as i64) {
        return Err(ValueError::new("too large".to_string()));
    }
    return Ok(age);
}

fn main() {
    println!("=== Built-in Error Classes ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let age: i64 = validate_age(-(5 as i64))?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught ValueError: {}", e.message);
    }
    let __sifr_try_res: Result<(), DivisionError> = (|| {
    let result: i64 = safe_divide(10 as i64, 0 as i64)?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught DivisionError: {}", e.message);
    }
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let n: i64 = ("not_a_number".to_string()).parse::<i64>().map_err(|e| ParseError { message: e.to_string() })?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught ParseError: {}", e.message);
    }
    println!("=== Custom Error Classes ===");
    let __sifr_try_res: Result<(), AppError> = (|| {
    let val: i64 = check_input(-(1 as i64))?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught AppError: {}", e.message);
    }
    println!("=== Exhaustiveness: Specific Except Arms ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let a: i64 = validate_age(-(10 as i64))?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught ValueError: {}", e.message);
    }
    println!("=== Exhaustiveness: Catch-All ===");
    let __sifr_try_res: Result<(), Error> = (|| {
    let b: i64 = (validate_age(200 as i64)).map_err(|__e| Error::new(__e.to_string()))?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught: {}", e.message);
    }
    println!("=== Error Propagation ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let c: i64 = process_age(-(1 as i64))?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("pipeline error: {}", e.message);
    }
    println!("=== Multiple Try/Except ===");
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let parsed: i64 = ("42".to_string()).parse::<i64>().map_err(|e| ParseError { message: e.to_string() })?;
    println!("parsed: {}", parsed);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("parse error: {}", e.message);
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let validated: i64 = validate_age(42 as i64)?;
    println!("validated: {}", validated);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("validation error: {}", e.message);
    }
    let __sifr_try_res: Result<(), DivisionError> = (|| {
    let divided: i64 = safe_divide(42 as i64, 6 as i64)?;
    println!("result: {}", divided);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("division error: {}", e.message);
    }
    println!("demo complete!");
}
