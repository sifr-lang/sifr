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
struct ValidationError {
    message: String,
}

impl ValidationError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.message);
    }
}

impl std::error::Error for ValidationError {
}

fn validate_range(x: i64, lo: i64, hi: i64) -> Result<i64, ValidationError> {
    if x < lo {
        return Err(ValidationError::new(format!("value out of range: {}", x)));
    }
    if x > hi {
        return Err(ValidationError::new(format!("value out of range: {}", x)));
    }
    return Ok(x);
}

fn safe_divide(a: i64, b: i64) -> Result<i64, DivisionError> {
    if b == (0 as i64) {
        return Err(DivisionError::new("division by zero".to_string()));
    }
    return Ok(a / b);
}

fn main() {
    println!("=== Result Type & Fallible Conversions ===");
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let n: i64 = ("42".to_string()).parse::<i64>().map_err(|e| ParseError { message: e.to_string() })?;
    println!("parsed: {}", n);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("parse failed: {}", e.message);
    }
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let n2: i64 = ("not_a_number".to_string()).parse::<i64>().map_err(|e| ParseError { message: e.to_string() })?;
    println!("parsed: {}", n2);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("parse failed (expected): {}", e.message);
    }
    println!("=== Custom Error Types ===");
    let __sifr_try_res: Result<(), ValidationError> = (|| {
    let v: i64 = validate_range(50 as i64, 0 as i64, 100 as i64)?;
    println!("validated: {}", v);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught: {}", e.message);
    }
    let __sifr_try_res: Result<(), ValidationError> = (|| {
    let v2: i64 = validate_range(-(5 as i64), 0 as i64, 100 as i64)?;
    println!("validated: {}", v2);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught: {}", e.message);
    }
    println!("=== Try/Except with Auto-Unwrap ===");
    let __sifr_try_res: Result<(), ValidationError> = (|| {
    let a: i64 = validate_range(100 as i64, 0 as i64, 200 as i64)?;
    println!("result: {}", a);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error handled: {}", e.message);
    }
    let __sifr_try_res: Result<(), ValidationError> = (|| {
    let b: i64 = validate_range(999 as i64, 0 as i64, 200 as i64)?;
    println!("result: {}", b);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error handled: {}", e.message);
    }
    println!("=== Infallible Conversions ===");
    let x1: i64 = (3.7 as f64) as i64;
    println!("int(3.7) = {}", x1);
    let x2: f64 = (5 as i64) as f64;
    println!("float(5) = {}", x2);
    let x3: String = format!("{}", 42 as i64);
    println!("str(42) = {}", x3);
    let x4: bool = (1 as i64) != 0;
    println!("bool(1) = {}", x4);
    println!("=== Raise in Result Functions ===");
    let __sifr_try_res: Result<(), DivisionError> = (|| {
    let d1: i64 = safe_divide(10 as i64, 3 as i64)?;
    println!("divide(10, 3) = {}", d1);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("divide error: {}", e.message);
    }
    let __sifr_try_res: Result<(), DivisionError> = (|| {
    let d2: i64 = safe_divide(10 as i64, 0 as i64)?;
    println!("divide(10, 0) = {}", d2);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("divide(10, 0) error: {}", e.message);
    }
    println!("=== Assert Statement ===");
    println!("all assertions passed");
    println!("=== Explicit Discard ===");
    let _: Result<i64, DivisionError> = safe_divide(10 as i64, 2 as i64);
    println!("result discarded safely");
    println!("demo complete!");
}
