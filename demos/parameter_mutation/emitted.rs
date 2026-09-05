// src/main.rs
use ::sifr_runtime::SifrInt;
fn mutate_and_return(mut items: Vec<SifrInt>) -> Vec<SifrInt> {
    if &SifrInt::from(items.len()) > &SifrInt::from_i64(1) {
        {
            let sifr_generated_assign_value = SifrInt::from_i64(9);
            {
                let sifr_generated_index_raw = SifrInt::from_i64(0);
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(items.len());
                if let Some(sifr_generated_elem) = items.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                }
            }
        }
        {
            let sifr_generated_assign_value = SifrInt::from_i64(10);
            {
                let sifr_generated_index_raw = SifrInt::from_i64(1);
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(items.len());
                if let Some(sifr_generated_elem) = items.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                }
            }
        }
    }
    items
}
fn mutate_borrowed(items: &mut Vec<SifrInt>) -> SifrInt {
    if &SifrInt::from(items.len()) > &SifrInt::from_i64(0) {
        {
            let sifr_generated_assign_value = SifrInt::from_i64(14);
            {
                let sifr_generated_index_raw = SifrInt::from_i64(0);
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(items.len());
                if let Some(sifr_generated_elem) = items.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                }
            }
        }
    }
    SifrInt::from(items.len())
}
fn main() {
    let values: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
    ];
    let mut moved: Vec<SifrInt> = mutate_and_return(values);
    println!(
        "{}",
        {
            let sifr_generated_index_list = &moved;
            let sifr_generated_index_i = SifrInt::from_i64(0);
            let sifr_generated_index_norm =
                sifr_generated_index_i.normalize_index_or_len(sifr_generated_index_list.len());
            sifr_generated_index_list
                .get(sifr_generated_index_norm)
                .cloned()
        }
        .map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    println!(
        "{}",
        {
            let sifr_generated_index_list = &moved;
            let sifr_generated_index_i = SifrInt::from_i64(1);
            let sifr_generated_index_norm =
                sifr_generated_index_i.normalize_index_or_len(sifr_generated_index_list.len());
            sifr_generated_index_list
                .get(sifr_generated_index_norm)
                .cloned()
        }
        .map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    println!("{}", mutate_borrowed(&mut moved));
}
