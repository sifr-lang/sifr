// src/main.rs
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
fn second_or_zero(values: &[SifrInt]) -> SifrInt {
    let Some(sifr_generated_checked_value_0) = ({
        let sifr_generated_checked_read_collection = &values;
        let sifr_generated_checked_read_index = SifrInt::from_i64(1);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    }) else {
        return SifrInt::from_i64(0);
    };
    sifr_generated_checked_value_0
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn neighbor_min_cost(cost: &mut Vec<SifrInt>) -> SifrInt {
    let Some(sifr_generated_checked_value_1) = ({
        let sifr_generated_checked_read_collection = &cost;
        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    }) else {
        return SifrInt::from_i64(0);
    };
    let Some(sifr_generated_checked_value_2) = ({
        let sifr_generated_checked_read_collection = &cost;
        let sifr_generated_checked_read_index = SifrInt::from_i64(1);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    }) else {
        return SifrInt::from_i64(0);
    };
    for i in SifrRange::new_known_nonzero(
        ::std::ops::Sub::sub(&SifrInt::from(cost.len()), &SifrInt::from_i64(3)),
        -SifrInt::from_i64(1),
        -SifrInt::from_i64(1),
    ) {
        let Some(sifr_generated_checked_value_3) = ({
            let sifr_generated_checked_read_collection = &cost;
            let sifr_generated_checked_read_index = i.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }) else {
            break;
        };
        let Some(sifr_generated_checked_value_4) = ({
            let sifr_generated_checked_read_collection = &cost;
            let sifr_generated_checked_read_index = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }) else {
            break;
        };
        let Some(sifr_generated_checked_value_5) = ({
            let sifr_generated_checked_read_collection = &cost;
            let sifr_generated_checked_read_index = ::std::ops::Add::add(&i, &SifrInt::from_i64(2));
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }) else {
            break;
        };
        {
            let sifr_generated_assign_value = ::std::ops::Add::add(
                &sifr_generated_checked_value_3,
                &::std::cmp::min(
                    sifr_generated_checked_value_4,
                    sifr_generated_checked_value_5,
                ),
            );
            {
                let sifr_generated_index_raw = i.clone();
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(cost.len());
                if let Some(sifr_generated_elem) = cost.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                }
            }
        }
    }
    let sifr_generated_checked_value_1 = {
        let sifr_generated_checked_read_collection = &cost;
        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    }
    .unwrap_or(sifr_generated_checked_value_1);
    let sifr_generated_checked_value_2 = {
        let sifr_generated_checked_read_collection = &cost;
        let sifr_generated_checked_read_index = SifrInt::from_i64(1);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    }
    .unwrap_or(sifr_generated_checked_value_2);
    ::std::cmp::min(
        sifr_generated_checked_value_1,
        sifr_generated_checked_value_2,
    )
}
fn main() {
    assert_eq!(
        second_or_zero(&[SifrInt::from_i64(8), SifrInt::from_i64(13)]),
        SifrInt::from_i64(13)
    );
    assert_eq!(
        second_or_zero(&[SifrInt::from_i64(8)]),
        SifrInt::from_i64(0)
    );
    assert_eq!(
        neighbor_min_cost(&[
            SifrInt::from_i64(10),
            SifrInt::from_i64(15),
            SifrInt::from_i64(20)
        ]),
        SifrInt::from_i64(15)
    );
}
