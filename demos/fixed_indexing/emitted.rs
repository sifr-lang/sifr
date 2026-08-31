// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn second_or_zero(values: &Vec<SifrInt>) -> SifrInt {
    let Some(__sifr_checked_value_0) = ({
    let __sifr_checked_read_collection = &values;
    let __sifr_checked_read_index = SifrInt::from_i64(1);
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
        return SifrInt::from_i64(0);
    };
    __sifr_checked_value_0.clone()
}

fn neighbor_min_cost(cost: &mut Vec<SifrInt>) -> SifrInt {
    let Some(__sifr_checked_value_1) = ({
    let __sifr_checked_read_collection = &cost;
    let __sifr_checked_read_index = SifrInt::from_i64(0);
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
        return SifrInt::from_i64(0);
    };
    let Some(__sifr_checked_value_2) = ({
    let __sifr_checked_read_collection = &cost;
    let __sifr_checked_read_index = SifrInt::from_i64(1);
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
        return SifrInt::from_i64(0);
    };
    for i in SifrRange::new_known_nonzero(&SifrInt::from(cost.len()) - &SifrInt::from_i64(3), -(SifrInt::from_i64(1)), -(SifrInt::from_i64(1))) {
        let Some(__sifr_checked_value_3) = ({
    let __sifr_checked_read_collection = &cost;
    let __sifr_checked_read_index = i.clone();
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
            break;
        };
        let Some(__sifr_checked_value_4) = ({
    let __sifr_checked_read_collection = &cost;
    let __sifr_checked_read_index = &i + &SifrInt::from_i64(1);
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
            break;
        };
        let Some(__sifr_checked_value_5) = ({
    let __sifr_checked_read_collection = &cost;
    let __sifr_checked_read_index = &i + &SifrInt::from_i64(2);
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
            break;
        };
        {
            let __assign_value = &__sifr_checked_value_3.clone() + &::std::cmp::min(__sifr_checked_value_4.clone(), __sifr_checked_value_5.clone());
            {
                let __index_raw = i.clone();
                let __index_normalized = __index_raw.normalize_index_or_len(cost.len());
                if let Some(__elem) = cost.get_mut(__index_normalized) {
                    *__elem = __assign_value;
                }
            }
        }
    }
    let __sifr_checked_value_1 = ({
    let __sifr_checked_read_collection = &cost;
    let __sifr_checked_read_index = SifrInt::from_i64(0);
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}).unwrap_or(__sifr_checked_value_1);
    let __sifr_checked_value_2 = ({
    let __sifr_checked_read_collection = &cost;
    let __sifr_checked_read_index = SifrInt::from_i64(1);
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}).unwrap_or(__sifr_checked_value_2);
    ::std::cmp::min(__sifr_checked_value_1.clone(), __sifr_checked_value_2.clone())
}

fn main() {
    assert!((&second_or_zero(&vec![SifrInt::from_i64(8), SifrInt::from_i64(13)]) == &SifrInt::from_i64(13)));
    assert!((&second_or_zero(&vec![SifrInt::from_i64(8)]) == &SifrInt::from_i64(0)));
    assert!((&neighbor_min_cost(&mut vec![SifrInt::from_i64(10), SifrInt::from_i64(15), SifrInt::from_i64(20)]) == &SifrInt::from_i64(15)));
}
