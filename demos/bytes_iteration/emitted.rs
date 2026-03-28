fn main() {
    let a: Vec<u8> = vec![(1 as i64) as u8, (2 as i64) as u8, (3 as i64) as u8];
    let b: Vec<u8> = vec![(1 as i64) as u8, (2 as i64) as u8];
    let c: Vec<u8> = {
    let mut __v = (b).clone();
    __v.extend((vec![(3 as i64) as u8]).iter().cloned());
    __v
};
    assert!(a == c);
    assert!((a.len() as i64) == (3 as i64));
    let idx0: Option<i64> = a.get((0 as i64) as usize).map(|__byte| *__byte as i64);
    let idx1: Option<i64> = a.get((1 as i64) as usize).map(|__byte| *__byte as i64);
    let idx2: Option<i64> = a.get((2 as i64) as usize).map(|__byte| *__byte as i64);
    assert!(idx0 == Some(1 as i64));
    assert!(idx1 == Some(2 as i64));
    assert!(idx2 == Some(3 as i64));
    let mut acc: i64 = 0 as i64;
    for item in a.iter().map(|__byte| *__byte as i64) {
        acc += item;
    }
    assert!(acc == (6 as i64));
}
