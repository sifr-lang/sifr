use std::collections::HashMap;

fn main() {
    let mut nums = vec![1_i64, 2, 3];
    nums[0] = 10;
    nums[2] = 30;
    println!("{nums:?}");
    assert_eq!(format!("{nums:?}"), "[10, 2, 30]");

    let mut dict = HashMap::from([("a".to_string(), 1_i64)]);
    dict.insert("b".to_string(), 2);

    if let Some(value) = dict.get("b").copied() {
        println!("{value}");
        assert_eq!(value.to_string(), "2");
    }
}
