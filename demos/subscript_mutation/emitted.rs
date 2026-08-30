// src/main.rs
use ::std::collections::HashMap;

use ::sifr_runtime::SifrInt;

fn main() {
    let mut nums: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)];
    {
        let __idx_raw = SifrInt::from_i64(0);
        let __idx_norm = __idx_raw.normalize_index_or_len(nums.len());
        if let Some(__elem) = nums.get_mut(__idx_norm) {
            *__elem = SifrInt::from_i64(10);
        }
    }
    {
        let __idx_raw = SifrInt::from_i64(2);
        let __idx_norm = __idx_raw.normalize_index_or_len(nums.len());
        if let Some(__elem) = nums.get_mut(__idx_norm) {
            *__elem = SifrInt::from_i64(30);
        }
    }
    println!("{:?}", nums);
    assert!((format!("{:?}", nums) == "[10, 2, 30]"));
    let mut d: HashMap<String, SifrInt> = HashMap::from([("a".to_string(), SifrInt::from_i64(1))]);
    d.insert("b".to_string(), SifrInt::from_i64(2));
    let val: Option<SifrInt> = Some(d["b"].clone());
    if let Some(val) = val.clone() {
        println!("{}", val);
        assert!((format!("{}", val) == "2"));
    }
}
