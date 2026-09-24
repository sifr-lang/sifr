// src/main.rs
use ::bigdecimal::BigDecimal;
use ::rust_decimal::Decimal;
fn main() {
    println!("decimal_diagnostics decimal diagnostics behavior demo");
    let d: Decimal = Decimal::from_i128_with_scale(105_000_i128, 4);
    let b: BigDecimal = BigDecimal::new(
        ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[3, 42, 140]),
        4,
    );
    assert_eq!(
        d.round_dp_with_strategy(
            {
                let sifr_generated_scale = 2_i32;
                if sifr_generated_scale < 0 {
                    0
                } else {
                    sifr_generated_scale
                }
                .cast_unsigned()
            },
            ::rust_decimal::RoundingStrategy::MidpointNearestEven
        )
        .to_string(),
        "10.50"
    );
    assert_eq!(
        d.round_dp_with_strategy(
            {
                let sifr_generated_scale = 2_i32;
                if sifr_generated_scale < 0 {
                    0
                } else {
                    sifr_generated_scale
                }
                .cast_unsigned()
            },
            ::rust_decimal::RoundingStrategy::MidpointNearestEven
        )
        .to_string(),
        "10.50"
    );
    assert_eq!(
        ::bigdecimal::Context::new(
            ::std::num::NonZeroU64::MIN.saturating_add(27),
            ::bigdecimal::RoundingMode::HalfEven
        )
        .round_decimal_ref(&b.with_scale_round(2, ::bigdecimal::RoundingMode::HalfEven))
        .to_string(),
        "20.75000000000000000000000000"
    );
    assert_eq!(
        ::bigdecimal::Context::new(
            ::std::num::NonZeroU64::MIN.saturating_add(27),
            ::bigdecimal::RoundingMode::HalfEven
        )
        .round_decimal_ref(&b.with_scale_round(2, ::bigdecimal::RoundingMode::HalfEven))
        .to_string(),
        "20.75000000000000000000000000"
    );
    println!(
        "diagnostic range SIFR-DECIMAL-0001 through SIFR-DECIMAL-0008 is reserved and enforced"
    );
}
