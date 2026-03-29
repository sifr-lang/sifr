fn collect_vowels(text: &str) -> String {
    let mut result = String::new();
    for ch in text.chars() {
        if "aeiou".contains(ch) {
            result.push(ch);
        }
    }
    result
}

fn sum_all(values: &[i64]) -> i64 {
    values.iter().sum()
}

fn head_or_zero(values: &[i64]) -> i64 {
    values.first().copied().unwrap_or(0)
}

fn main() {
    assert_eq!(collect_vowels("sequoia"), "euoia");
    assert_eq!(sum_all(&[4, 5, 6]), 15);
    assert_eq!(head_or_zero(&[]), 0);
    assert_eq!(head_or_zero(&[9, 1]), 9);
}
