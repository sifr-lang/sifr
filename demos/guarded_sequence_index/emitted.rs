// src/main.rs
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
fn collect_vowels(text: &str) -> String {
    let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut result: String = String::new();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while i < sifr_generated_chars_text.len() {
        let Some(sifr_generated_checked_value_0) = {
            let sifr_generated_string_index = &i;
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_text.len());
            sifr_generated_chars_text
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            break;
        };
        let ch: String = sifr_generated_checked_value_0;
        if "aeiou".to_string().contains(&ch) {
            result.push_str(ch.as_str());
        }
        i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
    }
    result
}
fn sum_all(values: &[SifrInt]) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for i in SifrRange::new_known_nonzero(
        SifrInt::from_i64(0),
        SifrInt::from(values.len()),
        SifrInt::from_i64(1),
    ) {
        let Some(sifr_generated_checked_value_1) = ({
            let sifr_generated_checked_read_collection = &values;
            let sifr_generated_checked_read_index = i.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }) else {
            continue;
        };
        total = ::std::ops::Add::add(&total, &sifr_generated_checked_value_1);
    }
    total
}
fn head_or_zero(values: &[SifrInt]) -> SifrInt {
    let Some(sifr_generated_checked_value_2) = ({
        let sifr_generated_checked_read_collection = &values;
        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    }) else {
        return SifrInt::from_i64(0);
    };
    sifr_generated_checked_value_2
}
fn main() {
    assert_eq!(collect_vowels("sequoia"), "euoia");
    assert_eq!(
        sum_all(&[
            SifrInt::from_i64(4),
            SifrInt::from_i64(5),
            SifrInt::from_i64(6)
        ]),
        SifrInt::from_i64(15)
    );
    assert_eq!(head_or_zero(&[]), SifrInt::from_i64(0));
    assert_eq!(
        head_or_zero(&[SifrInt::from_i64(9), SifrInt::from_i64(1)]),
        SifrInt::from_i64(9)
    );
}
