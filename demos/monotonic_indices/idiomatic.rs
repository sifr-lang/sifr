fn active_indices(flags: &[bool]) -> Vec<i64> {
    let mut out = Vec::new();
    for index in 0..flags.len() {
        if flags[index] {
            out.push(index as i64);
        }
    }
    out
}

fn main() {
    assert_eq!(
        format!("{:?}", active_indices(&[true, false, true, true])),
        "[0, 2, 3]"
    );
    assert_eq!(format!("{:?}", active_indices(&[false, false])), "[]");
    println!("monotonic_indices: ok");
}
