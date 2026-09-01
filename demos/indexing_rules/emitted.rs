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
pub use sifr_generated_project_nominals::IndexError;
fn main() {
    let mut items: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
    ];
    let sifr_generated_try_res: Result<(), IndexError> = (|| {
        {
            let sifr_generated_assign_value = SifrInt::from_i64(9);
            {
                let sifr_generated_index_raw = -SifrInt::from_i64(1);
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(items.len());
                if let Some(sifr_generated_elem) = items.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                } else {
                    return Err(IndexError::new("collection index out of range".to_string()));
                }
            }
        }
        {
            let sifr_generated_assign_value = SifrInt::from_i64(5);
            {
                let sifr_generated_index_raw = -SifrInt::from_i64(2);
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(items.len());
                if let Some(sifr_generated_elem) = items.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem += sifr_generated_assign_value;
                } else {
                    return Err(IndexError::new("collection index out of range".to_string()));
                }
            }
        }
        {
            let sifr_generated_delete_target = &mut items;
            let sifr_generated_idx_raw = -SifrInt::from_i64(1);
            let sifr_generated_idx_norm =
                sifr_generated_idx_raw.normalize_index_or_len(sifr_generated_delete_target.len());
            if sifr_generated_idx_norm < sifr_generated_delete_target.len() {
                let _ = sifr_generated_delete_target.remove(sifr_generated_idx_norm);
            } else {
                return Err(IndexError::new("collection index out of range".to_string()));
            }
        }
        {
            let sifr_generated_delete_target = &mut items;
            let sifr_generated_idx_raw = -SifrInt::from_i64(5);
            let sifr_generated_idx_norm =
                sifr_generated_idx_raw.normalize_index_or_len(sifr_generated_delete_target.len());
            if sifr_generated_idx_norm < sifr_generated_delete_target.len() {
                let _ = sifr_generated_delete_target.remove(sifr_generated_idx_norm);
            } else {
                return Err(IndexError::new("collection index out of range".to_string()));
            }
        }
        Ok(())
    })();
    let _ = sifr_generated_try_res.is_err();
    println!("indexing_rules indexing and semantics parity fixes demo:");
    println!("{items:?}");
}
