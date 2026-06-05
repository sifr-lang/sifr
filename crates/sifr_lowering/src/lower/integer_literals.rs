use num_bigint::BigUint;
use sifr_python_ast::Int;

pub(in crate::lower) fn canonical_large_int_literal_text(value: &Int) -> String {
    let text = value.to_string();
    parse_unsigned_integer_literal_text(&text).map_or(text, |integer| integer.to_str_radix(10))
}

fn parse_unsigned_integer_literal_text(text: &str) -> Option<BigUint> {
    let (radix, digits) =
        if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
            (16, hex)
        } else if let Some(octal) = text.strip_prefix("0o").or_else(|| text.strip_prefix("0O")) {
            (8, octal)
        } else if let Some(binary) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
            (2, binary)
        } else {
            (10, text)
        };
    let compact_digits: String = digits.chars().filter(|&ch| ch != '_').collect();
    BigUint::parse_bytes(compact_digits.as_bytes(), radix)
}
