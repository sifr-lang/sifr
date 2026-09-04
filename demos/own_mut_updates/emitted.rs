// src/main.rs
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
fn increment_all(mut values: Vec<SifrInt>) -> Vec<SifrInt> {
    for i in SifrRange::new_known_nonzero(
        SifrInt::from_i64(0),
        SifrInt::from(values.len()),
        SifrInt::from_i64(1),
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
            continue;
        };
        {
            let sifr_generated_assign_value =
                ::std::ops::Add::add(&sifr_generated_checked_value_0, &SifrInt::from_i64(1));
            {
                let sifr_generated_index_raw = i.clone();
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(values.len());
                if let Some(sifr_generated_elem) = values.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                }
            }
        }
    }
    values
}
fn clear_all(mut values: Vec<SifrInt>) -> Vec<SifrInt> {
    for i in SifrRange::new_known_nonzero(
        SifrInt::from_i64(0),
        SifrInt::from(values.len()),
        SifrInt::from_i64(1),
    ) {
        {
            let sifr_generated_assign_value = SifrInt::from_i64(0);
            {
                let sifr_generated_index_raw = i.clone();
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(values.len());
                if let Some(sifr_generated_elem) = values.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                }
            }
        }
    }
    values
}
fn main() {
    assert_eq!(
        format!(
            "{:?}",
            increment_all(vec![
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3)
            ])
        ),
        "[2, 3, 4]"
    );
    assert_eq!(
        format!(
            "{:?}",
            clear_all(vec![
                SifrInt::from_i64(4),
                SifrInt::from_i64(5),
                SifrInt::from_i64(6)
            ])
        ),
        "[0, 0, 0]"
    );
    println!("own_mut_updates: ok");
}
