use bigdecimal::{BigDecimal, Context, RoundingMode};
use rust_decimal::{Decimal, MathematicalOps, RoundingStrategy};

#[derive(Debug)]
struct DecimalConversionError {
    #[allow(dead_code)]
    message: String,
}

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

fn floor_div_decimal(lhs: Decimal, rhs: Decimal) -> Decimal {
    (lhs / rhs).floor()
}

fn mod_decimal(lhs: Decimal, rhs: Decimal) -> Decimal {
    lhs - floor_div_decimal(lhs, rhs) * rhs
}

fn floor_div_big(lhs: &BigDecimal, rhs: &BigDecimal) -> BigDecimal {
    round_big((lhs / rhs).with_scale_round(0, RoundingMode::Floor))
}

fn mod_big(lhs: &BigDecimal, rhs: &BigDecimal) -> BigDecimal {
    let quotient = floor_div_big(lhs, rhs);
    round_big(lhs.clone() - quotient * rhs.clone())
}

fn main() {
    println!("decimal_arithmetic deterministic arithmetic and context demo");

    let cash = dec("10.00");
    let fee = dec("3.00");
    println!("{}", floor_div_decimal(cash, fee));
    println!("{}", mod_decimal(cash, fee));
    println!(
        "{}",
        dec("2.5").round_dp_with_strategy(0, RoundingStrategy::MidpointNearestEven)
    );
    println!(
        "{}",
        dec("2.5").round_dp_with_strategy(0, RoundingStrategy::MidpointNearestEven)
    );

    let precise = big("1.234567890123456789012345678901");
    println!("{}", round_big(precise + big("0")));

    let negative = big("-1.9");
    let one = big("1");
    println!("{}", floor_div_big(&negative, &one));
    println!("{}", mod_big(&negative, &one));
    println!(
        "{}",
        round_big(big("2.5").with_scale_round(0, RoundingMode::HalfEven))
    );
    println!(
        "{}",
        round_big(big("2.5").with_scale_round(0, RoundingMode::HalfEven))
    );

    let bad_decimal = dec("-4").sqrt().ok_or_else(|| DecimalConversionError {
        message: "decimal.sqrt() is undefined for negative values".to_string(),
    });
    println!("{bad_decimal:?}");

    let bad_big = big("-4")
        .sqrt_with_context(
            &Context::default()
                .with_rounding_mode(RoundingMode::HalfEven)
                .with_prec(28)
                .unwrap_or_else(|| Context::default().with_rounding_mode(RoundingMode::HalfEven)),
        )
        .map(round_big)
        .ok_or_else(|| DecimalConversionError {
            message: "bigdecimal.sqrt() is undefined for negative values".to_string(),
        });
    println!("{bad_big:?}");
}
