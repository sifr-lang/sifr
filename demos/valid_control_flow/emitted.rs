fn compute(limit: i64) -> i64 {
    let mut total: i64 = 0 as i64;
    for n in 0 as i64..limit {
        if n == (2 as i64) {
            continue;
        }
        if n == (4 as i64) {
            break;
        }
        total += n;
    }
    return total;
}

fn main() {
    println!("valid_control_flow cfg validity invariants demo:");
    println!("{}", compute(8 as i64));
}
