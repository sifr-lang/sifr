// src/main.rs
fn parseDigit(ch: &String) -> i64 {
    if (ch).as_str() == "0" {
        return 0_i64;
    }
    if (ch).as_str() == "1" {
        return 1_i64;
    }
    if (ch).as_str() == "2" {
        return 2_i64;
    }
    if (ch).as_str() == "3" {
        return 3_i64;
    }
    if (ch).as_str() == "4" {
        return 4_i64;
    }
    if (ch).as_str() == "5" {
        return 5_i64;
    }
    if (ch).as_str() == "6" {
        return 6_i64;
    }
    if (ch).as_str() == "7" {
        return 7_i64;
    }
    if (ch).as_str() == "8" {
        return 8_i64;
    }
    if (ch).as_str() == "9" {
        return 9_i64;
    }
    -(1_i64)
}

fn parseNumber(s: &String) -> i64 {
    let __sifr_chars_s: Vec<char> = s.chars().collect::<Vec<char>>();
    let mut value: i64 = 0_i64;
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_s.len() as i64)) {
        let ch: String = {
    let Some(__indexed_char) = __sifr_chars_s.get(i as usize).map(|c| c.to_string()) else {
        unreachable!("compiler-verified string index should be in range");
    };
    __indexed_char
};
        let d: i64 = parseDigit(&ch);
        if d < (0_i64) {
            return -(1_i64);
        }
        value = (value * (10_i64)) + d;
        i += 1_i64;
    }
    value
}

fn multiply(num1: &String, num2: &String) -> String {
    let n1: i64 = parseNumber(num1);
    let n2: i64 = parseNumber(num2);
    if (n1 < (0_i64)) || (n2 < (0_i64)) {
        return "0".to_string();
    }
    format!("{}", n1 * n2)
}

fn main() {
    assert!((format!("{}", multiply(&"2".to_string(), &"3".to_string())) == "6"));
    assert!((format!("{}", multiply(&"123".to_string(), &"456".to_string())) == "56088"));
}
