// src/main.rs
use ::std::collections::HashMap;

use ::sifr_runtime::SifrInt;

fn guarded_lookup(table: &HashMap<SifrInt, SifrInt>, key: SifrInt) -> SifrInt {
    if !table.contains_key(&key) {
        return -&SifrInt::from_i64(1);
    }
    let value: SifrInt = table[&key].clone();
    value.clone()
}

fn expression_lookup(table: &HashMap<SifrInt, SifrInt>, base: SifrInt) -> SifrInt {
    if (table.keys().cloned().collect::<Vec<_>>()).contains(&(&base + &SifrInt::from_i64(1))) {
        let value: SifrInt = table[&(&base + &SifrInt::from_i64(1))].clone();
        return value.clone();
    }
    -&SifrInt::from_i64(1)
}

fn sum_known_keys(table: &HashMap<SifrInt, SifrInt>, keys: &Vec<SifrInt>) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for key in keys.iter().cloned() {
        if table.contains_key(&key) {
            total = &total + &table[&key].clone();
        }
    }
    total.clone()
}

fn main() {
    let t: HashMap<SifrInt, SifrInt> = HashMap::from([(SifrInt::from_i64(1), SifrInt::from_i64(10)), (SifrInt::from_i64(2), SifrInt::from_i64(20)), (SifrInt::from_i64(4), SifrInt::from_i64(40))]);
    assert!((&guarded_lookup(&t, SifrInt::from_i64(2)) == &SifrInt::from_i64(20)));
    assert!((&guarded_lookup(&t, SifrInt::from_i64(3)) == &-(SifrInt::from_i64(1))));
    assert!((&expression_lookup(&t, SifrInt::from_i64(1)) == &SifrInt::from_i64(20)));
    assert!((&expression_lookup(&t, SifrInt::from_i64(2)) == &-(SifrInt::from_i64(1))));
    assert!((&sum_known_keys(&t, &vec![SifrInt::from_i64(0), SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(5)]) == &SifrInt::from_i64(30)));
}
