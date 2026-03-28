use std::collections::HashMap;

fn guarded_lookup(table: &HashMap<i64, i64>, key: i64) -> i64 {
    if !table.contains_key(&key) {
        return -(1 as i64);
    }
    let value: i64 = table
        .get(&key)
        .copied()
        .expect(&"dict index proven by guard".to_string());
    return value;
}

fn expression_lookup(table: &HashMap<i64, i64>, base: i64) -> i64 {
    if (table.keys().cloned().collect::<Vec<_>>()).contains(&(base + (1 as i64))) {
        let value: i64 = table
            .get(&(base + (1 as i64)))
            .copied()
            .expect(&"dict index proven by guard".to_string());
        return value;
    }
    return -(1 as i64);
}

fn sum_known_keys(table: &HashMap<i64, i64>, keys: &Vec<i64>) -> i64 {
    let mut total: i64 = 0 as i64;
    for key in keys.iter().copied() {
        if table.contains_key(&key) {
            total = total
                + table
                    .get(&key)
                    .copied()
                    .expect(&"dict index proven by guard".to_string());
        }
    }
    return total;
}

fn main() {
    let t: HashMap<i64, i64> = HashMap::from([
        (1 as i64, 10 as i64),
        (2 as i64, 20 as i64),
        (4 as i64, 40 as i64),
    ]);
    assert!(guarded_lookup(&t, 2 as i64) == (20 as i64));
    assert!(guarded_lookup(&t, 3 as i64) == -(1 as i64));
    assert!(expression_lookup(&t, 1 as i64) == (20 as i64));
    assert!(expression_lookup(&t, 2 as i64) == -(1 as i64));
    assert!(sum_known_keys(&t, &vec![0 as i64, 1 as i64, 2 as i64, 5 as i64]) == (30 as i64));
}
