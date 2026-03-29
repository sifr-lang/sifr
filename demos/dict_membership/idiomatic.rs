use std::collections::HashMap;

fn guarded_lookup(table: &HashMap<i64, i64>, key: i64) -> i64 {
    table.get(&key).copied().unwrap_or(-1)
}

fn expression_lookup(table: &HashMap<i64, i64>, base: i64) -> i64 {
    table.get(&(base + 1)).copied().unwrap_or(-1)
}

fn sum_known_keys(table: &HashMap<i64, i64>, keys: &[i64]) -> i64 {
    keys.iter().filter_map(|key| table.get(key)).copied().sum()
}

fn main() {
    let table = HashMap::from([(1_i64, 10_i64), (2, 20), (4, 40)]);

    assert_eq!(guarded_lookup(&table, 2), 20);
    assert_eq!(guarded_lookup(&table, 3), -1);
    assert_eq!(expression_lookup(&table, 1), 20);
    assert_eq!(expression_lookup(&table, 2), -1);
    assert_eq!(sum_known_keys(&table, &[0, 1, 2, 5]), 30);
}
