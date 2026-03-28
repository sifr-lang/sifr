fn main() {
    let mut total: i64 = 0 as i64;
    for i in 1 as i64..6 as i64 {
        if (i % (2 as i64)) == (0 as i64) {
            total += i;
        } else {
            total += i * (2 as i64);
        }
    }
    let verdict: String = if total > (10 as i64) {
        "high".to_string()
    } else {
        "low".to_string()
    };
    println!("total = {}", total);
    assert!(format!("{}", format!("total = {}", total)) == "total = 24".to_string());
    println!("verdict = {}", verdict);
    assert!(format!("{}", format!("verdict = {}", verdict)) == "verdict = high".to_string());
}
