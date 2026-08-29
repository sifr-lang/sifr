// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn collect_vowels(text: &String) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut result: String = "".to_string();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_text.len())) {
        let ch: String = {
    let Some(__indexed_char) = __sifr_chars_text.get(::sifr_runtime::to_usize_proven(&(i))).map(|c| c.to_string()) else {
        unreachable!("compiler-verified string index should be in range");
    };
    __indexed_char
};
        if "aeiou".to_string().contains(&ch) {
            result.push_str((ch).as_str());
        }
        i = &i + &SifrInt::from_i64(1);
    }
    result
}

fn sum_all(values: &Vec<SifrInt>) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), SifrInt::from(values.len()), SifrInt::from_i64(1)) {
        total = &total + &values[::sifr_runtime::to_usize_proven(&(i))].clone();
    }
    total.clone()
}

fn head_or_zero(values: &Vec<SifrInt>) -> SifrInt {
    if &SifrInt::from(values.len()) == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let first: SifrInt = values[::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(0)))].clone();
    first.clone()
}

fn main() {
    assert!((collect_vowels(&"sequoia".to_string()) == "euoia"));
    assert!((&sum_all(&vec![SifrInt::from_i64(4), SifrInt::from_i64(5), SifrInt::from_i64(6)]) == &SifrInt::from_i64(15)));
    assert!((&head_or_zero(&vec![]) == &SifrInt::from_i64(0)));
    assert!((&head_or_zero(&vec![SifrInt::from_i64(9), SifrInt::from_i64(1)]) == &SifrInt::from_i64(9)));
}
