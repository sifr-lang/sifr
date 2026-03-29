fn write_indices(size: i64) -> Vec<i64> {
    let mut out = vec![0_i64; size as usize];
    for i in 0..out.len() {
        out[i] = i as i64 + 1;
    }
    out
}

fn main() {
    assert_eq!(format!("{:?}", write_indices(4)), "[1, 2, 3, 4]");
    assert_eq!(format!("{:?}", write_indices(0)), "[]");
    println!("indexed_tables: ok");
}
