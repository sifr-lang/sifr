// src/main.rs
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
fn reversed_values(values: &[SifrInt]) -> Vec<SifrInt> {
    let mut out: Vec<SifrInt> = Vec::new();
    for i in SifrRange::new_known_nonzero(
        ::std::ops::Sub::sub(&SifrInt::from(values.len()), &SifrInt::from_i64(1)),
        -SifrInt::from_i64(1),
        -SifrInt::from_i64(1),
    ) {
        let Some(sifr_generated_checked_value_0) = ({
            let sifr_generated_checked_read_collection = &values;
            let sifr_generated_checked_read_index = i.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }) else {
            break;
        };
        out.push(sifr_generated_checked_value_0);
    }
    out
}
fn main() {
    assert_eq!(
        format!(
            "{:?}",
            reversed_values(&[
                SifrInt::from_i64(4),
                SifrInt::from_i64(5),
                SifrInt::from_i64(6)
            ])
        ),
        "[6, 5, 4]"
    );
    assert_eq!(format!("{:?}", reversed_values(&Vec::new())), "[]");
    println!("reverse_indices: ok");
}
