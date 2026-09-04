// src/main.rs
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::assertions_on_constants,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn main() {
    let a: Vec<u8> = vec![1u8, 2u8, 3u8];
    let b: Vec<u8> = vec![1u8, 2u8];
    let c: Vec<u8> = {
        let mut sifr_generated_v = b;
        sifr_generated_v.extend(vec![3_u8].iter().copied());
        sifr_generated_v
    };
    assert_eq!(a, c);
    assert_eq!(&SifrInt::from(a.len()), &SifrInt::from_i64(3));
    let idx0: Option<u8> = {
        let sifr_generated_checked_read_collection = &a;
        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .copied()
    };
    let idx1_value_9ec3e0c5494675fb: Option<u8> = {
        let sifr_generated_checked_read_collection = &a;
        let sifr_generated_checked_read_index = SifrInt::from_i64(1);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .copied()
    };
    let idx2_value_9ec3e1c5494677ae: Option<u8> = {
        let sifr_generated_checked_read_collection = &a;
        let sifr_generated_checked_read_index = SifrInt::from_i64(2);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .copied()
    };
    if let Some(idx0) = idx0 {
        let expected0: u8 = 1u8;
        assert_eq!(idx0, expected0);
    } else {
        assert!(false);
    }
    if let Some(idx1) = idx1_value_9ec3e0c5494675fb {
        let expected1_value_8741070b69e58438: u8 = 2u8;
        assert_eq!(idx1, expected1_value_8741070b69e58438);
    } else {
        assert!(false);
    }
    if let Some(idx2) = idx2_value_9ec3e1c5494677ae {
        let expected2_value_87410a0b69e58951: u8 = 3u8;
        assert_eq!(idx2, expected2_value_87410a0b69e58951);
    } else {
        assert!(false);
    }
    let mut acc: SifrInt = SifrInt::from_i64(0);
    let items: Vec<SifrInt> = a
        .iter()
        .map(|sifr_generated_byte| SifrInt::from(*sifr_generated_byte))
        .collect::<Vec<SifrInt>>();
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for item in items.iter() {
        acc = ::std::ops::Add::add(&acc, item);
    }
    assert_eq!(&acc, &SifrInt::from_i64(6));
}
