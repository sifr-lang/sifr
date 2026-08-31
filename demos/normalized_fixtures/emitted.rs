// src/main.rs
use ::sifr_runtime::SifrInt;

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
    -&SifrInt::from_i64(1)
}

fn parseNumber(s: &str) -> SifrInt {
    let __sifr_chars_s: Vec<char> = s.chars().collect::<Vec<char>>();
    let mut value: SifrInt = SifrInt::from_i64(0);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_s.len())) {
        let Some(__sifr_checked_value_0) = ({
    let __sifr_string_index = i.clone();
    let __sifr_string_index_normalized = __sifr_string_index.normalize_index_or_len(__sifr_chars_s.len());
    __sifr_chars_s.get(__sifr_string_index_normalized)
}).map(|c| c.to_string()) else {
            break;
        };
        let ch: String = __sifr_checked_value_0.clone();
        let d: SifrInt = parseDigit(&ch);
        if (&d < &SifrInt::from_i64(0)) {
            return -&SifrInt::from_i64(1);
        }
        value = &(&value * &SifrInt::from_i64(10)) + &d;
        i = &i + &SifrInt::from_i64(1);
    }
    value.clone()
}

fn multiply(num1: &str, num2: &str) -> String {
    let n1: SifrInt = parseNumber(num1);
    let n2: SifrInt = parseNumber(num2);
    if (&n1 < &SifrInt::from_i64(0)) || (&n2 < &SifrInt::from_i64(0)) {
        return "0".to_string();
    }
    format!("{}", &n1 * &n2)
}

fn main() {
    assert!((format!("{}", multiply(&"2".to_string(), &"3".to_string())) == "6"));
    assert!((format!("{}", multiply(&"123".to_string(), &"456".to_string())) == "56088"));
}
