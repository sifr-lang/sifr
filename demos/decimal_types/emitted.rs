// src/main.rs
use ::rust_decimal::Decimal;

use ::bigdecimal::BigDecimal;

fn main() {
    let d: Decimal = Decimal::from_str_exact(("12.50".to_string()).as_str()).unwrap_or_else(|__e| unreachable!());
    let b: BigDecimal = ("3.25".to_string()).parse::<BigDecimal>().unwrap_or_else(|__e| unreachable!());
    let d_plus: Decimal = d + Decimal::from_str_exact(("2".to_string()).as_str()).unwrap_or_else(|__e| unreachable!());
    let b_plus: BigDecimal = ::bigdecimal::Context::default().with_rounding_mode(::bigdecimal::RoundingMode::HalfEven).with_prec(28).unwrap_or_else(|| ::bigdecimal::Context::default().with_rounding_mode(::bigdecimal::RoundingMode::HalfEven)).round_decimal_ref(&(b.clone() + ("4".to_string()).parse::<BigDecimal>().unwrap_or_else(|__e| unreachable!()).clone()));
    assert!((d_plus == Decimal::from_str_exact(("14.50".to_string()).as_str()).unwrap_or_else(|__e| unreachable!())));
    assert!((b_plus == ("7.25".to_string()).parse::<BigDecimal>().unwrap_or_else(|__e| unreachable!())));
    println!("decimal_types type-system/parser/HIR integration demo");
    println!("{}", format!("{}", d_plus));
    println!("{}", format!("{}", b_plus));
}
