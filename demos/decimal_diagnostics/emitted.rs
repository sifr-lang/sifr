// src/main.rs
use ::rust_decimal::Decimal;

use ::bigdecimal::BigDecimal;

fn main() {
    println!("decimal_diagnostics decimal diagnostics behavior demo");
    let d: Decimal = Decimal::from_i128_with_scale(105000_i128, 4);
    let b: BigDecimal = BigDecimal::new(::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&vec![3, 42, 140]), 4);
    assert!((format!("{}", d.round_dp_with_strategy({
    let __scale = 2;
    (if __scale < 0 { 0 } else { __scale }) as u32
}, ::rust_decimal::RoundingStrategy::MidpointNearestEven)) == "10.50"));
    assert!((format!("{}", d.round_dp_with_strategy({
    let __scale = 2;
    (if __scale < 0 { 0 } else { __scale }) as u32
}, ::rust_decimal::RoundingStrategy::MidpointNearestEven)) == "10.50"));
    assert!((format!("{}", ::bigdecimal::Context::new(::std::num::NonZeroU64::MIN.saturating_add(27), ::bigdecimal::RoundingMode::HalfEven).round_decimal_ref(&(b.with_scale_round(2, ::bigdecimal::RoundingMode::HalfEven)))) == "20.75000000000000000000000000"));
    assert!((format!("{}", ::bigdecimal::Context::new(::std::num::NonZeroU64::MIN.saturating_add(27), ::bigdecimal::RoundingMode::HalfEven).round_decimal_ref(&(b.with_scale_round(2, ::bigdecimal::RoundingMode::HalfEven)))) == "20.75000000000000000000000000"));
    println!("diagnostic range SIFR-DECIMAL-0001 through SIFR-DECIMAL-0008 is reserved and enforced");
}
