// src/main.rs
use ::sifr_runtime::SifrInt;

fn main() {
    let a: Vec<u8> = vec![1u8, 2u8, 3u8];
    let b: Vec<u8> = vec![1u8, 2u8];
    let c: Vec<u8> = {
    let mut __v = (b).clone();
    __v.extend((vec![3u8]).iter().cloned());
    __v
};
    assert!(a == c);
    assert!(&SifrInt::from(a.len()) == &SifrInt::from_i64(3));
    let idx0: Option<u8> = a.get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(0)))).map(|__byte| *__byte as u8);
    let idx1: Option<u8> = a.get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(1)))).map(|__byte| *__byte as u8);
    let idx2: Option<u8> = a.get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(2)))).map(|__byte| *__byte as u8);
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
    let mut acc: SifrInt = SifrInt::from_i64(0);
    let items: Vec<SifrInt> = a.iter().map(|__byte| SifrInt::from(*__byte)).collect::<Vec<SifrInt>>();
    for item in items.iter().cloned() {
        acc = &acc + &item;
    }
    assert!(&acc == &SifrInt::from_i64(6));
}
