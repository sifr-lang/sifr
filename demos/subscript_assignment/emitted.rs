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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Counter {
    count: SifrInt,
}
impl Counter {
    fn new(count: SifrInt) -> Self {
        Self { count }
    }
}
impl Counter {
    fn increment(&mut self) {
        self.count += SifrInt::from_i64(1);
    }
}
impl ::std::fmt::Display for Counter {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Counter(count={})", self.count)
    }
}
fn main() {
    let mut matrix: Vec<Vec<SifrInt>> = vec![
        vec![SifrInt::from_i64(0), SifrInt::from_i64(0), SifrInt::from_i64(0)],
        vec![SifrInt::from_i64(0), SifrInt::from_i64(0), SifrInt::from_i64(0)],
        vec![SifrInt::from_i64(0), SifrInt::from_i64(0), SifrInt::from_i64(0)]
    ];
    let __sifr_try_res: Result<(), IndexError> = (|| {
        {
            let __nested_assign_value = SifrInt::from_i64(1);
            {
                let __outer_raw = SifrInt::from_i64(0);
                let __outer_normalized = __outer_raw
                    .normalize_index_or_len(matrix.len());
                if let Some(__row) = matrix.get_mut(__outer_normalized) {
                    {
                        let __inner_raw = SifrInt::from_i64(0);
                        let __inner_normalized = __inner_raw
                            .normalize_index_or_len(__row.len());
                        if let Some(__elem) = __row.get_mut(__inner_normalized) {
                            *__elem = __nested_assign_value;
                        } else {
                            return Err(
                                IndexError::new("collection index out of range".to_string()),
                            );
                        }
                    }
                } else {
                    return Err(
                        IndexError::new("collection index out of range".to_string()),
                    );
                }
            }
        }
        {
            let __nested_assign_value = SifrInt::from_i64(1);
            {
                let __outer_raw = SifrInt::from_i64(1);
                let __outer_normalized = __outer_raw
                    .normalize_index_or_len(matrix.len());
                if let Some(__row) = matrix.get_mut(__outer_normalized) {
                    {
                        let __inner_raw = SifrInt::from_i64(1);
                        let __inner_normalized = __inner_raw
                            .normalize_index_or_len(__row.len());
                        if let Some(__elem) = __row.get_mut(__inner_normalized) {
                            *__elem = __nested_assign_value;
                        } else {
                            return Err(
                                IndexError::new("collection index out of range".to_string()),
                            );
                        }
                    }
                } else {
                    return Err(
                        IndexError::new("collection index out of range".to_string()),
                    );
                }
            }
        }
        {
            let __nested_assign_value = SifrInt::from_i64(1);
            {
                let __outer_raw = SifrInt::from_i64(2);
                let __outer_normalized = __outer_raw
                    .normalize_index_or_len(matrix.len());
                if let Some(__row) = matrix.get_mut(__outer_normalized) {
                    {
                        let __inner_raw = SifrInt::from_i64(2);
                        let __inner_normalized = __inner_raw
                            .normalize_index_or_len(__row.len());
                        if let Some(__elem) = __row.get_mut(__inner_normalized) {
                            *__elem = __nested_assign_value;
                        } else {
                            return Err(
                                IndexError::new("collection index out of range".to_string()),
                            );
                        }
                    }
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
    println!(
        "{}", ({ let __sifr_outer_list = & matrix; let __sifr_outer_i =
        SifrInt::from_i64(0); let __sifr_outer_norm = __sifr_outer_i
        .normalize_index_or_len(__sifr_outer_list.len()); __sifr_outer_list
        .get(::sifr_runtime::to_usize_proven(& (__sifr_outer_norm))).and_then(|
        __sifr_row | { let __sifr_inner_i = SifrInt::from_i64(0); let __sifr_inner_norm =
        __sifr_inner_i.normalize_index_or_len(__sifr_row.len()); __sifr_row
        .get(::sifr_runtime::to_usize_proven(& (__sifr_inner_norm))).cloned() }) })
        .map_or("None".to_string().to_string(), | __v | format!("{}", __v))
    );
    println!(
        "{}", ({ let __sifr_outer_list = & matrix; let __sifr_outer_i =
        SifrInt::from_i64(1); let __sifr_outer_norm = __sifr_outer_i
        .normalize_index_or_len(__sifr_outer_list.len()); __sifr_outer_list
        .get(::sifr_runtime::to_usize_proven(& (__sifr_outer_norm))).and_then(|
        __sifr_row | { let __sifr_inner_i = SifrInt::from_i64(1); let __sifr_inner_norm =
        __sifr_inner_i.normalize_index_or_len(__sifr_row.len()); __sifr_row
        .get(::sifr_runtime::to_usize_proven(& (__sifr_inner_norm))).cloned() }) })
        .map_or("None".to_string().to_string(), | __v | format!("{}", __v))
    );
    println!(
        "{}", ({ let __sifr_outer_list = & matrix; let __sifr_outer_i =
        SifrInt::from_i64(2); let __sifr_outer_norm = __sifr_outer_i
        .normalize_index_or_len(__sifr_outer_list.len()); __sifr_outer_list
        .get(::sifr_runtime::to_usize_proven(& (__sifr_outer_norm))).and_then(|
        __sifr_row | { let __sifr_inner_i = SifrInt::from_i64(2); let __sifr_inner_norm =
        __sifr_inner_i.normalize_index_or_len(__sifr_row.len()); __sifr_row
        .get(::sifr_runtime::to_usize_proven(& (__sifr_inner_norm))).cloned() }) })
        .map_or("None".to_string().to_string(), | __v | format!("{}", __v))
    );
    let mut scores: Vec<SifrInt> = vec![
        SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30)
    ];
    let __sifr_try_res: Result<(), IndexError> = (|| {
        {
            let __assign_value = SifrInt::from_i64(5);
            {
                let __index_raw = SifrInt::from_i64(0);
                let __index_normalized = __index_raw
                    .normalize_index_or_len(scores.len());
                if let Some(__elem) = scores.get_mut(__index_normalized) {
                    *__elem += __assign_value;
                } else {
                    return Err(
                        IndexError::new("collection index out of range".to_string()),
                    );
                }
            }
        }
        {
            let __assign_value = SifrInt::from_i64(3);
            {
                let __index_raw = SifrInt::from_i64(1);
                let __index_normalized = __index_raw
                    .normalize_index_or_len(scores.len());
                if let Some(__elem) = scores.get_mut(__index_normalized) {
                    *__elem -= __assign_value;
                } else {
                    return Err(
                        IndexError::new("collection index out of range".to_string()),
                    );
                }
            }
        }
        {
            let __assign_value = SifrInt::from_i64(2);
            {
                let __index_raw = SifrInt::from_i64(2);
                let __index_normalized = __index_raw
                    .normalize_index_or_len(scores.len());
                if let Some(__elem) = scores.get_mut(__index_normalized) {
                    *__elem *= __assign_value;
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
    let s0: Option<SifrInt> = {
        let __sifr_checked_read_collection = &scores;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let s1: Option<SifrInt> = {
        let __sifr_checked_read_collection = &scores;
        let __sifr_checked_read_index = SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let s2: Option<SifrInt> = {
        let __sifr_checked_read_collection = &scores;
        let __sifr_checked_read_index = SifrInt::from_i64(2);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    if let Some(s0) = s0.clone() {
        println!("{}", s0);
    }
    if let Some(s1) = s1.clone() {
        println!("{}", s1);
    }
    if let Some(s2) = s2.clone() {
        println!("{}", s2);
    }
    let mut c = Counter::new(SifrInt::from_i64(0));
    c.increment();
    c.increment();
    c.increment();
    println!("{}", c.count.clone());
}
