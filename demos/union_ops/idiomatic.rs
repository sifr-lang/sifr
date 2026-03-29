fn increment(x: Option<i64>) -> i64 {
    x.map_or(0, |x| x + 1)
}

fn double(x: Option<f64>) -> f64 {
    x.map_or(0.0, |x| x * 2.0)
}

fn safe_len(items: Option<&[String]>) -> usize {
    items.map_or(0, |items| items.len())
}

fn merge_lists(a: Vec<i64>, b: Vec<i64>) -> Vec<i64> {
    a.into_iter().chain(b).collect()
}

fn main() {
    let v = Some(10);
    println!("{}", increment(v));
    let f = Some(3.14);
    println!("{}", double(f));
    let names = vec![
        "alice".to_string(),
        "bob".to_string(),
        "charlie".to_string(),
    ];
    println!("{}", safe_len(Some(&names)));
    let merged = merge_lists(vec![1, 2, 3], vec![4, 5, 6]);
    println!("{}", merged.len());
}
