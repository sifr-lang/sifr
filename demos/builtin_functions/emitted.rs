fn main() {
    println!("max(3, 7) = {}", std::cmp::max(3 as i64, 7 as i64));
    assert!(format!("{}", format!("max(3, 7) = {}", std::cmp::max(3 as i64, 7 as i64))) == "max(3, 7) = 7".to_string());
    println!("min(3, 7) = {}", std::cmp::min(3 as i64, 7 as i64));
    assert!(format!("{}", format!("min(3, 7) = {}", std::cmp::min(3 as i64, 7 as i64))) == "min(3, 7) = 3".to_string());
    println!("pow(2, 10) = {}", (2 as i64).pow((10 as i64) as u32));
    assert!(format!("{}", format!("pow(2, 10) = {}", (2 as i64).pow((10 as i64) as u32))) == "pow(2, 10) = 1024".to_string());
    let mut result: String = "".to_string();
    for i in (0 as i64..10 as i64).step_by((2 as i64) as usize) {
        if (result.len() as i64) > (0 as i64) {
            result = format!("{}{}", result, " ".to_string());
        }
        result = format!("{}{}", result, format!("{}", i));
    }
    println!("{}", result);
    assert!(format!("{}", result) == "0 2 4 6 8".to_string());
}
