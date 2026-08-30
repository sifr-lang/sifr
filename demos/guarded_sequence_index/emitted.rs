// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn collect_vowels(text: &String) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut result: String = "".to_string();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_text.len())) {
        let Some(__sifr_checked_value_0) = ({
    let __sifr_string_index = i.clone();
    let __sifr_string_index_normalized = __sifr_string_index.normalize_index_or_len(__sifr_chars_text.len());
    __sifr_chars_text.get(__sifr_string_index_normalized)
}).map(|c| c.to_string()) else {
            break;
        };
        let ch: String = __sifr_checked_value_0.clone();
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
        let Some(__sifr_checked_value_1) = ({
    let __sifr_checked_read_collection = &values;
    let __sifr_checked_read_index = i.clone();
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
            continue;
        };
        total = &total + &__sifr_checked_value_1.clone();
    }
    total.clone()
}

fn head_or_zero(values: &Vec<SifrInt>) -> SifrInt {
    let Some(__sifr_checked_value_2) = ({
    let __sifr_checked_read_collection = &values;
    let __sifr_checked_read_index = SifrInt::from_i64(0);
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
        return SifrInt::from_i64(0);
    };
    let first: SifrInt = __sifr_checked_value_2.clone();
    first.clone()
}

fn main() {
    assert!((collect_vowels(&"sequoia".to_string()) == "euoia"));
    assert!((&sum_all(&vec![SifrInt::from_i64(4), SifrInt::from_i64(5), SifrInt::from_i64(6)]) == &SifrInt::from_i64(15)));
    assert!((&head_or_zero(&vec![]) == &SifrInt::from_i64(0)));
    assert!((&head_or_zero(&vec![SifrInt::from_i64(9), SifrInt::from_i64(1)]) == &SifrInt::from_i64(9)));
}
