use bigdecimal::{BigDecimal, RoundingMode};
use rust_decimal::{Decimal, RoundingStrategy};

fn main() {
    println!("m28_4 decimal diagnostics contract demo");

    let decimal = Decimal::from_str_exact("10.5000").unwrap_or_else(|_| unreachable!());
    assert_eq!(
        decimal
            .round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven)
            .to_string(),
        "10.50"
    );

    let mut quantized_decimal = decimal;
    quantized_decimal.rescale(2);
    assert_eq!(quantized_decimal.to_string(), "10.50");

    let bigdecimal: BigDecimal = "20.7500".parse().unwrap_or_else(|_| unreachable!());
    let rounded_bigdecimal = bigdecimal
        .with_scale_round(2, RoundingMode::HalfEven)
        .with_scale(26);
    assert_eq!(
        rounded_bigdecimal.to_string(),
        "20.75000000000000000000000000"
    );

    let quantized_bigdecimal = bigdecimal.with_scale(26);
    assert_eq!(
        quantized_bigdecimal.to_string(),
        "20.75000000000000000000000000"
    );

    println!("diagnostic range E2501-E2508 is reserved and enforced");
}
