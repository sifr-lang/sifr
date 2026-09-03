// src/main.rs
use ::sifr_runtime::SifrInt;
fn main() {
    let pair: (SifrInt, SifrInt) = (SifrInt::from_i64(10), SifrInt::from_i64(20));
    let a: SifrInt = pair.0.clone();
    let b: SifrInt = pair.1.clone();
    println!("Tuple index: {a}, {b}");
    assert_eq!(format!("Tuple index: {a}, {b}"), "Tuple index: 10, 20");
    let _x: SifrInt = SifrInt::from_i64(10);
    let _y: SifrInt = SifrInt::from_i64(2);
    let result: f64 = 10.0 / 2.0;
    println!("Division 10/2: {result}");
    assert_eq!(format!("Division 10/2: {result}"), "Division 10/2: 5");
    let val: Option<SifrInt> = None;
    if val.is_none() {
        println!("None value: None");
    } else if let Some(val) = val.clone() {
        println!("None value: {val}");
    }
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
    ];
    let empty: Vec<SifrInt> = Vec::new();
    println!("bool([1,2,3]): {}", !nums.is_empty());
    assert_eq!(
        format!("bool([1,2,3]): {}", !nums.is_empty()),
        "bool([1,2,3]): true"
    );
    println!("bool([]): {}", !empty.is_empty());
    assert_eq!(
        format!("bool([]): {}", !empty.is_empty()),
        "bool([]): false"
    );
    let mut base: SifrInt = SifrInt::from_i64(2);
    base = base.pow_known_valid(0_u32);
    println!("2**0 = {base}");
    assert_eq!(format!("2**0 = {base}"), "2**0 = 1");
    let _i: SifrInt = SifrInt::from_i64(10);
    let f: f64 = 3.5_f64;
    let mixed: f64 = 10.0 + f;
    println!("10 + 3.5 = {mixed}");
    assert_eq!(format!("10 + 3.5 = {mixed}"), "10 + 3.5 = 13.5");
    let msg: String = "She said \"hello\"".to_string();
    println!("{msg}");
    assert_eq!(msg.to_string(), "She said \"hello\"");
}
