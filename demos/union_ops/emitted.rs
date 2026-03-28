fn increment(x: Option<i64>) -> i64 {
    let Some(x) = x else {
        return 0 as i64;
    };
    return x + (1 as i64);
}

fn double(x: Option<f64>) -> f64 {
    let Some(x) = x else {
        return 0.0 as f64;
    };
    return x * (2.0 as f64);
}

fn safe_len(items: &Option<Vec<String>>) -> i64 {
    return items.as_ref().map_or(0 as usize, |v| v.len()) as i64;
}

fn merge_lists(a: Vec<i64>, b: Vec<i64>) -> Vec<i64> {
    return {
    let mut __v = (a).clone();
    __v.extend((b).iter().cloned());
    __v
};
}

fn main() {
    let v: Option<i64> = Some(10 as i64);
    println!("{}", increment(v));
    let f: Option<f64> = Some(3.14 as f64);
    println!("{}", double(f));
    let names: Option<Vec<String>> = Some(vec!["alice".to_string(), "bob".to_string(), "charlie".to_string()]);
    println!("{}", safe_len(&names));
    let merged: Vec<i64> = merge_lists(vec![1 as i64, 2 as i64, 3 as i64], vec![4 as i64, 5 as i64, 6 as i64]);
    println!("{}", merged.len() as i64);
}
