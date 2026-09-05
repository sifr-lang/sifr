// src/main.rs
mod sifr_generated_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ParseError {
        pub message: String,
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
        #[must_use]
        pub const fn new(message: String) -> Self {
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
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::DivisionError;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::ValueError;
#[derive(Clone, PartialEq, Eq, Hash)]
struct ValidationError {
    message: String,
}
impl ValidationError {
    const fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Debug for ValidationError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("ValidationError")
            .field("message", &self.message)
            .finish()
    }
}
impl ::std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for ValidationError {}
fn validate_range(x: SifrInt, lo: &SifrInt, hi: &SifrInt) -> Result<SifrInt, ValidationError> {
    if &x < lo {
        return Err(ValidationError::new(format!("value out of range: {x}")));
    }
    if &x > hi {
        return Err(ValidationError::new(format!("value out of range: {x}")));
    }
    Ok(x)
}
fn safe_divide(a: &SifrInt, b: &SifrInt) -> Result<SifrInt, DivisionError> {
    if b == &SifrInt::from_i64(0) {
        return Err(DivisionError::new("division by zero".to_string()));
    }
    Ok(a.floor_div_known_nonzero(b))
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    println!("=== Result Type & Fallible Conversions ===");
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let n: SifrInt = SifrInt::parse_decimal("42", ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS)
            .map_err(|e| ParseError {
            message: e.to_string(),
        })?;
        println!("parsed: {n}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("parse failed: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let n2: SifrInt =
            SifrInt::parse_decimal("not_a_number", ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS)
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
        println!("parsed: {n2}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("parse failed (expected): {}", e.message);
    }
    println!("=== Custom Error Types ===");
    let sifr_generated_try_res: Result<(), ValidationError> = (|| {
        let v: SifrInt = validate_range(
            SifrInt::from_i64(50),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(100),
        )?;
        println!("validated: {v}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), ValidationError> = (|| {
        let v2: SifrInt = validate_range(
            ::std::ops::Neg::neg(&SifrInt::from_i64(5)),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(100),
        )?;
        println!("validated: {v2}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught: {}", e.message);
    }
    println!("=== Try/Except with Auto-Unwrap ===");
    let sifr_generated_try_res: Result<(), ValidationError> = (|| {
        let a: SifrInt = validate_range(
            SifrInt::from_i64(100),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(200),
        )?;
        println!("result: {a}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("error handled: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), ValidationError> = (|| {
        let b: SifrInt = validate_range(
            SifrInt::from_i64(999),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(200),
        )?;
        println!("result: {b}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("error handled: {}", e.message);
    }
    println!("=== Explicit Conversions ===");
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let x1: SifrInt = SifrInt::from_f64_trunc(3.7_f64).ok_or_else(|| ValueError {
            message: "cannot convert non-finite float to int".to_string(),
        })?;
        println!("int(3.7) = {x1}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("conversion error: {}", e.message);
    }
    let x2: f64 = 5.0;
    println!("float(5) = {x2}");
    let x3: String = SifrInt::from_i64(42).to_string();
    println!("str(42) = {x3}");
    let x4: bool = SifrInt::from_i64(1) != 0;
    println!("bool(1) = {x4}");
    println!("=== Raise in Result Functions ===");
    let sifr_generated_try_res: Result<(), DivisionError> = (|| {
        let d1: SifrInt = safe_divide(&SifrInt::from_i64(10), &SifrInt::from_i64(3))?;
        println!("divide(10, 3) = {d1}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("divide error: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), DivisionError> = (|| {
        let d2: SifrInt = safe_divide(&SifrInt::from_i64(10), &SifrInt::from_i64(0))?;
        println!("divide(10, 0) = {d2}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("divide(10, 0) error: {}", e.message);
    }
    println!("=== Assert Statement ===");
    println!("all assertions passed");
    println!("=== Explicit Discard ===");
    let _: Result<SifrInt, DivisionError> =
        safe_divide(&SifrInt::from_i64(10), &SifrInt::from_i64(2));
    println!("result discarded safely");
    println!("demo complete!");
}
