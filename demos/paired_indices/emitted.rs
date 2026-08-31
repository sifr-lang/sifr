// src/main.rs
use ::sifr_runtime::SifrInt;

fn edge_pairs_text(text: &str) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut left: SifrInt = SifrInt::from_i64(0);
    let mut right: SifrInt = &SifrInt::from(__sifr_chars_text.len()) - &SifrInt::from_i64(1);
    let mut out: String = "".to_string();
    while (&left < &right) {
        let Some(__sifr_checked_value_0) = ({
    let __sifr_string_index = left.clone();
    let __sifr_string_index_normalized = __sifr_string_index.normalize_index_or_len(__sifr_chars_text.len());
    __sifr_chars_text.get(__sifr_string_index_normalized)
}).map(|c| c.to_string()) else {
            break;
        };
        let Some(__sifr_checked_value_1) = ({
    let __sifr_string_index = right.clone();
    let __sifr_string_index_normalized = __sifr_string_index.normalize_index_or_len(__sifr_chars_text.len());
    __sifr_chars_text.get(__sifr_string_index_normalized)
}).map(|c| c.to_string()) else {
            break;
        };
        out.push('(');
        out.push_str(__sifr_checked_value_0.clone().as_str());
        out.push(',');
        out.push_str(__sifr_checked_value_1.clone().as_str());
        out.push(')');
        left = &left + &SifrInt::from_i64(1);
        right = &right - &SifrInt::from_i64(1);
    }
    out
}

fn main() {
    assert!((edge_pairs_text(&"code".to_string()) == "(c,e)(o,d)"));
    assert!((edge_pairs_text(&"xy".to_string()) == "(x,y)"));
    println!("paired_indices: ok");
}
