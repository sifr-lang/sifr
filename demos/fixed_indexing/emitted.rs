// src/main.rs
fn second_or_zero(values: &Vec<i64>) -> i64 {
    if (values.len() as i64) < (2_i64) {
        return 0_i64;
    }
    values[(1_i64) as usize]
}

fn neighbor_min_cost(cost: &mut Vec<i64>) -> i64 {
    if (cost.len() as i64) < (2_i64) {
        return 0_i64;
    }
    for i in (-(1_i64) + (1_i64)..((cost.len() as i64) - (3_i64)) + (1_i64)).rev() {
        {
            let __assign_value = cost[i as usize] + std::cmp::min({
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &cost;
    let __sifr_index_i = i + (1_i64);
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
}, {
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &cost;
    let __sifr_index_i = i + (2_i64);
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
});
            {
                let __idx_raw = i;
                let __idx_norm = if __idx_raw < 0 { (cost.len() as i64) + __idx_raw } else { __idx_raw };
                if __idx_norm >= 0 {
                    if let Some(__elem) = cost.get_mut(__idx_norm as usize) {
                        *__elem = __assign_value;
                    }
                }
            }
        }
    }
    std::cmp::min({
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &cost;
    let __sifr_index_i = 0_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
}, {
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &cost;
    let __sifr_index_i = 1_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
})
}

fn main() {
    assert!((second_or_zero(&vec![8_i64, 13_i64]) == (13_i64)));
    assert!((second_or_zero(&vec![8_i64]) == (0_i64)));
    assert!((neighbor_min_cost(&mut vec![10_i64, 15_i64, 20_i64]) == (15_i64)));
}
