#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ValidationError {
    message: String,
}

impl ValidationError {
    fn new(message: String) -> Self {
        Self {
            message,
        }
    }

}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValidationError {}

fn validate_range(x: i64, lo: i64, hi: i64) -> Result<i64, ValidationError> {
    if x < lo {
        return Err(ValidationError::new(format!("value out of range: {}", x)));
    }
    if x > hi {
        return Err(ValidationError::new(format!("value out of range: {}", x)));
    }
    return Ok(x);
}

fn safe_divide(a: i64, b: i64) -> Result<i64, String> {
    if b == 0_i64 {
        return Err("division by zero".to_string());
    }
    return Ok(a / b);
}

fn main() {
    println!("{}", "=== Result Type & Fallible Conversions ===".to_string());
    match (|| -> Result<(), String> {
        let mut n: i64 = "42".to_string().parse::<i64>().map_err(|e| e.to_string())?;
        println!("{}", format!("parsed: {}", n));
        Ok(())
    })() {
        Ok(()) => {}
        Err(e) => {
            println!("{}", format!("parse failed: {}", e));
        }
    }
    match (|| -> Result<(), String> {
        let mut n2: i64 = "not_a_number".to_string().parse::<i64>().map_err(|e| e.to_string())?;
        println!("{}", format!("parsed: {}", n2));
        Ok(())
    })() {
        Ok(()) => {}
        Err(e) => {
            println!("{}", format!("parse failed (expected): {}", e));
        }
    }
    println!("{}", "=== Custom Error Types ===".to_string());
    match (|| -> Result<(), ValidationError> {
        let mut v: i64 = validate_range(50_i64, 0_i64, 100_i64)?;
        println!("{}", format!("validated: {}", v));
        Ok(())
    })() {
        Ok(()) => {}
        Err(e) => {
            println!("{}", format!("caught: {}", e.message));
        }
    }
    match (|| -> Result<(), ValidationError> {
        let mut v2: i64 = validate_range(-5_i64, 0_i64, 100_i64)?;
        println!("{}", format!("validated: {}", v2));
        Ok(())
    })() {
        Ok(()) => {}
        Err(e) => {
            println!("{}", format!("caught: {}", e.message));
        }
    }
    println!("{}", "=== Try/Except with Auto-Unwrap ===".to_string());
    match (|| -> Result<(), ValidationError> {
        let mut a: i64 = validate_range(100_i64, 0_i64, 200_i64)?;
        println!("{}", format!("result: {}", a));
        Ok(())
    })() {
        Ok(()) => {}
        Err(e) => {
            println!("{}", format!("error handled: {}", e.message));
        }
    }
    match (|| -> Result<(), ValidationError> {
        let mut b: i64 = validate_range(999_i64, 0_i64, 200_i64)?;
        println!("{}", format!("result: {}", b));
        Ok(())
    })() {
        Ok(()) => {}
        Err(e) => {
            println!("{}", format!("error handled: {}", e.message));
        }
    }
    println!("{}", "=== Infallible Conversions ===".to_string());
    let mut x1: i64 = 3.7_f64 as i64;
    println!("{}", format!("int(3.7) = {}", x1));
    let mut x2: f64 = 5_i64 as f64;
    println!("{}", format!("float(5) = {}", x2));
    let mut x3: String = format!("{}", 42_i64);
    println!("{}", format!("str(42) = {}", x3));
    let mut x4: bool = 1_i64 != 0;
    println!("{}", format!("bool(1) = {}", x4));
    println!("{}", "=== Raise in Result Functions ===".to_string());
    match (|| -> Result<(), String> {
        let mut d1: i64 = safe_divide(10_i64, 3_i64)?;
        println!("{}", format!("divide(10, 3) = {}", d1));
        Ok(())
    })() {
        Ok(()) => {}
        Err(e) => {
            println!("{}", format!("divide error: {}", e));
        }
    }
    match (|| -> Result<(), String> {
        let mut d2: i64 = safe_divide(10_i64, 0_i64)?;
        println!("{}", format!("divide(10, 0) = {}", d2));
        Ok(())
    })() {
        Ok(()) => {}
        Err(e) => {
            println!("{}", format!("divide(10, 0) error: {}", e));
        }
    }
    println!("{}", "=== Assert Statement ===".to_string());
    assert!(1_i64 + 1_i64 == 2_i64);
    assert!(true);
    println!("{}", "all assertions passed".to_string());
    println!("{}", "=== Explicit Discard ===".to_string());
    let _: Result<i64, String> = safe_divide(10_i64, 2_i64);
    println!("{}", "result discarded safely".to_string());
    println!("{}", "demo complete!".to_string());
}
