// src/main.rs
mod __sifr_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Error {
        pub message: String,
    }
    impl Error {
        pub fn new(message: String) -> Self {
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
pub use __sifr_project_nominals::DivisionError;
pub use __sifr_project_nominals::Error;
pub use __sifr_project_nominals::ParseError;
pub use __sifr_project_nominals::ValueError;

mod __sifr_project_unions {
    #[derive(Debug, Clone)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aValueError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        __SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(crate::__sifr_project_nominals::Error),
        __SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
            crate::__sifr_project_nominals::ValueError,
        ),
    }
    impl From<crate::__sifr_project_nominals::Error>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aValueError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        fn from(value: crate::__sifr_project_nominals::Error) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aValueError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(
                value,
            )
        }
    }
    impl From<crate::__sifr_project_nominals::ValueError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aValueError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        fn from(value: crate::__sifr_project_nominals::ValueError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aValueError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aValueError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aValueError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aValueError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
}
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aValueError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0;
#[derive(Clone, PartialEq, Eq, Hash)]
struct AppError {
    message: String,
}
impl AppError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl AppError {}
impl ::std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("AppError").field("message", &self.message).finish()
    }
}
impl ::std::fmt::Display for AppError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for AppError {}
fn validate_age(age: i64) -> Result<i64, ValueError> {
    if age < (0_i64) {
        return Err(ValueError::new("age must be positive".to_string()));
    }
    if age > (150_i64) {
        return Err(ValueError::new("too large".to_string()));
    }
    Ok(age)
}
fn safe_divide(a: i64, b: i64) -> Result<i64, DivisionError> {
    if b == (0_i64) {
        return Err(DivisionError::new("division by zero".to_string()));
    }
    Ok(a / b)
}
fn check_input(x: i64) -> Result<i64, AppError> {
    if x < (0_i64) {
        return Err(AppError::new("invalid input".to_string()));
    }
    Ok(x)
}
fn process_age(age: i64) -> Result<i64, ValueError> {
    if age < (0_i64) {
        return Err(ValueError::new("age must be positive".to_string()));
    }
    if age > (150_i64) {
        return Err(ValueError::new("too large".to_string()));
    }
    Ok(age)
}
fn main() {
    println!("=== Built-in Error Classes ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let age: i64 = validate_age(-(5_i64))?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught ValueError: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), DivisionError> = (|| {
        let result: i64 = safe_divide(10_i64, 0_i64)?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught DivisionError: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), ParseError> = (|| {
        let n: i64 = ("not_a_number".to_string())
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught ParseError: {}", e.message.clone());
    }
    println!("=== Custom Error Classes ===");
    let __sifr_try_res: Result<(), AppError> = (|| {
        let val: i64 = check_input(-(1_i64))?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught AppError: {}", e.message.clone());
    }
    println!("=== Exhaustiveness: Specific Except Arms ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let a: i64 = validate_age(-(10_i64))?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught ValueError: {}", e.message.clone());
    }
    println!("=== Exhaustiveness: Catch-All ===");
    let __sifr_try_res: Result<
        (),
        __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aValueError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0,
    > = (|| {
        let b: i64 = (validate_age(200_i64))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aValueError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                __e,
            ))?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        match __sifr_try_err {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aValueError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let e = __sifr_try_variant_error.clone();
                println!("caught: {}", e.message.clone());
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aValueError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let e = Error::new(__sifr_try_variant_error.clone().message);
                println!("caught: {}", e.message.clone());
            }
        }
    }
    println!("=== Error Propagation ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let c: i64 = process_age(-(1_i64))?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("pipeline error: {}", e.message.clone());
    }
    println!("=== Multiple Try/Except ===");
    let __sifr_try_res: Result<(), ParseError> = (|| {
        let parsed: i64 = ("42".to_string())
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        println!("parsed: {}", parsed);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("parse error: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let validated: i64 = validate_age(42_i64)?;
        println!("validated: {}", validated);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("validation error: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), DivisionError> = (|| {
        let divided: i64 = safe_divide(42_i64, 6_i64)?;
        println!("result: {}", divided);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("division error: {}", e.message.clone());
    }
    println!("demo complete!");
}
