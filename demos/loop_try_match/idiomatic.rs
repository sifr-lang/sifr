fn normalize(n: i64) -> i64 {
    match n {
        value if value > 0 => value,
        _ => 0,
    }
}

fn compute(values: &[i64]) -> i64 {
    let mut total = 0;
    for value in values.iter().copied() {
        total += normalize(value);
    }
    total + 1
}

fn main() {
    println!("m24_1 canonical traversal layer contract demo:");
    println!("{}", compute(&[3, 2, -1]));
}
