// src/main.rs
mod __sifr_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct DivisionError {
        pub message: String,
    }
    impl DivisionError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for DivisionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for DivisionError {}
}
pub use __sifr_project_nominals::DivisionError;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ParseError {
    message: String,
}
impl ParseError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for ParseError {}
#[derive(Clone, PartialEq, Eq, Hash)]
struct ValidationError {
    message: String,
}
impl ValidationError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ValidationError {}
impl ::std::fmt::Debug for ValidationError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("ValidationError").field("message", &self.message).finish()
    }
}
impl ::std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for ValidationError {}
fn validate_range(x: i64, lo: i64, hi: i64) -> Result<i64, ValidationError> {
    if x < lo {
        return Err(ValidationError::new(format!("value out of range: {}", x)));
    }
    if x > hi {
        return Err(ValidationError::new(format!("value out of range: {}", x)));
    }
    Ok(x)
}
fn safe_divide(a: i64, b: i64) -> Result<i64, DivisionError> {
    if b == (0_i64) {
        return Err(DivisionError::new("division by zero".to_string()));
    }
    Ok(a / b)
}
fn main() {
    println!("=== Result Type & Fallible Conversions ===");
    let __sifr_try_res: Result<(), ParseError> = (|| {
        let n: i64 = ("42".to_string())
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        println!("parsed: {}", n);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("parse failed: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), ParseError> = (|| {
        let n2: i64 = ("not_a_number".to_string())
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        println!("parsed: {}", n2);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("parse failed (expected): {}", e.message.clone());
    }
    println!("=== Custom Error Types ===");
    let __sifr_try_res: Result<(), ValidationError> = (|| {
        let v: i64 = validate_range(50_i64, 0_i64, 100_i64)?;
        println!("validated: {}", v);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), ValidationError> = (|| {
        let v2: i64 = validate_range(-(5_i64), 0_i64, 100_i64)?;
        println!("validated: {}", v2);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught: {}", e.message.clone());
    }
    println!("=== Try/Except with Auto-Unwrap ===");
    let __sifr_try_res: Result<(), ValidationError> = (|| {
        let a: i64 = validate_range(100_i64, 0_i64, 200_i64)?;
        println!("result: {}", a);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error handled: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), ValidationError> = (|| {
        let b: i64 = validate_range(999_i64, 0_i64, 200_i64)?;
        println!("result: {}", b);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error handled: {}", e.message.clone());
    }
    println!("=== Infallible Conversions ===");
    let x1: i64 = (3.7_f64) as i64;
    println!("int(3.7) = {}", x1);
    let x2: f64 = (5_i64) as f64;
    println!("float(5) = {}", x2);
    let x3: String = format!("{}", 42_i64);
    println!("str(42) = {}", x3);
    let x4: bool = (1_i64) != 0;
    println!("bool(1) = {}", x4);
    println!("=== Raise in Result Functions ===");
    let __sifr_try_res: Result<(), DivisionError> = (|| {
        let d1: i64 = safe_divide(10_i64, 3_i64)?;
        println!("divide(10, 3) = {}", d1);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("divide error: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), DivisionError> = (|| {
        let d2: i64 = safe_divide(10_i64, 0_i64)?;
        println!("divide(10, 0) = {}", d2);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("divide(10, 0) error: {}", e.message.clone());
    }
    println!("=== Assert Statement ===");
    println!("all assertions passed");
    println!("=== Explicit Discard ===");
    let _ = safe_divide(10_i64, 2_i64);
    println!("result discarded safely");
    println!("demo complete!");
}
