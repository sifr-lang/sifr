// src/main.rs
fn compute(limit: i64) -> i64 {
    let mut total: i64 = 0_i64;
    for n in 0_i64..limit {
        if n == (2_i64) {
            continue;
        }
        if n == (4_i64) {
            break;
        }
        total += n;
    }
    total
}

fn main() {
    println!("valid_control_flow cfg validity invariants demo:");
    println!("{}", compute(8_i64));
}
