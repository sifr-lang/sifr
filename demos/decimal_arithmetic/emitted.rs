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
    println!("decimal_arithmetic deterministic arithmetic and context demo");
    let cash: Decimal = Decimal::from_str_exact(("10.00".to_string()).as_str())
        .unwrap_or_else(|__e| unreachable!());
    let fee: Decimal = Decimal::from_str_exact(("3.00".to_string()).as_str())
        .unwrap_or_else(|__e| unreachable!());
    println!(
        "{}", { let __l = cash; let __r = fee; Decimal::checked_div(__l, __r)
        .map_or_else(|| {
        eprintln!("runtime error: decimal floor-division failed (division by zero or overflow)");
        ::std::process::exit(1) }, | __q | __q.floor()) }
    );
    println!(
        "{}", { let __l = cash; let __r = fee; Decimal::checked_div(__l, __r)
        .map_or_else(|| {
        eprintln!("runtime error: decimal modulo failed (division by zero or overflow)");
        ::std::process::exit(1) }, | __q | __l - (__q.floor() * __r)) }
    );
    println!(
        "{}", Decimal::from_str_exact(("2.5".to_string()).as_str()).unwrap_or_else(| __e
        | unreachable!()).round_dp_with_strategy({ let __scale = 0_i64; (if __scale < 0 {
        0 } else { __scale }) as u32 },
        ::rust_decimal::RoundingStrategy::MidpointNearestEven)
    );
    println!(
        "{}", Decimal::from_str_exact(("2.5".to_string()).as_str()).unwrap_or_else(| __e
        | unreachable!()).round_dp_with_strategy({ let __scale = 0_i64; (if __scale < 0 {
        0 } else { __scale }) as u32 },
        ::rust_decimal::RoundingStrategy::MidpointNearestEven)
    );
    let precise: BigDecimal = ("1.234567890123456789012345678901".to_string())
        .parse::<BigDecimal>()
        .unwrap_or_else(|__e| unreachable!());
    println!(
        "{}", ::bigdecimal::Context::default()
        .with_rounding_mode(::bigdecimal::RoundingMode::HalfEven).with_prec(28)
        .unwrap_or_else(|| ::bigdecimal::Context::default()
        .with_rounding_mode(::bigdecimal::RoundingMode::HalfEven)).round_decimal_ref(&
        (precise.clone() + ("0".to_string()).parse::< BigDecimal > ().unwrap_or_else(|
        __e | unreachable!()).clone()))
    );
    println!(
        "{}", { let __l = ("-1.9".to_string()).parse::< BigDecimal > ().unwrap_or_else(|
        __e | unreachable!()).clone(); let __r = ("1".to_string()).parse::< BigDecimal >
        ().unwrap_or_else(| __e | unreachable!()).clone(); if __r == BigDecimal::from(0)
        { { eprintln!("runtime error: bigdecimal floor-division by zero");
        ::std::process::exit(1) } } else { ::bigdecimal::Context::default()
        .with_rounding_mode(::bigdecimal::RoundingMode::HalfEven).with_prec(28)
        .unwrap_or_else(|| ::bigdecimal::Context::default()
        .with_rounding_mode(::bigdecimal::RoundingMode::HalfEven)).round_decimal_ref(&
        ((& __l / & __r).with_scale_round(0, ::bigdecimal::RoundingMode::Floor))) } }
    );
    println!(
        "{}", { let __l = ("-1.9".to_string()).parse::< BigDecimal > ().unwrap_or_else(|
        __e | unreachable!()).clone(); let __r = ("1".to_string()).parse::< BigDecimal >
        ().unwrap_or_else(| __e | unreachable!()).clone(); if __r == BigDecimal::from(0)
        { { eprintln!("runtime error: bigdecimal modulo by zero");
        ::std::process::exit(1) } } else { ::bigdecimal::Context::default()
        .with_rounding_mode(::bigdecimal::RoundingMode::HalfEven).with_prec(28)
        .unwrap_or_else(|| ::bigdecimal::Context::default()
        .with_rounding_mode(::bigdecimal::RoundingMode::HalfEven)).round_decimal_ref(& (&
        __l - ((& __l / & __r).with_scale_round(0, ::bigdecimal::RoundingMode::Floor) * &
        __r))) } }
    );
    println!(
        "{}", ::bigdecimal::Context::default()
        .with_rounding_mode(::bigdecimal::RoundingMode::HalfEven).with_prec(28)
        .unwrap_or_else(|| ::bigdecimal::Context::default()
        .with_rounding_mode(::bigdecimal::RoundingMode::HalfEven)).round_decimal_ref(&
        (("2.5".to_string()).parse::< BigDecimal > ().unwrap_or_else(| __e |
        unreachable!()).with_scale_round(0_i64, ::bigdecimal::RoundingMode::HalfEven)))
    );
    println!(
        "{}", ::bigdecimal::Context::default()
        .with_rounding_mode(::bigdecimal::RoundingMode::HalfEven).with_prec(28)
        .unwrap_or_else(|| ::bigdecimal::Context::default()
        .with_rounding_mode(::bigdecimal::RoundingMode::HalfEven)).round_decimal_ref(&
        (("2.5".to_string()).parse::< BigDecimal > ().unwrap_or_else(| __e |
        unreachable!()).with_scale_round(0_i64, ::bigdecimal::RoundingMode::HalfEven)))
    );
    println!(
        "{:?}", < Decimal as ::rust_decimal::MathematicalOps >::sqrt(&
        Decimal::from_str_exact(("-4".to_string()).as_str()).unwrap_or_else(| __e |
        unreachable!())).map_or_else(|| Err(DecimalConversionError { message :
        "decimal.sqrt() is undefined for negative values".to_string().to_string() }), |
        __v | Ok(__v))
    );
    println!(
        "{:?}", ("-4".to_string()).parse::< BigDecimal > ().unwrap_or_else(| __e |
        unreachable!()).sqrt_with_context(&::bigdecimal::Context::default()
        .with_rounding_mode(::bigdecimal::RoundingMode::HalfEven).with_prec(28)
        .unwrap_or_else(|| ::bigdecimal::Context::default()
        .with_rounding_mode(::bigdecimal::RoundingMode::HalfEven))).map_or_else(||
        Err(DecimalConversionError { message :
        "bigdecimal.sqrt() is undefined for negative values".to_string().to_string() }),
        | __v | Ok(::bigdecimal::Context::default()
        .with_rounding_mode(::bigdecimal::RoundingMode::HalfEven).with_prec(28)
        .unwrap_or_else(|| ::bigdecimal::Context::default()
        .with_rounding_mode(::bigdecimal::RoundingMode::HalfEven)).round_decimal_ref(&
        (__v))))
    );
}
