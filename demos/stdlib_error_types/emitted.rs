// src/main.rs
mod sifr_generated_project_nominals {
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2egraphlibX2eCycleError {
        pub message: String,
    }
    impl SifrGeneratedStdlibSifrX2egraphlibX2eCycleError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Debug for SifrGeneratedStdlibSifrX2egraphlibX2eCycleError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("CycleError")
                .field("message", &self.message)
                .finish()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2egraphlibX2eCycleError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    impl ::std::error::Error for SifrGeneratedStdlibSifrX2egraphlibX2eCycleError {}
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
        pub message: String,
    }
    impl SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Debug for SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("StatisticsError")
                .field("message", &self.message)
                .finish()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    impl ::std::error::Error for SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {}
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
    pub struct FloatOverflowError {
        pub message: String,
    }
    impl FloatOverflowError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for FloatOverflowError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for FloatOverflowError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct FloatPrecisionLossError {
        pub message: String,
    }
    impl FloatPrecisionLossError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for FloatPrecisionLossError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for FloatPrecisionLossError {}
}
pub use sifr_generated_project_nominals::FloatOverflowError;
pub use sifr_generated_project_nominals::FloatPrecisionLossError;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2egraphlibX2eCycleError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError;
mod sifr_generated_project_unions {
    #[derive(Debug, Clone)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a236X3a5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a044X3a5X3aclass31X3asifrX2estatisticsX2eStatisticsError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0(
            crate::sifr_generated_project_nominals::ParseError,
        ),
        SifrGeneratedUnionVariant5X3aclass31X3asifrX2estatisticsX2eStatisticsError1X3a0(
            crate::sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        ),
    }
    impl From<crate::sifr_generated_project_nominals::ParseError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a236X3a5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a044X3a5X3aclass31X3asifrX2estatisticsX2eStatisticsError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::ParseError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0(
                value,
            )
        }
    }
    impl From<
        crate::sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
    >
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a236X3a5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a044X3a5X3aclass31X3asifrX2estatisticsX2eStatisticsError1X3a0 {
        fn from(
            value: crate::sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        ) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass31X3asifrX2estatisticsX2eStatisticsError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a236X3a5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a044X3a5X3aclass31X3asifrX2estatisticsX2eStatisticsError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant5X3aclass31X3asifrX2estatisticsX2eStatisticsError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a244X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
            crate::sifr_generated_project_nominals::FloatOverflowError,
        ),
        SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
            crate::sifr_generated_project_nominals::FloatPrecisionLossError,
        ),
    }
    impl From<crate::sifr_generated_project_nominals::FloatOverflowError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a244X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0 {
        fn from(
            value: crate::sifr_generated_project_nominals::FloatOverflowError,
        ) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::FloatPrecisionLossError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a244X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0 {
        fn from(
            value: crate::sifr_generated_project_nominals::FloatPrecisionLossError,
        ) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a244X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a236X3a5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a044X3a5X3aclass31X3asifrX2estatisticsX2eStatisticsError1X3a0;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a244X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0;
fn compute_mean(
    data: &[f64],
) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
    if data.len() == SifrInt::from_i64(0) {
        return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
            "cannot compute mean of empty dataset".to_string(),
        ));
    }
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        total += val;
    }
    let sifr_generated_try_res: Result<
        Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError>,
        SifrGeneratedUnion8X3asequence5X3aunion1X3a244X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0,
    > = (|| {
        let count: f64 = SifrInt::from(data.len())
            .checked_to_f64()
            .map_err(|sifr_generated_float_error| match sifr_generated_float_error {
                ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a244X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                        FloatOverflowError::new(
                            "exact integer is outside the finite float range".to_string(),
                        ),
                    )
                }
                ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a244X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                        FloatPrecisionLossError::new(
                            "exact integer cannot be represented without float precision loss"
                                .to_string(),
                        ),
                    )
                }
            })?;
        Ok(Ok(total / count))
    })();
    sifr_generated_try_res
        .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a244X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let error = sifr_generated_try_variant_error;
                Err(
                    SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                        error.message,
                    ),
                )
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a244X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let error = sifr_generated_try_variant_error;
                Err(
                    SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                        error.message,
                    ),
                )
            }
        })
}
fn topo_sort(has_cycle: bool) -> Result<SifrInt, SifrGeneratedStdlibSifrX2egraphlibX2eCycleError> {
    if has_cycle {
        return Err(SifrGeneratedStdlibSifrX2egraphlibX2eCycleError::new(
            "graph contains a cycle".to_string(),
        ));
    }
    Ok(SifrInt::from_i64(42))
}
fn main() {
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let mean: f64 = compute_mean(&[1.0_f64, 2.0_f64, 3.0_f64])?;
            println!("mean = {mean}");
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("stats error: {}", e.message);
    }
    let empty: Vec<f64> = Vec::new();
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let _mean2: f64 = compute_mean(&empty)?;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught StatisticsError: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2egraphlibX2eCycleError> =
        (|| {
            let order: SifrInt = topo_sort(false)?;
            println!("topo sort result = {order}");
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("cycle error: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2egraphlibX2eCycleError> =
        (|| {
            let _order2: SifrInt = topo_sort(true)?;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught CycleError: {}", e.message);
    }
    let sifr_generated_try_res: Result<
        (),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a236X3a5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a044X3a5X3aclass31X3asifrX2estatisticsX2eStatisticsError1X3a0,
    > = (|| {
        let _val: SifrInt = SifrInt::parse_decimal(
                "not_a_number",
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a236X3a5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a044X3a5X3aclass31X3asifrX2estatisticsX2eStatisticsError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0,
            )?;
        let _mean3_value_1dfeeebaa86aa660: f64 = compute_mean(&empty)
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a236X3a5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a044X3a5X3aclass31X3asifrX2estatisticsX2eStatisticsError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2estatisticsX2eStatisticsError1X3a0,
            )?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        match sifr_generated_try_err {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a236X3a5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a044X3a5X3aclass31X3asifrX2estatisticsX2eStatisticsError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = sifr_generated_try_variant_error;
                println!("caught ParseError: {}", e.message);
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a236X3a5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a044X3a5X3aclass31X3asifrX2estatisticsX2eStatisticsError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2estatisticsX2eStatisticsError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = sifr_generated_try_variant_error;
                println!("caught StatisticsError: {}", e.message);
            }
        }
    }
    println!("all module-specific error types work correctly");
}
