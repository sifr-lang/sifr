// src/main.rs
fn sort_values(mut values: Vec<i64>) -> Vec<i64> {
    values.sort();
    values
}

fn main() {
    assert!((format!("{:?}", sort_values(vec![5_i64, 1_i64, 4_i64, 2_i64])) == "[1, 2, 4, 5]"));
    assert!((format!("{:?}", sort_values(vec![3_i64])) == "[3]"));
    println!("mut_sort: ok");
}
