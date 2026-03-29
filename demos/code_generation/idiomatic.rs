fn main() {
    let pair = (10, 20);
    println!("Tuple index: {}, {}", pair.0, pair.1);
    assert_eq!(
        format!("Tuple index: {}, {}", pair.0, pair.1),
        "Tuple index: 10, 20"
    );

    let result = 10.0 / 3.0;
    println!("Division 10/3: {result}");
    assert_eq!(
        format!("Division 10/3: {result}"),
        "Division 10/3: 3.3333333333333335"
    );

    let val: Option<i64> = None;
    if val.is_none() {
        println!("None value: None");
    } else if let Some(value) = val {
        println!("None value: {value}");
    }

    let nums = [1, 2, 3];
    let empty: [i64; 0] = [];
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

    let mut base = 2_i64;
    base = base.pow(3);
    println!("2**3 = {base}");
    assert_eq!(format!("2**3 = {base}"), "2**3 = 8");

    let mixed = 10.0 + 3.5;
    println!("10 + 3.5 = {mixed}");
    assert_eq!(format!("10 + 3.5 = {mixed}"), "10 + 3.5 = 13.5");

    let msg = "She said \"hello\"";
    println!("{msg}");
    assert_eq!(msg, "She said \"hello\"");
}
