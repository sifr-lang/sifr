// src/main.rs
mod __sifr_project_nominals {
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
use ::rust_decimal::Decimal;
use ::bigdecimal::BigDecimal;
fn main() {
    let d: Decimal = Decimal::from_i128_with_scale(1250_i128, 2);
    let b: BigDecimal = BigDecimal::new(
        ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&vec![1, 69]),
        2,
    );
    let b_plus: BigDecimal = ::bigdecimal::Context::new(
            ::std::num::NonZeroU64::MIN.saturating_add(27),
            ::bigdecimal::RoundingMode::HalfEven,
        )
        .round_decimal_ref(
            &(b.clone()
                + BigDecimal::new(
                        ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&vec![4]),
                        0,
                    )
                    .clone()),
        );
    assert!(
        (b_plus ==
        BigDecimal::new(::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(& vec![2,
        213]), 2))
    );
    let __sifr_try_res: Result<(), DecimalConversionError> = (|| {
        let d_plus: Decimal = ({
            let __sifr_decimal_left_result = Ok(d);
            let __sifr_decimal_right_result = Ok(
                Decimal::from_i128_with_scale(2_i128, 0),
            );
            __sifr_decimal_left_result
                .and_then(move |__sifr_decimal_left| {
                    __sifr_decimal_right_result
                        .and_then(move |__sifr_decimal_right| {
                            Decimal::checked_add(
                                    __sifr_decimal_left,
                                    __sifr_decimal_right,
                                )
                                .map_or_else(
                                    || Err(
                                        DecimalConversionError::new(
                                            "decimal + operation overflowed its exact representation"
                                                .to_string(),
                                        ),
                                    ),
                                    |__sifr_decimal_value| Ok(__sifr_decimal_value),
                                )
                        })
                })
        })?;
        assert!((d_plus == Decimal::from_i128_with_scale(1450_i128, 2)));
        println!("decimal_types type-system/parser/HIR integration demo");
        println!("{}", format!("{}", d_plus));
        println!("{}", format!("{}", b_plus));
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let error = __sifr_try_err.clone();
        assert!(false, "{}", format!("{}", error));
    }
}
