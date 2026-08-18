// src/main.rs
use ::std::collections::HashMap;

fn guarded_lookup(table: &HashMap<i64, i64>, key: i64) -> i64 {
    if !table.contains_key(&key) {
        return -(1_i64);
    }
    let value: i64 = {
    let Some(__sifr_proven_dict_value) = table.get(&key).copied() else {
        ::std::process::abort();
    };
    __sifr_proven_dict_value
};
    value
}

fn expression_lookup(table: &HashMap<i64, i64>, base: i64) -> i64 {
    if (table.keys().cloned().collect::<Vec<_>>()).contains(&(base + (1_i64))) {
        let value: i64 = {
    let Some(__sifr_proven_dict_value) = table.get(&(base + (1_i64))).copied() else {
        ::std::process::abort();
    };
    __sifr_proven_dict_value
};
        return value;
    }
    -(1_i64)
}

fn sum_known_keys(table: &HashMap<i64, i64>, keys: &Vec<i64>) -> i64 {
    let mut total: i64 = 0_i64;
    for key in keys.iter().copied() {
        if table.contains_key(&key) {
            total += {
    let Some(__sifr_proven_dict_value) = table.get(&key).copied() else {
        ::std::process::abort();
    };
    __sifr_proven_dict_value
};
        }
    }
    total
}

fn main() {
    let t: HashMap<i64, i64> = HashMap::from([(1_i64, 10_i64), (2_i64, 20_i64), (4_i64, 40_i64)]);
    assert!((guarded_lookup(&t, 2_i64) == (20_i64)));
    assert!((guarded_lookup(&t, 3_i64) == -(1_i64)));
    assert!((expression_lookup(&t, 1_i64) == (20_i64)));
    assert!((expression_lookup(&t, 2_i64) == -(1_i64)));
    assert!((sum_known_keys(&t, &vec![0_i64, 1_i64, 2_i64, 5_i64]) == (30_i64)));
}
