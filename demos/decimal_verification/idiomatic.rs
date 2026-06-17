use bigdecimal::{BigDecimal, RoundingMode};
use rust_decimal::{Decimal, RoundingStrategy};

fn floor_div_decimal(lhs: Decimal, rhs: Decimal) -> Decimal {
    (lhs / rhs).floor()
}

fn floor_rem_decimal(lhs: Decimal, rhs: Decimal) -> Decimal {
    lhs - floor_div_decimal(lhs, rhs) * rhs
}

fn floor_div_bigdecimal(lhs: &BigDecimal, rhs: &BigDecimal) -> BigDecimal {
    (lhs / rhs).with_scale_round(0, RoundingMode::Floor)
}

fn floor_rem_bigdecimal(lhs: &BigDecimal, rhs: &BigDecimal) -> BigDecimal {
    lhs.clone() - floor_div_bigdecimal(lhs, rhs) * rhs.clone()
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| unreachable!())
}

fn main() {
    println!("decimal_verification verification corpus and determinism gates demo");

    let decimal = Decimal::from_str_exact("-7.5").unwrap_or_else(|_| unreachable!());
    let decimal_two = Decimal::from_str_exact("2").unwrap_or_else(|_| unreachable!());
    println!("{}", floor_div_decimal(decimal, decimal_two));
    println!("{}", floor_rem_decimal(decimal, decimal_two));

    let bigdecimal: BigDecimal = "-7.5".parse().unwrap_or_else(|_| unreachable!());
    let bigdecimal_two: BigDecimal = "2".parse().unwrap_or_else(|_| unreachable!());
    println!(
        "{}",
        floor_div_bigdecimal(&bigdecimal, &bigdecimal_two).with_scale(27)
    );
    println!(
        "{}",
        floor_rem_bigdecimal(&bigdecimal, &bigdecimal_two).with_scale(28)
    );

    println!("{}", json_string("1.2300"));
    println!("{}", json_string("1.2300"));

    let baseline_d = (Decimal::from_str_exact("1.2345").unwrap_or_else(|_| unreachable!())
        * Decimal::from_str_exact("3.0").unwrap_or_else(|_| unreachable!()))
    .round_dp_with_strategy(3, RoundingStrategy::MidpointNearestEven)
    .to_string();
    let baseline_bd = ("1.2345678901234567890123456789"
        .parse::<BigDecimal>()
        .unwrap_or_else(|_| unreachable!())
        + BigDecimal::from(0))
    .with_scale_round(6, RoundingMode::HalfEven)
    .to_string();

    for _ in 0..20 {
        let loop_decimal = (Decimal::from_str_exact("1.2345").unwrap_or_else(|_| unreachable!())
            * Decimal::from_str_exact("3.0").unwrap_or_else(|_| unreachable!()))
        .round_dp_with_strategy(3, RoundingStrategy::MidpointNearestEven)
        .to_string();
        assert_eq!(loop_decimal, baseline_d);

        let loop_bigdecimal = ("1.2345678901234567890123456789"
            .parse::<BigDecimal>()
            .unwrap_or_else(|_| unreachable!())
            + BigDecimal::from(0))
        .with_scale_round(6, RoundingMode::HalfEven)
        .to_string();
        assert_eq!(loop_bigdecimal, baseline_bd);
    }

    println!("deterministic decimal and bigdecimal corpus checks passed");
}
