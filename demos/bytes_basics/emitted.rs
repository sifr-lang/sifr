// src/main.rs
use ::sifr_runtime::SifrInt;

fn total(data: &[u8]) -> SifrInt {
    let mut out: SifrInt = SifrInt::from_i64(0);
    let values: Vec<SifrInt> = data.iter().map(|__byte| SifrInt::from(*__byte)).collect::<Vec<SifrInt>>();
    for value in values.iter().cloned() {
        out = &out + &value;
    }
    out.clone()
}

fn main() {
    let payload: Vec<u8> = vec![115u8, 105u8, 102u8, 114u8];
    let suffix: Vec<u8> = vec![0u8, 1u8];
    let combined: Vec<u8> = {
    let mut __v = (payload).to_vec();
    __v.extend((suffix).iter().cloned());
    __v
};
    assert!(&SifrInt::from(combined.len()) == &SifrInt::from_i64(6));
    let head: Option<u8> = {
    let __sifr_checked_read_collection = &combined;
    let __sifr_checked_read_index = SifrInt::from_i64(0);
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
};
    if let Some(head) = head {
        let expected_head: u8 = 115u8;
        assert!(head == expected_head);
    } else {
        assert!(false);
    }
    let window: Vec<u8> = {
    let _slice_src = &combined;
    let _slice_len = _slice_src.len();
    let _slice_start = SifrInt::from_i64(1).clamp_slice_bound(_slice_len);
    let _slice_stop = SifrInt::from_i64(4).clamp_slice_bound(_slice_len);
    Vec::from_iter(_slice_src.iter().skip(_slice_start).take(_slice_stop.saturating_sub(_slice_start)).cloned())
};
    assert!((&total(&window) == &SifrInt::from_i64(321)));
    let raw: Vec<SifrInt> = window.iter().map(|__byte| SifrInt::from(*__byte)).collect::<Vec<SifrInt>>();
    assert!((raw == vec![SifrInt::from_i64(105), SifrInt::from_i64(102), SifrInt::from_i64(114)]));
}
