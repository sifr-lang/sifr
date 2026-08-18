// src/main.rs
fn main() {
    let mut total: i64 = 0_i64;
    for i in 1_i64..6_i64 {
        if (i % (2_i64)) == (0_i64) {
            total += i;
        } else {
            total += i * (2_i64);
        }
    }
    let verdict: String = if total > (10_i64) { "high".to_string() } else { "low".to_string() };
    println!("total = {}", total);
    assert!((format!("{}", format!("total = {}", total)) == "total = 24"));
    println!("verdict = {}", verdict);
    assert!((format!("{}", format!("verdict = {}", verdict)) == "verdict = high"));
}
