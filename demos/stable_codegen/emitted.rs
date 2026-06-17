fn summarize(values: &Vec<i64>) -> i64 {
    let mut total: i64 = 0 as i64;
    for value in values.iter().copied() {
        if value > (10 as i64) {
            total = total + value;
        } else {
            total = total + (1 as i64);
        }
    }
    return total;
}

fn main() {
    println!("stable_codegen analysis/emission boundary hardening demo:");
    println!("{}", summarize(&vec![3 as i64, 12 as i64, 20 as i64]));
}
