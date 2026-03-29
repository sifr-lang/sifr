fn increment_all(mut values: Vec<i64>) -> Vec<i64> {
    for value in &mut values {
        *value += 1;
    }
    values
}

fn clear_all(mut values: Vec<i64>) -> Vec<i64> {
    for value in &mut values {
        *value = 0;
    }
    values
}

fn main() {
    assert_eq!(format!("{:?}", increment_all(vec![1, 2, 3])), "[2, 3, 4]");
    assert_eq!(format!("{:?}", clear_all(vec![4, 5, 6])), "[0, 0, 0]");
    println!("own_mut_updates: ok");
}
