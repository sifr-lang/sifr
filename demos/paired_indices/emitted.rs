// src/main.rs
use ::sifr_runtime::SifrInt;

fn edge_pairs_text(text: &String) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut left: SifrInt = SifrInt::from_i64(0);
    let mut right: SifrInt = &SifrInt::from(__sifr_chars_text.len()) - &SifrInt::from_i64(1);
    let mut out: String = "".to_string();
    while &left < &right {
        out.push('(');
        out.push_str(({
    let Some(__indexed_char) = __sifr_chars_text.get(::sifr_runtime::to_usize_proven(&(left))).map(|c| c.to_string()) else {
        unreachable!("compiler-verified string index should be in range");
    };
    __indexed_char
}).as_str());
        out.push(',');
        out.push_str(({
    let Some(__indexed_char) = __sifr_chars_text.get(::sifr_runtime::to_usize_proven(&(right))).map(|c| c.to_string()) else {
        unreachable!("compiler-verified string index should be in range");
    };
    __indexed_char
}).as_str());
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
