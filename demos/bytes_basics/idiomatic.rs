fn total(data: &[u8]) -> i64 {
    data.iter().map(|&value| i64::from(value)).sum()
}

fn main() {
    let payload = b"sifr";
    let suffix = b"\x00\x01";

    let mut combined = payload.to_vec();
    combined.extend_from_slice(suffix);

    assert_eq!(combined.len(), 6);
    assert_eq!(combined.first().copied(), Some(115));

    let window = &combined[1..4];
    assert_eq!(total(window), 321);

    let raw: Vec<i64> = window.iter().map(|&value| i64::from(value)).collect();
    assert_eq!(raw, vec![105, 102, 114]);
}
