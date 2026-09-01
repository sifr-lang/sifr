// src/main.rs
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
fn sum_forward(nums: &[SifrInt]) -> SifrInt {
    let n: SifrInt = SifrInt::from(nums.len());
    let mut total: SifrInt = SifrInt::from_i64(0);
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), n.clone(), SifrInt::from_i64(1)) {
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
        total = &total + &sifr_generated_checked_value_0.clone();
    }
    total.clone()
}
fn sum_reverse(nums: &[SifrInt]) -> SifrInt {
    let n: SifrInt = SifrInt::from(nums.len());
    let mut total: SifrInt = SifrInt::from_i64(0);
    for i in SifrRange::new_known_nonzero(
        &n - &SifrInt::from_i64(1),
        -SifrInt::from_i64(1),
        -SifrInt::from_i64(1),
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
        total = &total + &sifr_generated_checked_value_1.clone();
    }
    total.clone()
}
fn sum_reverse_while(nums: &[SifrInt]) -> SifrInt {
    let n: SifrInt = SifrInt::from(nums.len());
    let mut i: SifrInt = &n - &SifrInt::from_i64(1);
    let mut total: SifrInt = SifrInt::from_i64(0);
    while &i >= &SifrInt::from_i64(0) {
        let Some(sifr_generated_checked_value_2) = ({
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
        total = &total + &sifr_generated_checked_value_2.clone();
        i = &i - &SifrInt::from_i64(1);
    }
    total.clone()
}
fn append_growth_product(nums: &[SifrInt]) -> SifrInt {
    let n: SifrInt = SifrInt::from(nums.len());
    let mut weights: Vec<SifrInt> = Vec::new();
    for _ in SifrRange::new_known_nonzero(SifrInt::from_i64(0), n.clone(), SifrInt::from_i64(1)) {
        weights.push(SifrInt::from_i64(1));
    }
    let mut i: SifrInt = &n - &SifrInt::from_i64(1);
    let mut product: SifrInt = SifrInt::from_i64(1);
    while &i >= &SifrInt::from_i64(0) {
        let Some(sifr_generated_checked_value_3) = ({
            let sifr_generated_checked_read_collection = &weights;
            let sifr_generated_checked_read_index = i.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }) else {
            break;
        };
        product = &product * &sifr_generated_checked_value_3.clone();
        i = &i - &SifrInt::from_i64(1);
    }
    product.clone()
}
fn main() {
    assert_eq!(
        &sum_forward(&vec![
            SifrInt::from_i64(4),
            SifrInt::from_i64(5),
            SifrInt::from_i64(6)
        ]),
        &SifrInt::from_i64(15)
    );
    assert_eq!(
        &sum_reverse(&vec![
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
            SifrInt::from_i64(3),
            SifrInt::from_i64(4)
        ]),
        &SifrInt::from_i64(10)
    );
    assert_eq!(&sum_reverse(&Vec::new()), &SifrInt::from_i64(0));
    assert_eq!(
        &sum_reverse_while(&vec![
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
            SifrInt::from_i64(3),
            SifrInt::from_i64(4)
        ]),
        &SifrInt::from_i64(10)
    );
    assert_eq!(&sum_reverse_while(&Vec::new()), &SifrInt::from_i64(0));
    assert_eq!(
        &append_growth_product(&vec![
            SifrInt::from_i64(2),
            SifrInt::from_i64(3),
            SifrInt::from_i64(4)
        ]),
        &SifrInt::from_i64(1)
    );
    assert_eq!(&append_growth_product(&Vec::new()), &SifrInt::from_i64(1));
}
