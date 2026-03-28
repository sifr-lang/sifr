fn collect_vowels(text: &String) -> String {
    let mut result: String = "".to_string();
    let mut i: i64 = 0 as i64;
    while i < (text.chars().count() as i64) {
        let ch: String = {
    let Some(__indexed_char) = text.chars().nth(i as usize) else {
        unreachable!("compiler-verified string index should be in range");
    };
    __indexed_char.to_string()
};
        if "aeiou".to_string().contains(&ch) {
            result = format!("{}{}", result, ch);
        }
        i = i + (1 as i64);
    }
    return result;
}

fn sum_all(values: &Vec<i64>) -> i64 {
    let mut total: i64 = 0 as i64;
    for i in 0 as i64..values.len() as i64 {
        total = total + values[i as usize];
    }
    return total;
}

fn head_or_zero(values: &Vec<i64>) -> i64 {
    if (values.len() as i64) == (0 as i64) {
        return 0 as i64;
    }
    let first: i64 = values[(0 as i64) as usize];
    return first;
}

fn main() {
    assert!(collect_vowels(&"sequoia".to_string()) == "euoia".to_string());
    assert!(sum_all(&vec![4 as i64, 5 as i64, 6 as i64]) == (15 as i64));
    assert!(head_or_zero(&vec![]) == (0 as i64));
    assert!(head_or_zero(&vec![9 as i64, 1 as i64]) == (9 as i64));
}
