fn compute(limit: i64) -> i64 {
    let mut total = 0;
    for n in 0..limit {
        if n == 2 {
            continue;
        }
        if n == 4 {
            break;
        }
        total += n;
    }
    total
}

fn main() {
    println!("valid_control_flow cfg validity invariants demo:");
    println!("{}", compute(8));
}
