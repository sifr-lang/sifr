// src/main.rs
use ::sifr_runtime::SifrInt;
fn total(data: &[u8]) -> SifrInt {
    let mut out: SifrInt = SifrInt::from_i64(0);
    let values: Vec<SifrInt> = data
        .iter()
        .map(|sifr_generated_byte| SifrInt::from(*sifr_generated_byte))
        .collect::<Vec<SifrInt>>();
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for value in values.iter() {
        out = ::std::ops::Add::add(&out, value);
    }
    out
}
#[expect(
    clippy::assertions_on_constants,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn main() {
    let payload: Vec<u8> = vec![115u8, 105u8, 102u8, 114u8];
    let suffix: Vec<u8> = vec![0u8, 1u8];
    let combined: Vec<u8> = {
        let mut sifr_generated_v = payload;
        sifr_generated_v.extend(suffix.iter().copied());
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
            .copied()
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
                .copied(),
        )
    };
    assert_eq!(total(&window), SifrInt::from_i64(321));
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
