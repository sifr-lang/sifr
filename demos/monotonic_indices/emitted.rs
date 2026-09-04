// src/main.rs
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
fn active_indices(flags: &[bool]) -> Vec<SifrInt> {
    let mut out: Vec<SifrInt> = Vec::new();
    for index in SifrRange::new_known_nonzero(
        SifrInt::from_i64(0),
        SifrInt::from(flags.len()),
        SifrInt::from_i64(1),
    ) {
        let Some(sifr_generated_checked_value_0) = ({
            let sifr_generated_checked_read_collection = &flags;
            let sifr_generated_checked_read_index = index.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }) else {
            continue;
        };
        if sifr_generated_checked_value_0 {
            out.push(index);
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
