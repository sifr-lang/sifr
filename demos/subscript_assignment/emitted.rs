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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Counter {
    count: SifrInt,
}
impl Counter {
    const fn new(count: SifrInt) -> Self {
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
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    let mut matrix: Vec<Vec<SifrInt>> = vec![
        vec![
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
        ],
        vec![
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
        ],
        vec![
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
        ],
    ];
    let sifr_generated_try_res: Result<(), IndexError> = (|| {
        {
            let sifr_generated_nested_assign_value = SifrInt::from_i64(1);
            {
                let sifr_generated_outer_raw = SifrInt::from_i64(0);
                let sifr_generated_outer_normalized =
                    sifr_generated_outer_raw.normalize_index_or_len(matrix.len());
                if let Some(sifr_generated_row) = matrix.get_mut(sifr_generated_outer_normalized) {
                    {
                        let sifr_generated_inner_raw = SifrInt::from_i64(0);
                        let sifr_generated_inner_normalized = sifr_generated_inner_raw
                            .normalize_index_or_len(sifr_generated_row.len());
                        if let Some(sifr_generated_elem) =
                            sifr_generated_row.get_mut(sifr_generated_inner_normalized)
                        {
                            *sifr_generated_elem = sifr_generated_nested_assign_value;
                        } else {
                            return Err(IndexError::new(
                                "collection index out of range".to_string(),
                            ));
                        }
                    }
                } else {
                    return Err(IndexError::new("collection index out of range".to_string()));
                }
            }
        }
        {
            let sifr_generated_nested_assign_value = SifrInt::from_i64(1);
            {
                let sifr_generated_outer_raw = SifrInt::from_i64(1);
                let sifr_generated_outer_normalized =
                    sifr_generated_outer_raw.normalize_index_or_len(matrix.len());
                if let Some(sifr_generated_row) = matrix.get_mut(sifr_generated_outer_normalized) {
                    {
                        let sifr_generated_inner_raw = SifrInt::from_i64(1);
                        let sifr_generated_inner_normalized = sifr_generated_inner_raw
                            .normalize_index_or_len(sifr_generated_row.len());
                        if let Some(sifr_generated_elem) =
                            sifr_generated_row.get_mut(sifr_generated_inner_normalized)
                        {
                            *sifr_generated_elem = sifr_generated_nested_assign_value;
                        } else {
                            return Err(IndexError::new(
                                "collection index out of range".to_string(),
                            ));
                        }
                    }
                } else {
                    return Err(IndexError::new("collection index out of range".to_string()));
                }
            }
        }
        {
            let sifr_generated_nested_assign_value = SifrInt::from_i64(1);
            {
                let sifr_generated_outer_raw = SifrInt::from_i64(2);
                let sifr_generated_outer_normalized =
                    sifr_generated_outer_raw.normalize_index_or_len(matrix.len());
                if let Some(sifr_generated_row) = matrix.get_mut(sifr_generated_outer_normalized) {
                    {
                        let sifr_generated_inner_raw = SifrInt::from_i64(2);
                        let sifr_generated_inner_normalized = sifr_generated_inner_raw
                            .normalize_index_or_len(sifr_generated_row.len());
                        if let Some(sifr_generated_elem) =
                            sifr_generated_row.get_mut(sifr_generated_inner_normalized)
                        {
                            *sifr_generated_elem = sifr_generated_nested_assign_value;
                        } else {
                            return Err(IndexError::new(
                                "collection index out of range".to_string(),
                            ));
                        }
                    }
                } else {
                    return Err(IndexError::new("collection index out of range".to_string()));
                }
            }
        }
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        return;
    }
    println!(
        "{}",
        {
            let sifr_generated_outer_list = &matrix;
            let sifr_generated_outer_i = SifrInt::from_i64(0);
            let sifr_generated_outer_norm =
                sifr_generated_outer_i.normalize_index_or_len(sifr_generated_outer_list.len());
            sifr_generated_outer_list
                .get(::sifr_runtime::to_usize_proven(&sifr_generated_outer_norm))
                .and_then(|sifr_generated_row_5f5f736966725f726f77| {
                    let sifr_generated_inner_i = SifrInt::from_i64(0);
                    let sifr_generated_inner_norm = sifr_generated_inner_i
                        .normalize_index_or_len(sifr_generated_row_5f5f736966725f726f77.len());
                    sifr_generated_row_5f5f736966725f726f77
                        .get(::sifr_runtime::to_usize_proven(&sifr_generated_inner_norm))
                        .cloned()
                })
        }
        .map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    println!(
        "{}",
        {
            let sifr_generated_outer_list = &matrix;
            let sifr_generated_outer_i = SifrInt::from_i64(1);
            let sifr_generated_outer_norm =
                sifr_generated_outer_i.normalize_index_or_len(sifr_generated_outer_list.len());
            sifr_generated_outer_list
                .get(::sifr_runtime::to_usize_proven(&sifr_generated_outer_norm))
                .and_then(|sifr_generated_row_5f5f736966725f726f77| {
                    let sifr_generated_inner_i = SifrInt::from_i64(1);
                    let sifr_generated_inner_norm = sifr_generated_inner_i
                        .normalize_index_or_len(sifr_generated_row_5f5f736966725f726f77.len());
                    sifr_generated_row_5f5f736966725f726f77
                        .get(::sifr_runtime::to_usize_proven(&sifr_generated_inner_norm))
                        .cloned()
                })
        }
        .map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    println!(
        "{}",
        {
            let sifr_generated_outer_list = &matrix;
            let sifr_generated_outer_i = SifrInt::from_i64(2);
            let sifr_generated_outer_norm =
                sifr_generated_outer_i.normalize_index_or_len(sifr_generated_outer_list.len());
            sifr_generated_outer_list
                .get(::sifr_runtime::to_usize_proven(&sifr_generated_outer_norm))
                .and_then(|sifr_generated_row_5f5f736966725f726f77| {
                    let sifr_generated_inner_i = SifrInt::from_i64(2);
                    let sifr_generated_inner_norm = sifr_generated_inner_i
                        .normalize_index_or_len(sifr_generated_row_5f5f736966725f726f77.len());
                    sifr_generated_row_5f5f736966725f726f77
                        .get(::sifr_runtime::to_usize_proven(&sifr_generated_inner_norm))
                        .cloned()
                })
        }
        .map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    let mut scores: Vec<SifrInt> = vec![
        SifrInt::from_i64(10),
        SifrInt::from_i64(20),
        SifrInt::from_i64(30),
    ];
    let sifr_generated_try_res: Result<(), IndexError> = (|| {
        {
            let sifr_generated_assign_value = SifrInt::from_i64(5);
            {
                let sifr_generated_index_raw = SifrInt::from_i64(0);
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(scores.len());
                if let Some(sifr_generated_elem) = scores.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem += sifr_generated_assign_value;
                } else {
                    return Err(IndexError::new("collection index out of range".to_string()));
                }
            }
        }
        {
            let sifr_generated_assign_value = SifrInt::from_i64(3);
            {
                let sifr_generated_index_raw = SifrInt::from_i64(1);
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(scores.len());
                if let Some(sifr_generated_elem) = scores.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem -= sifr_generated_assign_value;
                } else {
                    return Err(IndexError::new("collection index out of range".to_string()));
                }
            }
        }
        {
            let sifr_generated_assign_value = SifrInt::from_i64(2);
            {
                let sifr_generated_index_raw = SifrInt::from_i64(2);
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(scores.len());
                if let Some(sifr_generated_elem) = scores.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem *= sifr_generated_assign_value;
                } else {
                    return Err(IndexError::new("collection index out of range".to_string()));
                }
            }
        }
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        return;
    }
    let s0: Option<SifrInt> = {
        let sifr_generated_checked_read_collection = &scores;
        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    };
    let s1: Option<SifrInt> = {
        let sifr_generated_checked_read_collection = &scores;
        let sifr_generated_checked_read_index = SifrInt::from_i64(1);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    };
    let s2: Option<SifrInt> = {
        let sifr_generated_checked_read_collection = &scores;
        let sifr_generated_checked_read_index = SifrInt::from_i64(2);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    };
    if let Some(s0) = s0.clone() {
        println!("{s0}");
    }
    if let Some(s1) = s1.clone() {
        println!("{s1}");
    }
    if let Some(s2) = s2.clone() {
        println!("{s2}");
    }
    let mut c = Counter::new(SifrInt::from_i64(0));
    c.increment();
    c.increment();
    c.increment();
    println!("{}", c.count.clone());
}
