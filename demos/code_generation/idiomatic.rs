fn main() {
    let pair: (i64, i64) = (10 as i64, 20 as i64);
    let a: i64 = (pair).0;
    let b: i64 = (pair).1;
    println!("Tuple index: {}, {}", a, b);
    assert!(
        format!("{}", format!("Tuple index: {}, {}", a, b)) == "Tuple index: 10, 20".to_string()
    );
    let x: i64 = 10 as i64;
    let y: i64 = 3 as i64;
    let result: f64 = (x as f64) / (y as f64);
    println!("Division 10/3: {}", result);
    assert!(
        format!("{}", format!("Division 10/3: {}", result))
            == "Division 10/3: 3.3333333333333335".to_string()
    );
    let val: Option<i64> = None;
    if val.is_none() {
        println!("None value: None");
    } else {
        if let Some(val) = val {
            println!("None value: {}", val);
        }
    }
    let nums: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64];
    let empty: Vec<i64> = vec![];
    println!("bool([1,2,3]): {}", !nums.is_empty());
    assert!(
        format!("{}", format!("bool([1,2,3]): {}", !nums.is_empty()))
            == "bool([1,2,3]): true".to_string()
    );
    println!("bool([]): {}", !empty.is_empty());
    assert!(
        format!("{}", format!("bool([]): {}", !empty.is_empty())) == "bool([]): false".to_string()
    );
    let mut base: i64 = 2 as i64;
    base = (base).pow((3 as i64) as u32);
    println!("2**3 = {}", base);
    assert!(format!("{}", format!("2**3 = {}", base)) == "2**3 = 8".to_string());
    let i: i64 = 10 as i64;
    let f: f64 = 3.5 as f64;
    let mixed: f64 = (i as f64) + f;
    println!("10 + 3.5 = {}", mixed);
    assert!(format!("{}", format!("10 + 3.5 = {}", mixed)) == "10 + 3.5 = 13.5".to_string());
    let msg: String = "She said \"hello\"".to_string();
    println!("{}", msg);
    assert!(format!("{}", msg) == "She said \"hello\"".to_string());
}
