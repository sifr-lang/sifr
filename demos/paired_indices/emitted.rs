// src/main.rs
fn edge_pairs_text(text: &String) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut left: i64 = 0_i64;
    let mut right: i64 = (__sifr_chars_text.len() as i64) - (1_i64);
    let mut out: String = "".to_string();
    while left < right {
        out.push('(');
        out.push_str(({
    let Some(__indexed_char) = __sifr_chars_text.get(left as usize).map(|c| c.to_string()) else {
        unreachable!("compiler-verified string index should be in range");
    };
    __indexed_char
}).as_str());
        out.push(',');
        out.push_str(({
    let Some(__indexed_char) = __sifr_chars_text.get(right as usize).map(|c| c.to_string()) else {
        unreachable!("compiler-verified string index should be in range");
    };
    __indexed_char
}).as_str());
        out.push(')');
        left += 1_i64;
        right -= 1_i64;
    }
    out
}

fn main() {
    assert!((edge_pairs_text(&"code".to_string()) == "(c,e)(o,d)"));
    assert!((edge_pairs_text(&"xy".to_string()) == "(x,y)"));
    println!("paired_indices: ok");
}
