// src/main.rs
fn collect_vowels(text: &String) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut result: String = "".to_string();
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_text.len() as i64)) {
        let ch: String = {
    let Some(__indexed_char) = __sifr_chars_text.get(i as usize).map(|c| c.to_string()) else {
        unreachable!("compiler-verified string index should be in range");
    };
    __indexed_char
};
        if "aeiou".to_string().contains(&ch) {
            result.push_str((ch).as_str());
        }
        i += 1_i64;
    }
    result
}

fn sum_all(values: &Vec<i64>) -> i64 {
    let mut total: i64 = 0_i64;
    for i in 0_i64..values.len() as i64 {
        total += values[i as usize];
    }
    total
}

fn head_or_zero(values: &Vec<i64>) -> i64 {
    if (values.len() as i64) == (0_i64) {
        return 0_i64;
    }
    let first: i64 = values[(0_i64) as usize];
    first
}

fn main() {
    assert!((collect_vowels(&"sequoia".to_string()) == "euoia"));
    assert!((sum_all(&vec![4_i64, 5_i64, 6_i64]) == (15_i64)));
    assert!((head_or_zero(&vec![]) == (0_i64)));
    assert!((head_or_zero(&vec![9_i64, 1_i64]) == (9_i64)));
}
