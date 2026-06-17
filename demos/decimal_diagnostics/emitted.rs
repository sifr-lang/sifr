use rust_decimal::Decimal;

use bigdecimal::BigDecimal;

fn main() {
    println!("decimal_diagnostics decimal diagnostics behavior demo");
    let d: Decimal = Decimal::from_str_exact(("10.5000".to_string()).as_str()).unwrap_or_else(|__e| unreachable!());
    let b: BigDecimal = ("20.7500".to_string()).parse::<BigDecimal>().unwrap_or_else(|__e| unreachable!());
    assert!(format!("{}", d.round_dp_with_strategy({
    let __scale = 2 as i64;
    (if __scale < 0 { 0 } else { __scale }) as u32
}, rust_decimal::RoundingStrategy::MidpointNearestEven)) == "10.50".to_string());
    assert!(format!("{}", d.round_dp_with_strategy({
    let __scale = 2 as i64;
    (if __scale < 0 { 0 } else { __scale }) as u32
}, rust_decimal::RoundingStrategy::MidpointNearestEven)) == "10.50".to_string());
    assert!(format!("{}", bigdecimal::Context::default().with_rounding_mode(bigdecimal::RoundingMode::HalfEven).with_prec(28).unwrap_or_else(|| bigdecimal::Context::default().with_rounding_mode(bigdecimal::RoundingMode::HalfEven)).round_decimal_ref(&(b.with_scale_round(2 as i64, bigdecimal::RoundingMode::HalfEven)))) == "20.75000000000000000000000000".to_string());
    assert!(format!("{}", bigdecimal::Context::default().with_rounding_mode(bigdecimal::RoundingMode::HalfEven).with_prec(28).unwrap_or_else(|| bigdecimal::Context::default().with_rounding_mode(bigdecimal::RoundingMode::HalfEven)).round_decimal_ref(&(b.with_scale_round(2 as i64, bigdecimal::RoundingMode::HalfEven)))) == "20.75000000000000000000000000".to_string());
    println!(
        "diagnostic range SIFR-DECIMAL-0001 through SIFR-DECIMAL-0008 is reserved and enforced"
    );
}
