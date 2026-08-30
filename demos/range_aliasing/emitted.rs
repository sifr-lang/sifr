// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn sum_forward(nums: &Vec<SifrInt>) -> SifrInt {
    let n: SifrInt = SifrInt::from(nums.len());
    let mut total: SifrInt = SifrInt::from_i64(0);
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), n.clone(), SifrInt::from_i64(1)) {
        total = &total + &nums[::sifr_runtime::to_usize_proven(&(i))].clone();
    }
    total.clone()
}

fn sum_reverse(nums: &Vec<SifrInt>) -> SifrInt {
    let n: SifrInt = SifrInt::from(nums.len());
    let mut total: SifrInt = SifrInt::from_i64(0);
    for i in SifrRange::new_known_nonzero(&n - &SifrInt::from_i64(1), -(SifrInt::from_i64(1)), -(SifrInt::from_i64(1))) {
        total = &total + &nums[::sifr_runtime::to_usize_proven(&(i))].clone();
    }
    total.clone()
}

fn sum_reverse_while(nums: &Vec<SifrInt>) -> SifrInt {
    let n: SifrInt = SifrInt::from(nums.len());
    let mut i: SifrInt = &n - &SifrInt::from_i64(1);
    let mut total: SifrInt = SifrInt::from_i64(0);
    while &i >= &SifrInt::from_i64(0) {
        total = &total + &nums[::sifr_runtime::to_usize_proven(&(i))].clone();
        i = &i - &SifrInt::from_i64(1);
    }
    total.clone()
}

fn append_growth_product(nums: &Vec<SifrInt>) -> SifrInt {
    let n: SifrInt = SifrInt::from(nums.len());
    let mut weights: Vec<SifrInt> = vec![];
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), n.clone(), SifrInt::from_i64(1)) {
        weights.push(SifrInt::from_i64(1));
    }
    let mut i: SifrInt = &n - &SifrInt::from_i64(1);
    let mut product: SifrInt = SifrInt::from_i64(1);
    while &i >= &SifrInt::from_i64(0) {
        product = &product * &weights[::sifr_runtime::to_usize_proven(&(i))].clone();
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
