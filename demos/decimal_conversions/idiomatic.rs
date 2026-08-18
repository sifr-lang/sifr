use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Debug)]
struct DecimalConversionError {
    message: String,
}

impl DecimalConversionError {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

fn dec(text: &str) -> Decimal {
    Decimal::from_str_exact(text).expect("valid decimal literal")
}

fn big(text: &str) -> BigDecimal {
    text.parse().expect("valid bigdecimal literal")
}

fn int_from_decimal(value: Decimal) -> Result<i64, DecimalConversionError> {
    truncated_bigint_from_decimal(value)
        .to_i64()
        .ok_or_else(|| DecimalConversionError::new("decimal value out of range for int"))
}

fn int_from_bigdecimal(value: &BigDecimal) -> Result<i64, DecimalConversionError> {
    truncated_bigint_from_bigdecimal(value)
        .to_i64()
        .ok_or_else(|| DecimalConversionError::new("bigdecimal value out of range for int"))
}

fn truncated_bigint_from_decimal(value: Decimal) -> BigInt {
    let scale = value.scale();
    let divisor = BigInt::from(10_u32).pow(scale);
    BigInt::from(value.mantissa()) / divisor
}

fn truncated_bigint_from_bigdecimal(value: &BigDecimal) -> BigInt {
    let (digits, exponent) = value.as_bigint_and_exponent();
    if exponent <= 0 {
        digits * BigInt::from(10_u32).pow((-exponent) as u32)
    } else {
        digits / BigInt::from(10_u32).pow(exponent as u32)
    }
}

fn dumps<T: std::fmt::Display>(value: T) -> String {
    format!("\"{value}\"")
}

fn main() {
    println!("decimal_conversions conversion and boundary rules demo");

    let d = dec("-1.9");
    let bd = big("-1.9");

    match (int_from_decimal(d), int_from_bigdecimal(&bd)) {
        (Ok(i_from_decimal), Ok(i_from_bigdecimal)) => {
            println!("{i_from_decimal}");
            println!("{i_from_bigdecimal}");
        }
        (Err(err), _) | (_, Err(err)) => {
            println!("unexpected conversion failure: {}", err.message);
        }
    }

    let bd_from_decimal = BigDecimal::from_str(&dec("12.3400").to_string())
        .expect("decimal should round-trip into bigdecimal");
    println!("{bd_from_decimal}");

    match Decimal::from_str_exact(&big("7.5000").to_string()) {
        Ok(d_from_bigdecimal) => println!("{d_from_bigdecimal}"),
        Err(_) => println!("unexpected decimal conversion failure: invalid decimal"),
    }

    println!("{}", dumps(dec("1.2300")));
    println!("{}", dumps(big("1.2300")));

    match int_from_bigdecimal(&big("999999999999999999999999999999999999")) {
        Ok(out_of_range) => println!("{out_of_range}"),
        Err(err) => println!("caught conversion error: {}", err.message),
    }
}
