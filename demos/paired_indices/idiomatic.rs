fn edge_pairs_text(text: &String) -> String {
    let mut left: i64 = 0 as i64;
    let mut right: i64 = (text.chars().count() as i64) - (1 as i64);
    let mut out: String = "".to_string();
    while left < right {
        out = format!(
            "{}{}{}{}{}{}",
            out,
            "(".to_string(),
            {
                let Some(__indexed_char) = text.chars().nth(left as usize) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char.to_string()
            },
            ",".to_string(),
            {
                let Some(__indexed_char) = text.chars().nth(right as usize) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char.to_string()
            },
            ")".to_string()
        );
        left += 1 as i64;
        right -= 1 as i64;
    }
    return out;
}

fn main() {
    assert!(edge_pairs_text(&"code".to_string()) == "(c,e)(o,d)".to_string());
    assert!(edge_pairs_text(&"xy".to_string()) == "(x,y)".to_string());
    println!("paired_indices: ok");
}
