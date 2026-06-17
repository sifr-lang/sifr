fn safe_add_one(x: Option<i64>) -> i64 {
    let Some(x) = x else {
        return 0 as i64;
    };
    return x + (1 as i64);
}

fn safe_divide(total: Option<i64>, count: i64) -> f64 {
    let Some(total) = total else {
        return 0.0 as f64;
    };
    return (total as f64) / (count as f64);
}

fn main() {
    println!("optional_arithmetic optional arithmetic soundness demo:");
    println!("{}", safe_add_one(Some(5 as i64)));
    println!("{}", safe_add_one(None));
    println!("{}", safe_divide(Some(9 as i64), 3 as i64));
    println!("{}", safe_divide(None, 3 as i64));
}
