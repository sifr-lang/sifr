// src/main.rs
use ::sifr_runtime::SifrInt;
#[expect(
    non_snake_case,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn parseDigit(ch: &str) -> SifrInt {
    if ch == "0" {
        return SifrInt::from_i64(0);
    }
    if ch == "1" {
        return SifrInt::from_i64(1);
    }
    if ch == "2" {
        return SifrInt::from_i64(2);
    }
    if ch == "3" {
        return SifrInt::from_i64(3);
    }
    if ch == "4" {
        return SifrInt::from_i64(4);
    }
    if ch == "5" {
        return SifrInt::from_i64(5);
    }
    if ch == "6" {
        return SifrInt::from_i64(6);
    }
    if ch == "7" {
        return SifrInt::from_i64(7);
    }
    if ch == "8" {
        return SifrInt::from_i64(8);
    }
    if ch == "9" {
        return SifrInt::from_i64(9);
    }
    ::std::ops::Neg::neg(&SifrInt::from_i64(1))
}
#[expect(
    non_snake_case,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn parseNumber(s: &str) -> SifrInt {
    let sifr_generated_chars_s: Vec<char> = s.chars().collect::<Vec<char>>();
    let mut value: SifrInt = SifrInt::from_i64(0);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while i < sifr_generated_chars_s.len() {
        let Some(sifr_generated_checked_value_0) = {
            let sifr_generated_string_index = &i;
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_s.len());
            sifr_generated_chars_s
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            break;
        };
        let ch: String = sifr_generated_checked_value_0;
        let d: SifrInt = parseDigit(ch.as_str());
        if d < SifrInt::from_i64(0) {
            return ::std::ops::Neg::neg(&SifrInt::from_i64(1));
        }
        value = ::std::ops::Add::add(&::std::ops::Mul::mul(&value, &SifrInt::from_i64(10)), &d);
        i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
    }
    value
}
fn multiply(num1: &str, num2_argument_5b9f8cba52849293: &str) -> String {
    let n1: SifrInt = parseNumber(num1);
    let n2: SifrInt = parseNumber(num2_argument_5b9f8cba52849293);
    if n1 < SifrInt::from_i64(0) || n2 < SifrInt::from_i64(0) {
        return "0".to_string();
    }
    ::std::ops::Mul::mul(&n1, &n2).to_string()
}
fn main() {
    assert_eq!(multiply("2", "3"), "6");
    assert_eq!(multiply("123", "456"), "56088");
}
