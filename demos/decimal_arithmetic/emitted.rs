// src/main.rs
mod sifr_generated_project_nominals {
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct DecimalConversionError {
        pub message: String,
    }
    impl DecimalConversionError {
        #[must_use]
        pub const fn new(message: String) -> Self {
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
pub use sifr_generated_project_nominals::DecimalConversionError;
pub use sifr_generated_project_nominals::DivisionError;
mod sifr_generated_project_unions {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0(
            crate::sifr_generated_project_nominals::DecimalConversionError,
        ),
        SifrGeneratedUnionVariant5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a0(
            crate::sifr_generated_project_nominals::DivisionError,
        ),
    }
    impl From<crate::sifr_generated_project_nominals::DecimalConversionError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0 {
        fn from(
            value: crate::sifr_generated_project_nominals::DecimalConversionError,
        ) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::DivisionError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::DivisionError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::SifrGeneratedUnionVariant5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
}
use ::bigdecimal::BigDecimal;
use ::rust_decimal::Decimal;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0;
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
#[expect(
    clippy::assertions_on_constants,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn main() {
    println!("decimal_arithmetic deterministic arithmetic and context demo");
    let cash: Decimal = Decimal::from_i128_with_scale(1000_i128, 2);
    let fee: Decimal = Decimal::from_i128_with_scale(300_i128, 2);
    let sifr_generated_try_res: Result<
        (),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0,
    > = (|| {
        let cash_floor: Decimal = {
            let sifr_generated_decimal_left_result = Ok(cash);
            let sifr_generated_decimal_right_result = Ok(fee);
            sifr_generated_decimal_left_result
                .and_then(move |sifr_generated_decimal_left| {
                    sifr_generated_decimal_right_result
                        .and_then(move |sifr_generated_decimal_right| {
                            Decimal::checked_div(
                                    sifr_generated_decimal_left,
                                    sifr_generated_decimal_right,
                                )
                                .map(|sifr_generated_decimal_quotient| {
                                    sifr_generated_decimal_quotient.floor()
                                })
                                .map_or_else(
                                    || Err(
                                        if sifr_generated_decimal_right.is_zero() {
                                            SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a0(
                                                DivisionError::new("division by zero".to_string()),
                                            )
                                        } else {
                                            SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0(
                                                DecimalConversionError::new(
                                                    "decimal // operation overflowed its exact representation"
                                                        .to_string(),
                                                ),
                                            )
                                        },
                                    ),
                                    Ok,
                                )
                        })
                })
        }?;
        let cash_remainder: Decimal = {
            let sifr_generated_decimal_left_result = Ok(cash);
            let sifr_generated_decimal_right_result = Ok(fee);
            sifr_generated_decimal_left_result
                .and_then(move |sifr_generated_decimal_left| {
                    sifr_generated_decimal_right_result
                        .and_then(move |sifr_generated_decimal_right| {
                            Decimal::checked_div(
                                    sifr_generated_decimal_left,
                                    sifr_generated_decimal_right,
                                )
                                .and_then(|sifr_generated_decimal_quotient| {
                                    Decimal::checked_mul(
                                            sifr_generated_decimal_quotient.floor(),
                                            sifr_generated_decimal_right,
                                        )
                                        .and_then(|sifr_generated_decimal_product| Decimal::checked_sub(
                                            sifr_generated_decimal_left,
                                            sifr_generated_decimal_product,
                                        ))
                                })
                                .map_or_else(
                                    || Err(
                                        if sifr_generated_decimal_right.is_zero() {
                                            SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a0(
                                                DivisionError::new("division by zero".to_string()),
                                            )
                                        } else {
                                            SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0(
                                                DecimalConversionError::new(
                                                    "decimal % operation overflowed its exact representation"
                                                        .to_string(),
                                                ),
                                            )
                                        },
                                    ),
                                    Ok,
                                )
                        })
                })
        }?;
        let negative_floor: BigDecimal = {
            let sifr_generated_bigdecimal_left = BigDecimal::new(
                    ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[237]),
                    1,
                )
                .clone();
            let sifr_generated_bigdecimal_right = BigDecimal::new(
                    ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[1]),
                    0,
                )
                .clone();
            if ::bigdecimal::Zero::is_zero(&sifr_generated_bigdecimal_right) {
                Err(DivisionError::new("division by zero".to_string()))
            } else {
                Ok(
                    ::bigdecimal::Context::new(
                            ::std::num::NonZeroU64::MIN.saturating_add(27),
                            ::bigdecimal::RoundingMode::HalfEven,
                        )
                        .round_decimal_ref(
                            &(&sifr_generated_bigdecimal_left
                                / &sifr_generated_bigdecimal_right)
                                .with_scale_round(0, ::bigdecimal::RoundingMode::Floor),
                        ),
                )
            }
        }
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a0,
            )?;
        let negative_remainder: BigDecimal = {
            let sifr_generated_bigdecimal_left = BigDecimal::new(
                    ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[237]),
                    1,
                )
                .clone();
            let sifr_generated_bigdecimal_right = BigDecimal::new(
                    ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[1]),
                    0,
                )
                .clone();
            if ::bigdecimal::Zero::is_zero(&sifr_generated_bigdecimal_right) {
                Err(DivisionError::new("division by zero".to_string()))
            } else {
                Ok(
                    ::bigdecimal::Context::new(
                            ::std::num::NonZeroU64::MIN.saturating_add(27),
                            ::bigdecimal::RoundingMode::HalfEven,
                        )
                        .round_decimal_ref(
                            &(&sifr_generated_bigdecimal_left
                                - (&sifr_generated_bigdecimal_left
                                    / &sifr_generated_bigdecimal_right)
                                    .with_scale_round(0, ::bigdecimal::RoundingMode::Floor)
                                    * &sifr_generated_bigdecimal_right),
                        ),
                )
            }
        }
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a0,
            )?;
        println!("{cash_floor}");
        println!("{cash_remainder}");
        println!("{negative_floor}");
        println!("{negative_remainder}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        match sifr_generated_try_err {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let error = sifr_generated_try_variant_error;
                assert!(false, "{error}");
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let error = sifr_generated_try_variant_error;
                assert!(false, "{error}");
            }
        }
    }
    println!(
        "{}",
        Decimal::from_i128_with_scale(25_i128, 1).round_dp_with_strategy(
            {
                let sifr_generated_scale = 0;
                (if sifr_generated_scale < 0 {
                    0
                } else {
                    sifr_generated_scale
                }) as u32
            },
            ::rust_decimal::RoundingStrategy::MidpointNearestEven
        )
    );
    println!(
        "{}",
        Decimal::from_i128_with_scale(25_i128, 1).round_dp_with_strategy(
            {
                let sifr_generated_scale = 0;
                (if sifr_generated_scale < 0 {
                    0
                } else {
                    sifr_generated_scale
                }) as u32
            },
            ::rust_decimal::RoundingStrategy::MidpointNearestEven
        )
    );
    let precise: BigDecimal = BigDecimal::new(
        ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[
            15, 149, 26, 159, 163, 162, 134, 201, 79, 14, 118, 108, 53,
        ]),
        30,
    );
    println!(
        "{}",
        ::bigdecimal::Context::new(
            ::std::num::NonZeroU64::MIN.saturating_add(27),
            ::bigdecimal::RoundingMode::HalfEven
        )
        .round_decimal_ref(
            &(precise
                + BigDecimal::new(
                    ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[0]),
                    0
                )
                .clone())
        )
    );
    println!(
        "{}",
        ::bigdecimal::Context::new(
            ::std::num::NonZeroU64::MIN.saturating_add(27),
            ::bigdecimal::RoundingMode::HalfEven
        )
        .round_decimal_ref(
            &BigDecimal::new(
                ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[25]),
                1
            )
            .with_scale_round(0, ::bigdecimal::RoundingMode::HalfEven)
        )
    );
    println!(
        "{}",
        ::bigdecimal::Context::new(
            ::std::num::NonZeroU64::MIN.saturating_add(27),
            ::bigdecimal::RoundingMode::HalfEven
        )
        .round_decimal_ref(
            &BigDecimal::new(
                ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[25]),
                1
            )
            .with_scale_round(0, ::bigdecimal::RoundingMode::HalfEven)
        )
    );
    println!(
        "{:?}",
        <Decimal as ::rust_decimal::MathematicalOps>::sqrt(&Decimal::from_i128_with_scale(
            -4_i128, 0
        ))
        .map_or_else(
            || Err(DecimalConversionError {
                message: "decimal.sqrt() is undefined for negative values".to_string()
            }),
            Ok
        )
    );
    println!(
        "{:?}",
        BigDecimal::new(
            ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[252]),
            0
        )
        .sqrt_with_context(&::bigdecimal::Context::new(
            ::std::num::NonZeroU64::MIN.saturating_add(27),
            ::bigdecimal::RoundingMode::HalfEven
        ))
        .map_or_else(
            || Err(DecimalConversionError {
                message: "bigdecimal.sqrt() is undefined for negative values".to_string()
            }),
            |sifr_generated_v| Ok(::bigdecimal::Context::new(
                ::std::num::NonZeroU64::MIN.saturating_add(27),
                ::bigdecimal::RoundingMode::HalfEven
            )
            .round_decimal_ref(&sifr_generated_v))
        )
    );
}
