// src/main.rs
fn main() {
    let a: Vec<u8> = vec![(1_i64) as u8, (2_i64) as u8, (3_i64) as u8];
    let b: Vec<u8> = vec![(1_i64) as u8, (2_i64) as u8];
    let c: Vec<u8> = {
    let mut __v = (b).clone();
    __v.extend((vec![(3_i64) as u8]).iter().cloned());
    __v
};
    assert!(a == c);
    assert!((a.len() as i64) == (3_i64));
    let idx0: Option<u8> = a.get((0_i64) as usize).map(|__byte| *__byte as u8);
    let idx1: Option<u8> = a.get((1_i64) as usize).map(|__byte| *__byte as u8);
    let idx2: Option<u8> = a.get((2_i64) as usize).map(|__byte| *__byte as u8);
    if let Some(idx0) = idx0 {
        let expected0: u8 = 1u8;
        assert!(idx0 == expected0);
    } else {
        assert!(false);
    }
    if let Some(idx1) = idx1 {
        let expected1: u8 = 2u8;
        assert!(idx1 == expected1);
    } else {
        assert!(false);
    }
    if let Some(idx2) = idx2 {
        let expected2: u8 = 3u8;
        assert!(idx2 == expected2);
    } else {
        assert!(false);
    }
    let mut acc: i64 = 0_i64;
    let items: Vec<i64> = a.iter().map(|__byte| *__byte as i64).collect::<Vec<i64>>();
    for item in items.iter().copied() {
        acc += item;
    }
    assert!(acc == (6_i64));
}
