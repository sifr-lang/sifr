fn main() {
    let mut total = 0;

    for i in 1_i64..6 {
        if i % 2 == 0 {
            total += i;
        } else {
            total += i * 2;
        }
    }

    let verdict = if total > 10 { "high" } else { "low" };
    let total_line = format!("total = {total}");
    let verdict_line = format!("verdict = {verdict}");

    println!("{total_line}");
    assert_eq!(total_line, "total = 24");
    println!("{verdict_line}");
    assert_eq!(verdict_line, "verdict = high");
}
