// src/main.rs
mod sifr_generated_project_nominals {
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
use ::bigdecimal::BigDecimal;
use ::rust_decimal::Decimal;
pub use sifr_generated_project_nominals::DecimalConversionError;
#[expect(
    clippy::assertions_on_constants,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
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
    assert_eq!(
        b_plus,
        BigDecimal::new(
            ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&vec![2, 213]),
            2
        )
    );
    let sifr_generated_try_res: Result<(), DecimalConversionError> = (|| {
        let d_plus_value_f82c383d34c2ff04: Decimal = {
            let sifr_generated_decimal_left_result = Ok(d);
            let sifr_generated_decimal_right_result = Ok(Decimal::from_i128_with_scale(2_i128, 0));
            sifr_generated_decimal_left_result.and_then(move |sifr_generated_decimal_left| {
                sifr_generated_decimal_right_result.and_then(move |sifr_generated_decimal_right| {
                    Decimal::checked_add(sifr_generated_decimal_left, sifr_generated_decimal_right)
                        .map_or_else(
                            || {
                                Err(DecimalConversionError::new(
                                    "decimal + operation overflowed its exact representation"
                                        .to_string(),
                                ))
                            },
                            Ok,
                        )
                })
            })
        }?;
        assert_eq!(
            d_plus_value_f82c383d34c2ff04,
            Decimal::from_i128_with_scale(1450_i128, 2)
        );
        println!("decimal_types type-system/parser/HIR integration demo");
        println!("{d_plus_value_f82c383d34c2ff04}");
        println!("{b_plus}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let error = sifr_generated_try_err.clone();
        assert!(false, "{error}");
    }
}
