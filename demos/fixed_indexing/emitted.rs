// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn second_or_zero(values: &Vec<SifrInt>) -> SifrInt {
    if &SifrInt::from(values.len()) < &SifrInt::from_i64(2) {
        return SifrInt::from_i64(0);
    }
    values[::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(1)))].clone()
}

fn neighbor_min_cost(cost: &mut Vec<SifrInt>) -> SifrInt {
    if &SifrInt::from(cost.len()) < &SifrInt::from_i64(2) {
        return SifrInt::from_i64(0);
    }
    for i in SifrRange::new_known_nonzero(&SifrInt::from(cost.len()) - &SifrInt::from_i64(3), -(SifrInt::from_i64(1)), -(SifrInt::from_i64(1))) {
        {
            let __assign_value = &cost[::sifr_runtime::to_usize_proven(&(i))].clone() + &::std::cmp::min({
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &cost;
    let __sifr_index_i = &i + &SifrInt::from_i64(1);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
}, {
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &cost;
    let __sifr_index_i = &i + &SifrInt::from_i64(2);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
});
            {
                let __idx_raw = i.clone();
                let __idx_norm = __idx_raw.normalize_index_or_len(cost.len());
                if let Some(__elem) = cost.get_mut(__idx_norm) {
                    *__elem = __assign_value;
                }
            }
        }
    }
    ::std::cmp::min({
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &cost;
    let __sifr_index_i = SifrInt::from_i64(0);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
}, {
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &cost;
    let __sifr_index_i = SifrInt::from_i64(1);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
})
}

fn main() {
    assert!((&second_or_zero(&vec![SifrInt::from_i64(8), SifrInt::from_i64(13)]) == &SifrInt::from_i64(13)));
    assert!((&second_or_zero(&vec![SifrInt::from_i64(8)]) == &SifrInt::from_i64(0)));
    assert!((&neighbor_min_cost(&mut vec![SifrInt::from_i64(10), SifrInt::from_i64(15), SifrInt::from_i64(20)]) == &SifrInt::from_i64(15)));
}
