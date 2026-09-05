// src/main.rs
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
fn sum_forward(nums: &[SifrInt]) -> SifrInt {
    let n: SifrInt = SifrInt::from(nums.len());
    let mut total: SifrInt = SifrInt::from_i64(0);
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), n, SifrInt::from_i64(1)) {
        let Some(sifr_generated_checked_value_0) = ({
            let sifr_generated_checked_read_collection = &nums;
            let sifr_generated_checked_read_index = i.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }) else {
            break;
        };
        total = ::std::ops::Add::add(&total, &sifr_generated_checked_value_0);
    }
    total
}
fn sum_reverse(nums: &[SifrInt]) -> SifrInt {
    let n: SifrInt = SifrInt::from(nums.len());
    let mut total: SifrInt = SifrInt::from_i64(0);
    for i in SifrRange::new_known_nonzero(
        ::std::ops::Sub::sub(&n, &SifrInt::from_i64(1)),
        SifrInt::from_i64(-1),
        SifrInt::from_i64(-1),
    ) {
        let Some(sifr_generated_checked_value_1) = ({
            let sifr_generated_checked_read_collection = &nums;
            let sifr_generated_checked_read_index = i.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }) else {
            break;
        };
        total = ::std::ops::Add::add(&total, &sifr_generated_checked_value_1);
    }
    total
}
fn sum_reverse_while(nums: &[SifrInt]) -> SifrInt {
    let n: SifrInt = SifrInt::from(nums.len());
    let mut i: SifrInt = ::std::ops::Sub::sub(&n, &SifrInt::from_i64(1));
    let mut total: SifrInt = SifrInt::from_i64(0);
    while i >= SifrInt::from_i64(0) {
        let Some(sifr_generated_checked_value_2) = ({
            let sifr_generated_checked_read_collection = &nums;
            let sifr_generated_checked_read_index = &i;
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }) else {
            break;
        };
        total = ::std::ops::Add::add(&total, &sifr_generated_checked_value_2);
        i = ::std::ops::Sub::sub(&i, &SifrInt::from_i64(1));
    }
    total
}
fn append_growth_product(nums: &[SifrInt]) -> SifrInt {
    let n: SifrInt = SifrInt::from(nums.len());
    let mut weights: Vec<SifrInt> = Vec::new();
    for _i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), n.clone(), SifrInt::from_i64(1)) {
        weights.push(SifrInt::from_i64(1));
    }
    let mut i: SifrInt = ::std::ops::Sub::sub(&n, &SifrInt::from_i64(1));
    let mut product: SifrInt = SifrInt::from_i64(1);
    while i >= SifrInt::from_i64(0) {
        let Some(sifr_generated_checked_value_3) = ({
            let sifr_generated_checked_read_collection = &weights;
            let sifr_generated_checked_read_index = &i;
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }) else {
            break;
        };
        product = ::std::ops::Mul::mul(&product, &sifr_generated_checked_value_3);
        i = ::std::ops::Sub::sub(&i, &SifrInt::from_i64(1));
    }
    product
}
fn main() {
    assert_eq!(
        sum_forward(&[
            SifrInt::from_i64(4),
            SifrInt::from_i64(5),
            SifrInt::from_i64(6)
        ]),
        SifrInt::from_i64(15)
    );
    assert_eq!(
        sum_reverse(&[
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
            SifrInt::from_i64(3),
            SifrInt::from_i64(4)
        ]),
        SifrInt::from_i64(10)
    );
    assert_eq!(sum_reverse(&[]), SifrInt::from_i64(0));
    assert_eq!(
        sum_reverse_while(&[
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
            SifrInt::from_i64(3),
            SifrInt::from_i64(4)
        ]),
        SifrInt::from_i64(10)
    );
    assert_eq!(sum_reverse_while(&[]), SifrInt::from_i64(0));
    assert_eq!(
        append_growth_product(&[
            SifrInt::from_i64(2),
            SifrInt::from_i64(3),
            SifrInt::from_i64(4)
        ]),
        SifrInt::from_i64(1)
    );
    assert_eq!(append_growth_product(&[]), SifrInt::from_i64(1));
}
