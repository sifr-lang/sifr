// src/main.rs
mod __sifr_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IndexError {
        pub message: String,
    }
    impl IndexError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for IndexError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IndexError {}
}
pub use __sifr_project_nominals::IndexError;
use ::std::collections::HashMap;
use ::sifr_runtime::SifrInt;
fn main() {
    let mut nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)
    ];
    let __sifr_try_res: Result<(), IndexError> = (|| {
        {
            let __assign_value = SifrInt::from_i64(10);
            {
                let __index_raw = SifrInt::from_i64(0);
                let __index_normalized = __index_raw.normalize_index_or_len(nums.len());
                if let Some(__elem) = nums.get_mut(__index_normalized) {
                    *__elem = __assign_value;
                } else {
                    return Err(
                        IndexError::new("collection index out of range".to_string()),
                    );
                }
            }
        }
        {
            let __assign_value = SifrInt::from_i64(30);
            {
                let __index_raw = SifrInt::from_i64(2);
                let __index_normalized = __index_raw.normalize_index_or_len(nums.len());
                if let Some(__elem) = nums.get_mut(__index_normalized) {
                    *__elem = __assign_value;
                } else {
                    return Err(
                        IndexError::new("collection index out of range".to_string()),
                    );
                }
            }
        }
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let _e = __sifr_try_err.clone();
        return;
    }
    println!("{:?}", nums);
    assert!((format!("{:?}", nums) == "[10, 2, 30]"));
    let mut d: HashMap<String, SifrInt> = HashMap::from([
        ("a".to_string(), SifrInt::from_i64(1)),
    ]);
    {
        let __assign_value = SifrInt::from_i64(2);
        {
            let __assign_key = "b".to_string();
            d.insert(__assign_key, __assign_value);
        }
    }
    let val: Option<SifrInt> = d.get("b").cloned();
    if let Some(val) = val.clone() {
        println!("{}", val);
        assert!((format!("{}", val) == "2"));
    }
}
