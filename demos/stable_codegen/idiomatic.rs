fn summarize(values: &[i64]) -> i64 {
    let mut total = 0;

    for value in values.iter().copied() {
        if value > 10 {
            total += value;
        } else {
            total += 1;
        }
    }

    total
}

fn main() {
    println!("stable_codegen analysis/emission boundary hardening demo:");
    println!("{}", summarize(&[3, 12, 20]));
}
