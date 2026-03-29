fn reversed_values(values: &[i64]) -> Vec<i64> {
    let mut out = Vec::new();
    for i in (0..values.len()).rev() {
        out.push(values[i]);
    }
    out
}

fn main() {
    assert_eq!(
        format!("{:?}", reversed_values(&[4_i64, 5, 6])),
        "[6, 5, 4]"
    );
    assert_eq!(format!("{:?}", reversed_values(&[])), "[]");
    println!("reverse_indices: ok");
}
