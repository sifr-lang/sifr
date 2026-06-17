use bigdecimal::{BigDecimal, Context, RoundingMode};
use rust_decimal::Decimal;

fn dec(text: &str) -> Decimal {
    Decimal::from_str_exact(text).expect("valid decimal literal")
}

fn big(text: &str) -> BigDecimal {
    text.parse().expect("valid bigdecimal literal")
}

fn round_big(value: BigDecimal) -> BigDecimal {
    Context::default()
        .with_rounding_mode(RoundingMode::HalfEven)
        .with_prec(28)
        .unwrap_or_else(|| Context::default().with_rounding_mode(RoundingMode::HalfEven))
        .round_decimal_ref(&value)
}

fn main() {
    let d = dec("12.50");
    let b = big("3.25");

    let d_plus = d + Decimal::from(2_i64);
    let b_plus = round_big(b + BigDecimal::from(4_i64));

    assert_eq!(d_plus, dec("14.50"));
    assert_eq!(b_plus, big("7.25"));

    println!("decimal_types type-system/parser/HIR integration demo");
    println!("{d_plus}");
    println!("{b_plus}");
}
