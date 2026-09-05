// src/main.rs
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
fn write_indices(size: &SifrInt) -> Vec<SifrInt> {
    let mut out: Vec<SifrInt> = {
        let mut sifr_generated_list_comp = Vec::new();
        for _i in SifrRange::new_known_nonzero(
            SifrInt::from_i64(0),
            (*size).clone(),
            SifrInt::from_i64(1),
        ) {
            sifr_generated_list_comp.push(SifrInt::from_i64(0));
        }
        sifr_generated_list_comp
    };
    for i in SifrRange::new_known_nonzero(
        SifrInt::from_i64(0),
        SifrInt::from(out.len()),
        SifrInt::from_i64(1),
    ) {
        {
            let sifr_generated_assign_value = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
            {
                let sifr_generated_index_raw = &i;
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(out.len());
                if let Some(sifr_generated_elem) = out.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                }
            }
        }
    }
    out
}
fn main() {
    assert_eq!(
        format!("{:?}", write_indices(&SifrInt::from_i64(4))),
        "[1, 2, 3, 4]"
    );
    assert_eq!(format!("{:?}", write_indices(&SifrInt::from_i64(0))), "[]");
    println!("indexed_tables: ok");
}
