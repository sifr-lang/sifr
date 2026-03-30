fn sort_values(mut values: Vec<i64>) -> Vec<i64> {
    values.sort();
    values
}

fn main() {
    assert_eq!(sort_values(vec![5, 1, 4, 2]), vec![1, 2, 4, 5]);
    assert_eq!(sort_values(vec![3]), vec![3]);
    println!("mut_sort: ok");
}
