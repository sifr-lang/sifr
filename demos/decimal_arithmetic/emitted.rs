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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct DecimalConversionError {
        pub message: String,
    }
    impl DecimalConversionError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for DecimalConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for DecimalConversionError {}
}
pub use __sifr_project_nominals::DecimalConversionError;
pub use __sifr_project_nominals::DivisionError;

mod __sifr_project_unions {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0 {
        __SifrUnionVariant_5_x3aclass22_x3aDecimalConversionError1_x3a0(
            crate::__sifr_project_nominals::DecimalConversionError,
        ),
        __SifrUnionVariant_5_x3aclass13_x3aDivisionError1_x3a0(
            crate::__sifr_project_nominals::DivisionError,
        ),
    }
    impl From<crate::__sifr_project_nominals::DecimalConversionError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0 {
        fn from(value: crate::__sifr_project_nominals::DecimalConversionError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass22_x3aDecimalConversionError1_x3a0(
                value,
            )
        }
    }
    impl From<crate::__sifr_project_nominals::DivisionError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0 {
        fn from(value: crate::__sifr_project_nominals::DivisionError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass13_x3aDivisionError1_x3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass22_x3aDecimalConversionError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass13_x3aDivisionError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
}
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0;
use ::rust_decimal::Decimal;
use ::bigdecimal::BigDecimal;
fn main() {
    println!("decimal_arithmetic deterministic arithmetic and context demo");
    let cash: Decimal = Decimal::from_i128_with_scale(1000_i128, 2);
    let fee: Decimal = Decimal::from_i128_with_scale(300_i128, 2);
    let __sifr_try_res: Result<
        (),
        __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0,
    > = (|| {
        let cash_floor: Decimal = ({
            let __sifr_decimal_left_result = Ok(cash);
            let __sifr_decimal_right_result = Ok(fee);
            __sifr_decimal_left_result
                .and_then(move |__sifr_decimal_left| {
                    __sifr_decimal_right_result
                        .and_then(move |__sifr_decimal_right| {
                            Decimal::checked_div(
                                    __sifr_decimal_left,
                                    __sifr_decimal_right,
                                )
                                .map(|__sifr_decimal_quotient| {
                                    __sifr_decimal_quotient.floor()
                                })
                                .map_or_else(
                                    || Err(
                                        if __sifr_decimal_right.is_zero() {
                                            __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass13_x3aDivisionError1_x3a0(
                                                DivisionError::new("division by zero".to_string()),
                                            )
                                        } else {
                                            __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass22_x3aDecimalConversionError1_x3a0(
                                                DecimalConversionError::new(
                                                    "decimal // operation overflowed its exact representation"
                                                        .to_string(),
                                                ),
                                            )
                                        },
                                    ),
                                    |__sifr_decimal_value| Ok(__sifr_decimal_value),
                                )
                        })
                })
        })?;
        let cash_remainder: Decimal = ({
            let __sifr_decimal_left_result = Ok(cash);
            let __sifr_decimal_right_result = Ok(fee);
            __sifr_decimal_left_result
                .and_then(move |__sifr_decimal_left| {
                    __sifr_decimal_right_result
                        .and_then(move |__sifr_decimal_right| {
                            Decimal::checked_div(
                                    __sifr_decimal_left,
                                    __sifr_decimal_right,
                                )
                                .and_then(|__sifr_decimal_quotient| {
                                    Decimal::checked_mul(
                                            __sifr_decimal_quotient.floor(),
                                            __sifr_decimal_right,
                                        )
                                        .and_then(|__sifr_decimal_product| Decimal::checked_sub(
                                            __sifr_decimal_left,
                                            __sifr_decimal_product,
                                        ))
                                })
                                .map_or_else(
                                    || Err(
                                        if __sifr_decimal_right.is_zero() {
                                            __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass13_x3aDivisionError1_x3a0(
                                                DivisionError::new("division by zero".to_string()),
                                            )
                                        } else {
                                            __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass22_x3aDecimalConversionError1_x3a0(
                                                DecimalConversionError::new(
                                                    "decimal % operation overflowed its exact representation"
                                                        .to_string(),
                                                ),
                                            )
                                        },
                                    ),
                                    |__sifr_decimal_value| Ok(__sifr_decimal_value),
                                )
                        })
                })
        })?;
        let negative_floor: BigDecimal = ({
            let __sifr_bigdecimal_left = BigDecimal::new(
                    ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&vec![237]),
                    1,
                )
                .clone();
            let __sifr_bigdecimal_right = BigDecimal::new(
                    ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&vec![1]),
                    0,
                )
                .clone();
            if ::bigdecimal::Zero::is_zero(&__sifr_bigdecimal_right) {
                Err(DivisionError::new("division by zero".to_string()))
            } else {
                Ok(
                    ::bigdecimal::Context::new(
                            ::std::num::NonZeroU64::MIN.saturating_add(27),
                            ::bigdecimal::RoundingMode::HalfEven,
                        )
                        .round_decimal_ref(
                            &((&__sifr_bigdecimal_left / &__sifr_bigdecimal_right)
                                .with_scale_round(0, ::bigdecimal::RoundingMode::Floor)),
                        ),
                )
            }
        })
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass13_x3aDivisionError1_x3a0(
                __e,
            ))?;
        let negative_remainder: BigDecimal = ({
            let __sifr_bigdecimal_left = BigDecimal::new(
                    ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&vec![237]),
                    1,
                )
                .clone();
            let __sifr_bigdecimal_right = BigDecimal::new(
                    ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&vec![1]),
                    0,
                )
                .clone();
            if ::bigdecimal::Zero::is_zero(&__sifr_bigdecimal_right) {
                Err(DivisionError::new("division by zero".to_string()))
            } else {
                Ok(
                    ::bigdecimal::Context::new(
                            ::std::num::NonZeroU64::MIN.saturating_add(27),
                            ::bigdecimal::RoundingMode::HalfEven,
                        )
                        .round_decimal_ref(
                            &(&__sifr_bigdecimal_left
                                - ((&__sifr_bigdecimal_left / &__sifr_bigdecimal_right)
                                    .with_scale_round(0, ::bigdecimal::RoundingMode::Floor)
                                    * &__sifr_bigdecimal_right)),
                        ),
                )
            }
        })
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass13_x3aDivisionError1_x3a0(
                __e,
            ))?;
        println!("{}", cash_floor);
        println!("{}", cash_remainder);
        println!("{}", negative_floor);
        println!("{}", negative_remainder);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        match __sifr_try_err {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass22_x3aDecimalConversionError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let error = __sifr_try_variant_error.clone();
                assert!(false, "{}", format!("{}", error));
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass13_x3aDivisionError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let error = __sifr_try_variant_error.clone();
                assert!(false, "{}", format!("{}", error));
            }
        }
    }
    println!(
        "{}", Decimal::from_i128_with_scale(25_i128, 1).round_dp_with_strategy({ let
        __scale = 0; (if __scale < 0 { 0 } else { __scale }) as u32 },
        ::rust_decimal::RoundingStrategy::MidpointNearestEven)
    );
    println!(
        "{}", Decimal::from_i128_with_scale(25_i128, 1).round_dp_with_strategy({ let
        __scale = 0; (if __scale < 0 { 0 } else { __scale }) as u32 },
        ::rust_decimal::RoundingStrategy::MidpointNearestEven)
    );
    let precise: BigDecimal = BigDecimal::new(
        ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(
            &vec![15, 149, 26, 159, 163, 162, 134, 201, 79, 14, 118, 108, 53],
        ),
        30,
    );
    println!(
        "{}", ::bigdecimal::Context::new(::std::num::NonZeroU64::MIN.saturating_add(27),
        ::bigdecimal::RoundingMode::HalfEven).round_decimal_ref(& (precise.clone() +
        BigDecimal::new(::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&
        vec![0]), 0).clone()))
    );
    println!(
        "{}", ::bigdecimal::Context::new(::std::num::NonZeroU64::MIN.saturating_add(27),
        ::bigdecimal::RoundingMode::HalfEven).round_decimal_ref(&
        (BigDecimal::new(::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&
        vec![25]), 1).with_scale_round(0, ::bigdecimal::RoundingMode::HalfEven)))
    );
    println!(
        "{}", ::bigdecimal::Context::new(::std::num::NonZeroU64::MIN.saturating_add(27),
        ::bigdecimal::RoundingMode::HalfEven).round_decimal_ref(&
        (BigDecimal::new(::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&
        vec![25]), 1).with_scale_round(0, ::bigdecimal::RoundingMode::HalfEven)))
    );
    println!(
        "{:?}", < Decimal as ::rust_decimal::MathematicalOps >::sqrt(&
        Decimal::from_i128_with_scale(- 4_i128, 0)).map_or_else(||
        Err(DecimalConversionError { message :
        "decimal.sqrt() is undefined for negative values".to_string().to_string() }), |
        __v | Ok(__v))
    );
    println!(
        "{:?}", BigDecimal::new(::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&
        vec![252]), 0)
        .sqrt_with_context(&::bigdecimal::Context::new(::std::num::NonZeroU64::MIN
        .saturating_add(27), ::bigdecimal::RoundingMode::HalfEven)).map_or_else(||
        Err(DecimalConversionError { message :
        "bigdecimal.sqrt() is undefined for negative values".to_string().to_string() }),
        | __v | Ok(::bigdecimal::Context::new(::std::num::NonZeroU64::MIN
        .saturating_add(27), ::bigdecimal::RoundingMode::HalfEven).round_decimal_ref(&
        (__v))))
    );
}
