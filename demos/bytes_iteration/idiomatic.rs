fn main() {
    let a = b"\x01\x02\x03";
    let b = b"\x01\x02";
    let mut c = b.to_vec();
    c.push(3);

    assert_eq!(a, c.as_slice());
    assert_eq!(a.len(), 3);

    let idx0 = a.get(0).copied().map(i64::from);
    let idx1 = a.get(1).copied().map(i64::from);
    let idx2 = a.get(2).copied().map(i64::from);
    assert_eq!(idx0, Some(1));
    assert_eq!(idx1, Some(2));
    assert_eq!(idx2, Some(3));

    let acc: i64 = a.iter().map(|&byte| i64::from(byte)).sum();
    assert_eq!(acc, 6);
}
