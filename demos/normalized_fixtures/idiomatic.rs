fn parseDigit(ch: &String) -> i64 {
    if ch.clone() == "0".to_string() {
        return 0 as i64;
    }
    if ch.clone() == "1".to_string() {
        return 1 as i64;
    }
    if ch.clone() == "2".to_string() {
        return 2 as i64;
    }
    if ch.clone() == "3".to_string() {
        return 3 as i64;
    }
    if ch.clone() == "4".to_string() {
        return 4 as i64;
    }
    if ch.clone() == "5".to_string() {
        return 5 as i64;
    }
    if ch.clone() == "6".to_string() {
        return 6 as i64;
    }
    if ch.clone() == "7".to_string() {
        return 7 as i64;
    }
    if ch.clone() == "8".to_string() {
        return 8 as i64;
    }
    if ch.clone() == "9".to_string() {
        return 9 as i64;
    }
    return -(1 as i64);
}

fn parseNumber(s: &String) -> i64 {
    let mut value: i64 = 0 as i64;
    let mut i: i64 = 0 as i64;
    while i < (s.chars().count() as i64) {
        let ch: String = {
            let Some(__indexed_char) = s.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        };
        let d: i64 = parseDigit(&ch);
        if d < (0 as i64) {
            return -(1 as i64);
        }
        value = (value * (10 as i64)) + d;
        i += 1 as i64;
    }
    return value;
}

fn multiply(num1: &String, num2: &String) -> String {
    let n1: i64 = parseNumber(num1);
    let n2: i64 = parseNumber(num2);
    if (n1 < (0 as i64)) || (n2 < (0 as i64)) {
        return "0".to_string();
    }
    return format!("{}", n1 * n2);
}

fn main() {
    assert!(format!("{}", multiply(&"2".to_string(), &"3".to_string())) == "6".to_string());
    assert!(format!("{}", multiply(&"123".to_string(), &"456".to_string())) == "56088".to_string());
}
