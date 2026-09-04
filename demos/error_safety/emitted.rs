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
}
pub use sifr_generated_project_nominals::DivisionError;
pub use sifr_generated_project_nominals::Error;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::ValueError;
mod sifr_generated_project_unions {
    #[derive(Debug, Clone)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass18X3asifrX2ebuiltinX2eError1X3a0(
            crate::sifr_generated_project_nominals::Error,
        ),
        SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
            crate::sifr_generated_project_nominals::ValueError,
        ),
    }
    impl From<crate::sifr_generated_project_nominals::Error>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::Error) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass18X3asifrX2ebuiltinX2eError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::ValueError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::ValueError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::SifrGeneratedUnionVariant5X3aclass18X3asifrX2ebuiltinX2eError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0;
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
    if age < SifrInt::from_i64(0) {
        return Err(ValueError::new("age must be positive".to_string()));
    }
    if age > SifrInt::from_i64(150) {
        return Err(ValueError::new("too large".to_string()));
    }
    Ok(age)
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn safe_divide(a: SifrInt, b: SifrInt) -> Result<SifrInt, DivisionError> {
    if b == SifrInt::from_i64(0) {
        return Err(DivisionError::new("division by zero".to_string()));
    }
    Ok(a.floor_div_known_nonzero(&b))
}
fn check_input(x: SifrInt) -> Result<SifrInt, AppError> {
    if x < SifrInt::from_i64(0) {
        return Err(AppError::new("invalid input".to_string()));
    }
    Ok(x)
}
fn process_age(age: SifrInt) -> Result<SifrInt, ValueError> {
    if age < SifrInt::from_i64(0) {
        return Err(ValueError::new("age must be positive".to_string()));
    }
    if age > SifrInt::from_i64(150) {
        return Err(ValueError::new("too large".to_string()));
    }
    Ok(age)
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    println!("=== Built-in Error Classes ===");
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _ = validate_age(::std::ops::Neg::neg(&SifrInt::from_i64(5)))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught ValueError: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), DivisionError> = (|| {
        let _ = safe_divide(SifrInt::from_i64(10), SifrInt::from_i64(0))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught DivisionError: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let _ = SifrInt::parse_decimal(
            &"not_a_number".to_string(),
            ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
        )
        .map_err(|e| ParseError {
            message: e.to_string(),
        })?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught ParseError: {}", e.message);
    }
    println!("=== Custom Error Classes ===");
    let sifr_generated_try_res: Result<(), AppError> = (|| {
        let _ = check_input(::std::ops::Neg::neg(&SifrInt::from_i64(1)))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught AppError: {}", e.message);
    }
    println!("=== Exhaustiveness: Specific Except Arms ===");
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _ = validate_age(::std::ops::Neg::neg(&SifrInt::from_i64(10)))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught ValueError: {}", e.message);
    }
    println!("=== Exhaustiveness: Catch-All ===");
    let sifr_generated_try_res: Result<
        (),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0,
    > = (|| {
        let _ = validate_age(SifrInt::from_i64(200))
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0,
            )?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        match sifr_generated_try_err {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3asifrX2ebuiltinX2eError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = sifr_generated_try_variant_error;
                println!("caught: {}", e.message);
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = Error::new(sifr_generated_try_variant_error.message);
                println!("caught: {}", e.message);
            }
        }
    }
    println!("=== Error Propagation ===");
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _ = process_age(::std::ops::Neg::neg(&SifrInt::from_i64(1)))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("pipeline error: {}", e.message);
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
        let e = sifr_generated_try_err;
        println!("parse error: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let validated: SifrInt = validate_age(SifrInt::from_i64(42))?;
        println!("validated: {validated}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("validation error: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), DivisionError> = (|| {
        let divided: SifrInt = safe_divide(SifrInt::from_i64(42), SifrInt::from_i64(6))?;
        println!("result: {divided}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("division error: {}", e.message);
    }
    println!("demo complete!");
}
