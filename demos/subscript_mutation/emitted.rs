// src/main.rs
use ::std::collections::HashMap;

fn main() {
    let mut nums: Vec<i64> = vec![1_i64, 2_i64, 3_i64];
    {
        let __idx_raw = 0_i64;
        let __idx_norm = if __idx_raw < 0 { (nums.len() as i64) + __idx_raw } else { __idx_raw };
        if __idx_norm >= 0 {
            if let Some(__elem) = nums.get_mut(__idx_norm as usize) {
                *__elem = 10_i64;
            }
        }
    }
    {
        let __idx_raw = 2_i64;
        let __idx_norm = if __idx_raw < 0 { (nums.len() as i64) + __idx_raw } else { __idx_raw };
        if __idx_norm >= 0 {
            if let Some(__elem) = nums.get_mut(__idx_norm as usize) {
                *__elem = 30_i64;
            }
        }
    }
    println!("{:?}", nums);
    assert!((format!("{:?}", nums) == "[10, 2, 30]"));
    let mut d: HashMap<String, i64> = HashMap::from([("a".to_string(), 1_i64)]);
    d.insert("b".to_string(), 2_i64);
    let val: Option<i64> = Some({
    let Some(__sifr_proven_dict_value) = d.get("b").copied() else {
        ::std::process::abort();
    };
    __sifr_proven_dict_value
});
    if let Some(val) = val {
        println!("{}", val);
        assert!((format!("{}", val) == "2"));
    }
}
