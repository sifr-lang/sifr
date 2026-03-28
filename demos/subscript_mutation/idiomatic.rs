use std::collections::HashMap;

fn main() {
    let mut nums: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64];
    {
        let __idx_raw = 0 as i64;
        let __idx_norm = if __idx_raw < 0 {
            (nums.len() as i64) + __idx_raw
        } else {
            __idx_raw
        };
        if __idx_norm >= 0 {
            if let Some(__elem) = nums.get_mut(__idx_norm as usize) {
                *__elem = 10 as i64;
            }
        }
    }
    {
        let __idx_raw = 2 as i64;
        let __idx_norm = if __idx_raw < 0 {
            (nums.len() as i64) + __idx_raw
        } else {
            __idx_raw
        };
        if __idx_norm >= 0 {
            if let Some(__elem) = nums.get_mut(__idx_norm as usize) {
                *__elem = 30 as i64;
            }
        }
    }
    println!("{:?}", nums);
    assert!(format!("{:?}", nums) == "[10, 2, 30]".to_string());
    let mut d: HashMap<String, i64> = HashMap::from([("a".to_string(), 1 as i64)]);
    d.insert("b".to_string(), 2 as i64);
    let val: Option<i64> = d.get("b").copied();
    if let Some(val) = val {
        println!("{}", val);
        assert!(format!("{}", val) == "2".to_string());
    }
}
