// src/main.rs
mod __sifr_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ParseError {
        pub message: String,
    }
    impl ParseError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ParseError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ParseError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
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
pub use __sifr_project_nominals::ParseError;
pub use __sifr_project_nominals::ValueError;
use ::sifr_runtime::SifrInt;
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
fn validate_range(
    x: SifrInt,
    lo: SifrInt,
    hi: SifrInt,
) -> Result<SifrInt, ValidationError> {
    if (&x < &lo) {
        return Err(ValidationError::new(format!("value out of range: {}", x)));
    }
    if (&x > &hi) {
        return Err(ValidationError::new(format!("value out of range: {}", x)));
    }
    Ok(x.clone())
}
fn safe_divide(a: SifrInt, b: SifrInt) -> Result<SifrInt, DivisionError> {
    if (&b == &SifrInt::from_i64(0)) {
        return Err(DivisionError::new("division by zero".to_string()));
    }
    Ok(a.floor_div_known_nonzero(&b))
}
fn main() {
    println!("=== Result Type & Fallible Conversions ===");
    let __sifr_try_res: Result<(), ParseError> = (|| {
        let n: SifrInt = SifrInt::parse_decimal(
                &("42".to_string()),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
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
        let n2: SifrInt = SifrInt::parse_decimal(
                &("not_a_number".to_string()),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
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
        let v: SifrInt = validate_range(
            SifrInt::from_i64(50),
            SifrInt::from_i64(0),
            SifrInt::from_i64(100),
        )?;
        println!("validated: {}", v);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), ValidationError> = (|| {
        let v2: SifrInt = validate_range(
            -&SifrInt::from_i64(5),
            SifrInt::from_i64(0),
            SifrInt::from_i64(100),
        )?;
        println!("validated: {}", v2);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught: {}", e.message.clone());
    }
    println!("=== Try/Except with Auto-Unwrap ===");
    let __sifr_try_res: Result<(), ValidationError> = (|| {
        let a: SifrInt = validate_range(
            SifrInt::from_i64(100),
            SifrInt::from_i64(0),
            SifrInt::from_i64(200),
        )?;
        println!("result: {}", a);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error handled: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), ValidationError> = (|| {
        let b: SifrInt = validate_range(
            SifrInt::from_i64(999),
            SifrInt::from_i64(0),
            SifrInt::from_i64(200),
        )?;
        println!("result: {}", b);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error handled: {}", e.message.clone());
    }
    println!("=== Explicit Conversions ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let x1: SifrInt = SifrInt::from_f64_trunc(3.7_f64)
            .ok_or_else(|| ValueError {
                message: "cannot convert non-finite float to int".to_string(),
            })?;
        println!("int(3.7) = {}", x1);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("conversion error: {}", e.message.clone());
    }
    let x2: f64 = 5.0;
    println!("float(5) = {}", x2);
    let x3: String = format!("{}", SifrInt::from_i64(42));
    println!("str(42) = {}", x3);
    let x4: bool = &SifrInt::from_i64(1) != &0;
    println!("bool(1) = {}", x4);
    println!("=== Raise in Result Functions ===");
    let __sifr_try_res: Result<(), DivisionError> = (|| {
        let d1: SifrInt = safe_divide(SifrInt::from_i64(10), SifrInt::from_i64(3))?;
        println!("divide(10, 3) = {}", d1);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("divide error: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), DivisionError> = (|| {
        let d2: SifrInt = safe_divide(SifrInt::from_i64(10), SifrInt::from_i64(0))?;
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
    let _ = safe_divide(SifrInt::from_i64(10), SifrInt::from_i64(2));
    println!("result discarded safely");
    println!("demo complete!");
}
