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
use ::sifr_runtime::SifrInt;
fn main() {
    let mut items: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)
    ];
    let __sifr_try_res: Result<(), IndexError> = (|| {
        {
            let __assign_value = SifrInt::from_i64(9);
            {
                let __index_raw = -(SifrInt::from_i64(1));
                let __index_normalized = __index_raw.normalize_index_or_len(items.len());
                if let Some(__elem) = items.get_mut(__index_normalized) {
                    *__elem = __assign_value;
                } else {
                    return Err(
                        IndexError::new("collection index out of range".to_string()),
                    );
                }
            }
        }
        {
            let __assign_value = SifrInt::from_i64(5);
            {
                let __index_raw = -(SifrInt::from_i64(2));
                let __index_normalized = __index_raw.normalize_index_or_len(items.len());
                if let Some(__elem) = items.get_mut(__index_normalized) {
                    *__elem += __assign_value;
                } else {
                    return Err(
                        IndexError::new("collection index out of range".to_string()),
                    );
                }
            }
        }
        {
            let __delete_target = &mut items;
            let __idx_raw = -(SifrInt::from_i64(1));
            let __idx_norm = __idx_raw.normalize_index_or_len(__delete_target.len());
            if __idx_norm < __delete_target.len() {
                let _ = __delete_target.remove(__idx_norm);
            } else {
                return Err(IndexError::new("collection index out of range".to_string()));
            }
        }
        {
            let __delete_target = &mut items;
            let __idx_raw = -(SifrInt::from_i64(5));
            let __idx_norm = __idx_raw.normalize_index_or_len(__delete_target.len());
            if __idx_norm < __delete_target.len() {
                let _ = __delete_target.remove(__idx_norm);
            } else {
                return Err(IndexError::new("collection index out of range".to_string()));
            }
        }
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let _e = __sifr_try_err.clone();
    }
    println!("indexing_rules indexing and semantics parity fixes demo:");
    println!("{:?}", items);
}
