// src/main.rs
mod sifr_generated_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IndexError {
        pub message: String,
    }
    impl IndexError {
        #[must_use]
        pub const fn new(message: String) -> Self {
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
use ::sifr_runtime::SifrInt;
use ::std::collections::HashMap;
pub use sifr_generated_project_nominals::IndexError;
fn main() {
    let mut nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
    ];
    let sifr_generated_try_res: Result<(), IndexError> = (|| {
        {
            let sifr_generated_assign_value = SifrInt::from_i64(10);
            {
                let sifr_generated_index_raw = SifrInt::from_i64(0);
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(nums.len());
                if let Some(sifr_generated_elem) = nums.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                } else {
                    return Err(IndexError::new("collection index out of range".to_string()));
                }
            }
        }
        {
            let sifr_generated_assign_value = SifrInt::from_i64(30);
            {
                let sifr_generated_index_raw = SifrInt::from_i64(2);
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(nums.len());
                if let Some(sifr_generated_elem) = nums.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                } else {
                    return Err(IndexError::new("collection index out of range".to_string()));
                }
            }
        }
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _ = sifr_generated_try_err;
        return;
    }
    println!("{nums:?}");
    assert_eq!(format!("{nums:?}"), "[10, 2, 30]");
    let mut d: HashMap<String, SifrInt> = HashMap::from([("a".to_string(), SifrInt::from_i64(1))]);
    {
        let sifr_generated_assign_value = SifrInt::from_i64(2);
        {
            let sifr_generated_assign_key = "b".to_string();
            d.insert(sifr_generated_assign_key, sifr_generated_assign_value);
        }
    }
    let val: Option<SifrInt> = d.get("b").cloned();
    if let Some(val) = val {
        println!("{val}");
        assert_eq!(val.to_string(), "2");
    }
}
