fn edge_pairs_text(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut left = 0usize;
    let mut right = chars.len().saturating_sub(1);

    while left < right {
        out.push('(');
        out.push(chars[left]);
        out.push(',');
        out.push(chars[right]);
        out.push(')');
        left += 1;
        right -= 1;
    }

    out
}

fn main() {
    assert_eq!(edge_pairs_text("code"), "(c,e)(o,d)");
    assert_eq!(edge_pairs_text("xy"), "(x,y)");
    println!("paired_indices: ok");
}
