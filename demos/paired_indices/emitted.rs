// src/main.rs
use ::sifr_runtime::SifrInt;
fn edge_pairs_text(text: &str) -> String {
    let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut left: SifrInt = SifrInt::from_i64(0);
    let mut right: SifrInt =
        &SifrInt::from(sifr_generated_chars_text.len()) - &SifrInt::from_i64(1);
    let mut out: String = String::new();
    while &left < &right {
        let Some(sifr_generated_checked_value_0) = {
            let sifr_generated_string_index = left.clone();
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_text.len());
            sifr_generated_chars_text
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            break;
        };
        let Some(sifr_generated_checked_value_1) = {
            let sifr_generated_string_index = right.clone();
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_text.len());
            sifr_generated_chars_text
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            break;
        };
        out.push('(');
        out.push_str(sifr_generated_checked_value_0.clone().as_str());
        out.push(',');
        out.push_str(sifr_generated_checked_value_1.clone().as_str());
        out.push(')');
        left = &left + &SifrInt::from_i64(1);
        right = &right - &SifrInt::from_i64(1);
    }
    out
}
fn main() {
    assert_eq!(edge_pairs_text(&"code".to_string()), "(c,e)(o,d)");
    assert_eq!(edge_pairs_text(&"xy".to_string()), "(x,y)");
    println!("paired_indices: ok");
}
