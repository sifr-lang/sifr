// src/main.rs
mod sifr_generated_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Error {
        pub message: String,
    }
    impl Error {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for Error {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for Error {}
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
    impl ValueError {
        #[must_use]
        pub const fn new(message: String) -> Self {
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
    impl From<ParseError> for Error {
        fn from(err: ParseError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<ValueError> for Error {
        fn from(err: ValueError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<DivisionError> for Error {
        fn from(err: DivisionError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<crate::AppError> for Error {
        fn from(err: crate::AppError) -> Self {
            Self::new(err.message)
        }
    }
}
pub use sifr_generated_project_nominals::DivisionError;
pub use sifr_generated_project_nominals::Error;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::ValueError;
mod sifr_generated_project_unions {
    #[derive(Debug, Clone)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a017X3a5X3aclass5X3aError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
            crate::sifr_generated_project_nominals::Error,
        ),
        SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
            crate::sifr_generated_project_nominals::ValueError,
        ),
    }
    impl From<crate::sifr_generated_project_nominals::Error>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::Error) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::ValueError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::ValueError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a017X3a5X3aclass5X3aError1X3a0;
#[derive(Clone, PartialEq, Eq, Hash)]
struct AppError {
    message: String,
}
impl AppError {
    const fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("AppError")
            .field("message", &self.message)
            .finish()
    }
}
impl ::std::fmt::Display for AppError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for AppError {}
fn validate_age(age: SifrInt) -> Result<SifrInt, ValueError> {
    if &age < &SifrInt::from_i64(0) {
        return Err(ValueError::new("age must be positive".to_string()));
    }
    if &age > &SifrInt::from_i64(150) {
        return Err(ValueError::new("too large".to_string()));
    }
    Ok(age.clone())
}
fn safe_divide(a: SifrInt, b: SifrInt) -> Result<SifrInt, DivisionError> {
    if &b == &SifrInt::from_i64(0) {
        return Err(DivisionError::new("division by zero".to_string()));
    }
    Ok(a.floor_div_known_nonzero(&b))
}
fn check_input(x: SifrInt) -> Result<SifrInt, AppError> {
    if &x < &SifrInt::from_i64(0) {
        return Err(AppError::new("invalid input".to_string()));
    }
    Ok(x.clone())
}
fn process_age(age: SifrInt) -> Result<SifrInt, ValueError> {
    if &age < &SifrInt::from_i64(0) {
        return Err(ValueError::new("age must be positive".to_string()));
    }
    if &age > &SifrInt::from_i64(150) {
        return Err(ValueError::new("too large".to_string()));
    }
    Ok(age.clone())
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    println!("=== Built-in Error Classes ===");
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _age: SifrInt = validate_age(-&SifrInt::from_i64(5))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("caught ValueError: {}", e.message.clone());
    }
    let sifr_generated_try_res: Result<(), DivisionError> = (|| {
        let _result: SifrInt = safe_divide(SifrInt::from_i64(10), SifrInt::from_i64(0))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("caught DivisionError: {}", e.message.clone());
    }
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let _n: SifrInt = SifrInt::parse_decimal(
            &"not_a_number".to_string(),
            ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
        )
        .map_err(|e| ParseError {
            message: e.to_string(),
        })?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("caught ParseError: {}", e.message.clone());
    }
    println!("=== Custom Error Classes ===");
    let sifr_generated_try_res: Result<(), AppError> = (|| {
        let _val: SifrInt = check_input(-&SifrInt::from_i64(1))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("caught AppError: {}", e.message.clone());
    }
    println!("=== Exhaustiveness: Specific Except Arms ===");
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _a: SifrInt = validate_age(-&SifrInt::from_i64(10))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("caught ValueError: {}", e.message.clone());
    }
    println!("=== Exhaustiveness: Catch-All ===");
    let sifr_generated_try_res: Result<
        (),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a017X3a5X3aclass5X3aError1X3a0,
    > = (|| {
        let _b: SifrInt = validate_age(SifrInt::from_i64(200))
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0,
            )?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        match sifr_generated_try_err {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = sifr_generated_try_variant_error.clone();
                println!("caught: {}", e.message.clone());
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = Error::new(sifr_generated_try_variant_error.clone().message);
                println!("caught: {}", e.message.clone());
            }
        }
    }
    println!("=== Error Propagation ===");
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _c: SifrInt = process_age(-&SifrInt::from_i64(1))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("pipeline error: {}", e.message.clone());
    }
    println!("=== Multiple Try/Except ===");
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let parsed: SifrInt = SifrInt::parse_decimal(
            &"42".to_string(),
            ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
        )
        .map_err(|e| ParseError {
            message: e.to_string(),
        })?;
        println!("parsed: {parsed}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("parse error: {}", e.message.clone());
    }
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let validated: SifrInt = validate_age(SifrInt::from_i64(42))?;
        println!("validated: {validated}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("validation error: {}", e.message.clone());
    }
    let sifr_generated_try_res: Result<(), DivisionError> = (|| {
        let divided: SifrInt = safe_divide(SifrInt::from_i64(42), SifrInt::from_i64(6))?;
        println!("result: {divided}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("division error: {}", e.message.clone());
    }
    println!("demo complete!");
}
