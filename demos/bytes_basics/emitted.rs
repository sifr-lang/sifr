// src/main.rs
use ::sifr_runtime::SifrInt;
fn total(data: &[u8]) -> SifrInt {
    let mut out: SifrInt = SifrInt::from_i64(0);
    let values: Vec<SifrInt> = data
        .iter()
        .map(|sifr_generated_byte| SifrInt::from(*sifr_generated_byte))
        .collect::<Vec<SifrInt>>();
    for value in values.iter().cloned() {
        out = &out + &value;
    }
    out.clone()
}
#[expect(
    clippy::assertions_on_constants,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn main() {
    let payload: Vec<u8> = vec![115u8, 105u8, 102u8, 114u8];
    let suffix: Vec<u8> = vec![0u8, 1u8];
    let combined: Vec<u8> = {
        let mut sifr_generated_v = payload.to_vec();
        sifr_generated_v.extend(suffix.iter().cloned());
        sifr_generated_v
    };
    assert_eq!(&SifrInt::from(combined.len()), &SifrInt::from_i64(6));
    let head: Option<u8> = {
        let sifr_generated_checked_read_collection = &combined;
        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    };
    if let Some(head) = head {
        let expected_head: u8 = 115u8;
        assert_eq!(head, expected_head);
    } else {
        assert!(false);
    }
    let window: Vec<u8> = {
        let sifr_generated_slice_src = &combined;
        let sifr_generated_slice_len = sifr_generated_slice_src.len();
        let sifr_generated_slice_start =
            SifrInt::from_i64(1).clamp_slice_bound(sifr_generated_slice_len);
        let sifr_generated_slice_stop =
            SifrInt::from_i64(4).clamp_slice_bound(sifr_generated_slice_len);
        Vec::from_iter(
            sifr_generated_slice_src
                .iter()
                .skip(sifr_generated_slice_start)
                .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                .cloned(),
        )
    };
    assert_eq!(&total(&window), &SifrInt::from_i64(321));
    let raw: Vec<SifrInt> = window
        .iter()
        .map(|sifr_generated_byte| SifrInt::from(*sifr_generated_byte))
        .collect::<Vec<SifrInt>>();
    assert_eq!(
        raw,
        vec![
            SifrInt::from_i64(105),
            SifrInt::from_i64(102),
            SifrInt::from_i64(114)
        ]
    );
}
