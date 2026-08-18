// src/main.rs
fn increment(x: Option<i64>) -> i64 {
    let Some(x) = x else {
        return 0_i64;
    };
    x + (1_i64)
}

fn double(x: Option<f64>) -> f64 {
    let Some(x) = x else {
        return 0.0_f64;
    };
    x * (2.0_f64)
}

fn safe_len(items: &Option<Vec<String>>) -> i64 {
    items.as_ref().map_or(0_usize, |v| v.len()) as i64
}

fn merge_lists(a: Vec<i64>, b: Vec<i64>) -> Vec<i64> {
    {
    let mut __v = (a).clone();
    __v.extend((b).iter().cloned());
    __v
}
}

fn main() {
    let v: Option<i64> = Some(10_i64);
    println!("{}", increment(v));
    let f: Option<f64> = Some(3.14_f64);
    println!("{}", double(f));
    let names: Option<Vec<String>> = Some(vec!["alice".to_string(), "bob".to_string(), "charlie".to_string()]);
    println!("{}", safe_len(&names));
    let merged: Vec<i64> = merge_lists(vec![1_i64, 2_i64, 3_i64], vec![4_i64, 5_i64, 6_i64]);
    println!("{}", merged.len() as i64);
}
