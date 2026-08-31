// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn sum_forward(nums: &[SifrInt]) -> SifrInt {
    let n: SifrInt = SifrInt::from(nums.len());
    let mut total: SifrInt = SifrInt::from_i64(0);
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), n.clone(), SifrInt::from_i64(1)) {
        let Some(__sifr_checked_value_0) = ({
    let __sifr_checked_read_collection = &nums;
    let __sifr_checked_read_index = i.clone();
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
            break;
        };
        total = &total + &__sifr_checked_value_0.clone();
    }
    total.clone()
}

fn sum_reverse(nums: &[SifrInt]) -> SifrInt {
    let n: SifrInt = SifrInt::from(nums.len());
    let mut total: SifrInt = SifrInt::from_i64(0);
    for i in SifrRange::new_known_nonzero(&n - &SifrInt::from_i64(1), -(SifrInt::from_i64(1)), -(SifrInt::from_i64(1))) {
        let Some(__sifr_checked_value_1) = ({
    let __sifr_checked_read_collection = &nums;
    let __sifr_checked_read_index = i.clone();
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
            break;
        };
        total = &total + &__sifr_checked_value_1.clone();
    }
    total.clone()
}

fn sum_reverse_while(nums: &[SifrInt]) -> SifrInt {
    let n: SifrInt = SifrInt::from(nums.len());
    let mut i: SifrInt = &n - &SifrInt::from_i64(1);
    let mut total: SifrInt = SifrInt::from_i64(0);
    while (&i >= &SifrInt::from_i64(0)) {
        let Some(__sifr_checked_value_2) = ({
    let __sifr_checked_read_collection = &nums;
    let __sifr_checked_read_index = i.clone();
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
            break;
        };
        total = &total + &__sifr_checked_value_2.clone();
        i = &i - &SifrInt::from_i64(1);
    }
    total.clone()
}

fn append_growth_product(nums: &[SifrInt]) -> SifrInt {
    let n: SifrInt = SifrInt::from(nums.len());
    let mut weights: Vec<SifrInt> = vec![];
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), n.clone(), SifrInt::from_i64(1)) {
        weights.push(SifrInt::from_i64(1));
    }
    let mut i: SifrInt = &n - &SifrInt::from_i64(1);
    let mut product: SifrInt = SifrInt::from_i64(1);
    while (&i >= &SifrInt::from_i64(0)) {
        let Some(__sifr_checked_value_3) = ({
    let __sifr_checked_read_collection = &weights;
    let __sifr_checked_read_index = i.clone();
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
            break;
        };
        product = &product * &__sifr_checked_value_3.clone();
        i = &i - &SifrInt::from_i64(1);
    }
    product.clone()
}

fn main() {
    assert!((&sum_forward(&vec![SifrInt::from_i64(4), SifrInt::from_i64(5), SifrInt::from_i64(6)]) == &SifrInt::from_i64(15)));
    assert!((&sum_reverse(&vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4)]) == &SifrInt::from_i64(10)));
    assert!((&sum_reverse(&vec![]) == &SifrInt::from_i64(0)));
    assert!((&sum_reverse_while(&vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4)]) == &SifrInt::from_i64(10)));
    assert!((&sum_reverse_while(&vec![]) == &SifrInt::from_i64(0)));
    assert!((&append_growth_product(&vec![SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4)]) == &SifrInt::from_i64(1)));
    assert!((&append_growth_product(&vec![]) == &SifrInt::from_i64(1)));
}
