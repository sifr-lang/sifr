// src/main.rs
fn main() {
    println!("max(3, 7) = {}", ::std::cmp::max(3_i64, 7_i64));
    assert!((format!("{}", format!("max(3, 7) = {}", ::std::cmp::max(3_i64, 7_i64))) == "max(3, 7) = 7"));
    println!("min(3, 7) = {}", ::std::cmp::min(3_i64, 7_i64));
    assert!((format!("{}", format!("min(3, 7) = {}", ::std::cmp::min(3_i64, 7_i64))) == "min(3, 7) = 3"));
    println!("pow(2, 10) = {}", (2_i64).pow((10_i64) as u32));
    assert!((format!("{}", format!("pow(2, 10) = {}", (2_i64).pow((10_i64) as u32))) == "pow(2, 10) = 1024"));
    let mut result: String = "".to_string();
    for i in (0_i64..10_i64).step_by((2_i64) as usize) {
        if ((result.chars().count() as i64) > (0_i64)) {
            result.push(' ');
        }
        result.push_str((format!("{}", i)).as_str());
    }
    println!("{}", result);
    assert!((format!("{}", result) == "0 2 4 6 8"));
}
